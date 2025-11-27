//! Implementation of the 'rigparse <PRD_FILE>' command.
//!
//! Parses a PRD markdown file and generates tasks using LLM-based decomposition.
//!
//! Revision History
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

    // Print next steps
    println!("Next steps:");
    println!("  1. View tasks: riglist");
    println!("  2. Execute a task: rigdo <TASK_ID>");
    println!();

    std::result::Result::Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_parse_fails_without_init() {
        // Test: Validates parse command fails if .rigdoesn't exist.
        // Justification: User must run init before using other commands.
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = super::execute("nonexistent.md").await;
        std::assert!(result.is_err(), "Parse should fail if .rigdoesn't exist");

        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
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

        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
