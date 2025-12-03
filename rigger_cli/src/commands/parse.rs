//! Implementation of the 'rigparse <PRD_FILE>' command.
//!
//! Parses a PRD markdown file and generates tasks using LLM-based decomposition.
//! Ingests PRD content into RAG knowledge base with vector embeddings for semantic search.
//!
//! Revision History
//! - 2025-11-28T20:45:00Z @AI: Add RAG artifact ingestion after task generation (Phase 3 Task 4.2).
//! - 2025-11-27T09:00:00Z @AI: Add auto-decomposition for complex tasks. After saving generated tasks, iterate through them and auto-decompose any with complexity >= 7. For each complex task: (1) call parser.decompose_task() to generate 3-5 sub-tasks, (2) save sub-tasks to database, (3) update parent task with subtask_ids and Decomposed status. Provides progress feedback ("🔄 Decomposing complex task...") and summary stats. Decomposition failures are non-fatal - logs warning and continues with original task.
//! - 2025-11-25T20:47:00Z @AI: Fix "runtime within runtime" error by using save_async() instead of blocking save().
//! - 2025-11-22T17:10:00Z @AI: Full implementation of parse command for Rigger Phase 0 Sprint 0.3.

/// Executes the 'rigparse <PRD_FILE>' command.
///
/// This command:
/// 1. Reads the PRD markdown file
/// 2. Parses it using task_manager::utils::prd_parser
/// 3. Uses Rig-powered PRD parser to generate tasks via LLM
/// 4. Saves all tasks to SQLite database
/// 5. Prints summary of results
///
/// # Arguments
///
/// * `prd_file` - Path to the PRD markdown file
///
/// # Errors
///
/// Returns an error if:
/// - PRD file doesn't exist or can't be read
/// - .rigdirectory doesn't exist (run 'riginit' first)
/// - PRD parsing fails
/// - LLM request fails
/// - Database operations fail
pub async fn execute(prd_file: &str) -> anyhow::Result<()> {
    // Check if .rigexists
    let current_dir = std::env::current_dir()?;
    let taskmaster_dir = current_dir.join(".rigger");

    if !taskmaster_dir.exists() {
        anyhow::bail!(
            ".rig directory not found.\nRun 'rig init' first to initialize the project."
        );
    }

    // Read PRD file
    let prd_path = std::path::Path::new(prd_file);
    if !prd_path.exists() {
        anyhow::bail!("PRD file not found: {}", prd_file);
    }

    let prd_content = std::fs::read_to_string(prd_path)
        .map_err(|e| anyhow::anyhow!("Failed to read PRD file: {}", e))?;

    println!("Reading PRD from: {}", prd_file);

    // Parse PRD markdown (using placeholder project ID for standalone parse command)
    let prd = task_manager::infrastructure::markdown_parsers::prd_parser::parse_prd_markdown("default-project", &prd_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse PRD: {}", e))?;

    println!("✓ Parsed PRD: {}", prd.title);
    println!("  Objectives: {}", prd.objectives.len());
    println!("  Tech Stack: {}", prd.tech_stack.len());
    println!("  Constraints: {}", prd.constraints.len());
    println!();

    // Read config to determine provider
    let config_path = taskmaster_dir.join("config.json");
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config.json: {}", e))?;
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    let provider = config["provider"]
        .as_str()
        .unwrap_or("ollama");

    let model_name = config["model"]["main"]
        .as_str()
        .unwrap_or("llama3.2:latest");

    println!("Generating tasks using {} with {}...", provider, model_name);

    // Define database paths early for persona queries
    let db_path = taskmaster_dir.join("tasks.db");
    let db_url = std::format!("sqlite:{}", db_path.display());

    // Connect to database for both persona queries and task storage
    let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&db_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    // Create PRD parser adapter based on provider
    let tasks = match provider {
        "ollama" => {
            use task_orchestrator::ports::prd_parser_port::PRDParserPort;

            // Query personas from database for task assignment
            let persona_rows = sqlx::query("SELECT id, project_id, name, role, description, llm_provider, llm_model, is_default, created_at, updated_at FROM personas")
                .fetch_all(adapter.pool())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to query personas: {}", e))?;

            let mut personas = std::vec::Vec::new();
            for row in persona_rows {
                use sqlx::Row;
                let persona = task_manager::domain::persona::Persona {
                    id: row.get(0),
                    project_id: row.get(1),
                    name: row.get(2),
                    role: row.get(3),
                    description: row.get(4),
                    llm_provider: row.get(5),
                    llm_model: row.get(6),
                    is_default: row.get(7),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>(8))
                        .map_err(|e| anyhow::anyhow!("Invalid created_at timestamp: {}", e))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>(9))
                        .map_err(|e| anyhow::anyhow!("Invalid updated_at timestamp: {}", e))?
                        .with_timezone(&chrono::Utc),
                    enabled_tools: std::vec::Vec::new(), // Will be populated from persona_tools if needed
                };
                personas.push(persona);
            }

            eprintln!("[PRD Parser] Loaded {} personas for task assignment", personas.len());

            // Extract fallback model from config
            let fallback_model = config["task_tools"]["fallback"]["model"]
                .as_str()
                .unwrap_or(model_name);

            let parser = task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter::new(
                model_name.to_string(),
                fallback_model.to_string(),
                personas
            );

            parser
                .parse_prd_to_tasks(&prd)
                .await
                .map_err(|e| anyhow::anyhow!("Task generation failed: {}", e))?
        }
        other => {
            anyhow::bail!("Unsupported provider: '{}'. Currently only 'ollama' is supported.", other);
        }
    };

    println!("✓ Generated {} tasks", tasks.len());
    println!();

    // Save tasks to database (reusing adapter from above)
    for task in &tasks {
        task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, task.clone()).await?;
    }

    println!("✓ Saved {} tasks to {}", tasks.len(), db_path.display());
    println!();

    // Ingest PRD content as artifacts for RAG
    println!("📚 Ingesting PRD content for semantic search...");
    match ingest_prd_artifacts(&prd, &prd_content, &db_url, provider, model_name).await {
        std::result::Result::Ok(artifact_count) => {
            println!("✓ Ingested {} knowledge artifacts with embeddings", artifact_count);
            println!();
        }
        std::result::Result::Err(e) => {
            eprintln!("⚠️  RAG ingestion failed (non-fatal): {}", e);
            eprintln!("  → Continuing with task generation");
            println!();
        }
    }

    // Auto-decompose complex tasks (complexity >= 7)
    let mut total_subtasks = 0;
    for task in &tasks {
        if let std::option::Option::Some(complexity) = task.complexity {
            if complexity >= 7 {
                println!("🔄 Decomposing complex task (complexity {}): {}", complexity, task.title);

                // Recreate parser for decomposition (needs same config)
                let parser = task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter::new(
                    model_name.to_string(),
                    config["task_tools"]["fallback"]["model"].as_str().unwrap_or(model_name).to_string(),
                    std::vec::Vec::new() // Personas already validated in original tasks
                );

                match parser.decompose_task(task, &prd_content).await {
                    std::result::Result::Ok(subtasks) => {
                        println!("  ✓ Generated {} sub-tasks", subtasks.len());

                        // Save sub-tasks
                        for subtask in &subtasks {
                            task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, subtask.clone()).await?;
                        }

                        // Update parent task with subtask IDs and Decomposed status
                        let mut updated_parent = task.clone();
                        updated_parent.subtask_ids = subtasks.iter().map(|st| st.id.clone()).collect();
                        updated_parent.status = task_manager::domain::task_status::TaskStatus::Decomposed;
                        task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, updated_parent).await?;

                        total_subtasks += subtasks.len();
                    }
                    std::result::Result::Err(e) => {
                        eprintln!("  ⚠️  Decomposition failed: {}", e);
                        eprintln!("  → Continuing with original task");
                    }
                }
            }
        }
    }

    if total_subtasks > 0 {
        println!();
        println!("✓ Auto-decomposed {} complex tasks into {} sub-tasks",
            tasks.iter().filter(|t| t.complexity.unwrap_or(0) >= 7).count(),
            total_subtasks
        );
        println!();
    }

    // Print next steps
    println!("Next steps:");
    println!("  1. View tasks: riglist");
    println!("  2. Execute a task: rigdo <TASK_ID>");
    println!();

    std::result::Result::Ok(())
}

/// Helper function to ingest PRD content as artifacts for RAG.
///
/// This function:
/// 1. Creates an artifact repository adapter connected to the database
/// 2. Creates an embedding adapter using the configured provider
/// 3. Creates an artifact service to coordinate ingestion
/// 4. Calls the service to chunk, embed, and persist the PRD content
///
/// # Arguments
///
/// * `prd` - The parsed PRD domain entity
/// * `prd_content` - Full markdown text of the PRD
/// * `db_url` - SQLite database URL
/// * `provider` - LLM provider name (for embedding model selection)
/// * `model_name` - Model name (for logging purposes)
///
/// # Returns
///
/// Returns the number of artifacts successfully ingested, or an error.
///
/// # Errors
///
/// This function returns errors for:
/// - Database connection failures
/// - Embedding adapter creation failures
/// - Artifact ingestion failures
async fn ingest_prd_artifacts(
    prd: &task_manager::domain::prd::PRD,
    prd_content: &str,
    db_url: &str,
    provider: &str,
    _model_name: &str,
) -> std::result::Result<usize, String> {
    // 0. Ensure default project exists (for foreign key constraint)
    let project_id = String::from("default-project");
    let task_adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(db_url)
        .await
        .map_err(|e| std::format!("Failed to connect task adapter: {}", e))?;

    // Create default project if it doesn't exist
    sqlx::query("INSERT OR IGNORE INTO projects (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)")
        .bind(&project_id)
        .bind("Default Project")
        .bind("Auto-created default project for artifact storage")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(task_adapter.pool())
        .await
        .map_err(|e| std::format!("Failed to create default project: {}", e))?;

    // 1. Create artifact repository adapter
    let artifact_adapter = task_manager::adapters::sqlite_artifact_adapter::SqliteArtifactAdapter::connect_and_init(db_url)
        .await
        .map_err(|e| std::format!("Failed to connect artifact repository: {}", e))?;

    // 2. Create embedding adapter using provider factory
    let provider_factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new(provider, "default")
        .map_err(|e| std::format!("Failed to create provider factory: {}", e))?;

    let embedding_adapter = provider_factory.create_embedding_adapter()
        .map_err(|e| std::format!("Failed to create embedding adapter: {}", e))?;

    // 3. Create artifact service
    let artifact_service = task_orchestrator::services::artifact_service::ArtifactService::new(
        std::sync::Arc::new(std::sync::Mutex::new(artifact_adapter)),
        embedding_adapter,
    );

    // 4. Ingest PRD content
    let artifacts = artifact_service.ingest_prd(
        String::from("default-project"), // Use same project ID as PRD parser
        prd.id.clone(),
        prd_content.to_string(),
    ).await?;

    std::result::Result::Ok(artifacts.len())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[serial_test::serial]
    async fn test_parse_fails_without_init() {
        // Test: Validates parse command fails if .rigdoesn't exist.
        // Justification: User must run init before using other commands.
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = super::execute("nonexistent.md").await;
        std::assert!(result.is_err(), "Parse should fail if .rigdoesn't exist");

        // Cleanup (ignore errors if already cleaned)
        let _ = std::env::set_current_dir(original_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_parse_fails_with_nonexistent_file() {
        // Test: Validates parse command fails if PRD file doesn't exist.
        // Justification: Must validate file exists before processing.
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Initialize .rigger
        crate::commands::init::execute().await.unwrap();

        // Try to parse nonexistent file
        let result = super::execute("nonexistent.md").await;
        std::assert!(result.is_err(), "Parse should fail if PRD file doesn't exist");
        std::assert!(result.unwrap_err().to_string().contains("not found"));

        // Cleanup (ignore errors if already cleaned)
        let _ = std::env::set_current_dir(original_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_ingest_prd_artifacts_helper() {
        // Test: Validates RAG artifact ingestion helper function.
        // Justification: Ensures PRD content is chunked and embedded correctly.
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let db_path = temp_dir.join("test.db");
        let db_url = std::format!("sqlite:{}", db_path.display());

        // Create sample PRD
        let prd = task_manager::domain::prd::PRD {
            id: String::from("test-prd-123"),
            title: String::from("Test PRD"),
            project_id: String::from("default-project"),
            objectives: std::vec![String::from("Build feature")],
            tech_stack: std::vec![String::from("Rust")],
            constraints: std::vec![String::from("Must be fast")],
            raw_content: String::from("# Test PRD\n\nBuild a feature."),
            created_at: chrono::Utc::now(),
        };

        let prd_content = "# Test PRD\n\nThis is the first paragraph.\n\nThis is the second paragraph.\n\nThis is the third paragraph.";

        // Call ingestion helper
        let result = super::ingest_prd_artifacts(
            &prd,
            prd_content,
            &db_url,
            "ollama",
            "llama3.2:latest",
        ).await;

        // Cleanup (ignore errors if already cleaned)
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Validate result
        // Note: This test will use fallback embeddings since Ollama may not be running
        std::assert!(result.is_ok(), "Ingestion should succeed with fallback embeddings: {:?}", result);
        let artifact_count = result.unwrap();
        std::assert!(artifact_count >= 3, "Should create at least 3 artifacts from 3 paragraphs, got {}", artifact_count);
    }
}
