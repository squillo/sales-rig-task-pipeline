//! Implementation of the 'rig init' command.
//!
//! Creates the .rigger directory structure with configuration files
//! and initializes the SQLite database for task storage.
//!
//! Revision History
//! - 2025-12-04T20:00:00Z @AI: Update to generate rigger_core v3.0 config with full provider support.
//! - 2025-11-23T14:30:00Z @AI: Rename taskmaster to rigger throughout codebase.
//! - 2025-11-22T19:00:00Z @AI: Rename CLI command from 'taskmaster' to 'rig'; fix SQLite database file creation.
//! - 2025-11-22T16:35:00Z @AI: Initial init command implementation for Rigger Phase 0 Sprint 0.2.

/// Executes the 'rig init' command.
///
/// Creates the .rigger directory in the current working directory with:
/// - config.json: Configuration for LLM provider and model settings
/// - tasks.db: SQLite database for task persistence
/// - prds/: Directory for storing PRD markdown files
///
/// # Errors
///
/// Returns an error if:
/// - .rigger directory already exists
/// - Filesystem operations fail (permissions, disk space)
/// - Database initialization fails
pub async fn execute() -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let rigger_dir = current_dir.join(".rigger");

    // Check if .rigger already exists
    if rigger_dir.exists() {
        anyhow::bail!(
            ".rigger directory already exists at {}\nUse 'rig list' to view existing tasks.",
            rigger_dir.display()
        );
    }

    // Create .rigger directory
    std::fs::create_dir(&rigger_dir)?;
    println!("✓ Created .rigger directory");

    // Create prds subdirectory
    let prds_dir = rigger_dir.join("prds");
    std::fs::create_dir(&prds_dir)?;
    println!("✓ Created .rigger/prds directory");

    // Create lib subdirectory for SQLite extensions
    let lib_dir = rigger_dir.join("lib");
    std::fs::create_dir(&lib_dir)?;

    // Try to copy sqlite-vec extension from common locations
    let vec_sources = std::vec![
        std::path::PathBuf::from("/tmp/vec0.dylib"),
        std::path::PathBuf::from("/opt/homebrew/lib/vec0.dylib"),
        std::path::PathBuf::from("/usr/local/lib/vec0.dylib"),
    ];

    let vec_dest = lib_dir.join("vec0.dylib");
    let mut vec_copied = false;

    for source in vec_sources {
        if source.exists() {
            if let std::result::Result::Ok(_) = std::fs::copy(&source, &vec_dest) {
                println!("✓ Installed sqlite-vec extension (RAG enabled)");
                vec_copied = true;
                break;
            }
        }
    }

    if !vec_copied {
        println!("⚠ sqlite-vec extension not found (RAG features will be disabled)");
        println!("  To enable RAG: Download vec0.dylib from https://github.com/asg017/sqlite-vec");
        println!("  and copy to .rigger/lib/vec0.dylib");
    }

    // Create config.json using rigger_core v3.0 format
    let mut providers = std::collections::HashMap::new();

    // Default to Ollama (no API key required)
    providers.insert(
        String::from("ollama"),
        rigger_core::config::ProviderConfig {
            provider_type: rigger_core::config::ProviderType::Ollama,
            base_url: String::from("http://localhost:11434"),
            api_key_env: None,
            timeout_seconds: 120,
            max_retries: 2,
            default_model: String::from("llama3.2"),
        },
    );

    // Create v3.0 config with Ollama as default for all slots
    let config = rigger_core::RiggerConfig {
        version: String::from("3.0"),
        database: rigger_core::config::DatabaseConfig {
            url: String::from("sqlite:.rigger/tasks.db"),
            auto_vacuum: true,
            pool_size: 5,
        },
        providers,
        task_slots: rigger_core::config::TaskSlotConfig {
            main: rigger_core::config::TaskSlot {
                provider: String::from("ollama"),
                model: String::from("llama3.2"),
                enabled: true,
                description: String::from("Primary task decomposition and generation"),
                streaming: None,
            },
            research: rigger_core::config::TaskSlot {
                provider: String::from("ollama"),
                model: String::from("llama3.2"),
                enabled: true,
                description: String::from("Web and artifact research"),
                streaming: None,
            },
            fallback: rigger_core::config::TaskSlot {
                provider: String::from("ollama"),
                model: String::from("llama3.2"),
                enabled: true,
                description: String::from("Fallback processing for errors"),
                streaming: None,
            },
            embedding: rigger_core::config::TaskSlot {
                provider: String::from("ollama"),
                model: String::from("nomic-embed-text"),
                enabled: true,
                description: String::from("Semantic search and RAG embeddings"),
                streaming: None,
            },
            vision: rigger_core::config::TaskSlot {
                provider: String::from("ollama"),
                model: String::from("llava:latest"),
                enabled: true,
                description: String::from("Image and PDF processing"),
                streaming: None,
            },
            chat_agent: rigger_core::config::TaskSlot {
                provider: String::from("ollama"),
                model: String::from("llama3.2"),
                enabled: true,
                description: String::from("Interactive chat agent with tool calling"),
                streaming: Some(true),
            },
        },
        performance: rigger_core::config::PerformanceConfig::default(),
        tui: rigger_core::config::TuiConfig::default(),
    };

    let config_path = rigger_dir.join("config.json");
    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
    println!("✓ Created config.json (v3.0 format)");

    // Initialize SQLite database
    let db_path = rigger_dir.join("tasks.db");

    // Create empty database file (SQLite/sqlx requirement for file-based databases)
    if !db_path.exists() {
        std::fs::File::create(&db_path)
            .map_err(|e| anyhow::anyhow!("Failed to create database file: {}", e))?;
    }

    // SQLx expects file path in format: sqlite:path/to/file.db
    let db_url = std::format!("sqlite:{}", db_path.display());

    let _adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&db_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?;
    println!("✓ Initialized tasks.db database");

    // Print success message with next steps
    println!("\n🎉 Rig Task Pipeline initialized successfully!\n");
    println!("Next steps:");
    println!("  1. Create a PRD markdown file (or use an existing one)");
    println!("  2. Run: rig parse <PRD_FILE>");
    println!("  3. View tasks: rig list");
    println!("  4. Execute a task: rig do <TASK_ID>");
    println!("  5. Launch TUI: rig tui\n");
    println!("Configuration (v3.0):");
    println!("  Default provider: Ollama (http://localhost:11434)");
    println!("  All task slots: llama3.2");
    println!("  Embedding: nomic-embed-text");
    println!("  Vision: llava:latest");
    println!("  Database: {}", db_path.display());
    println!("\n💡 Tip: Run 'rig config edit' to configure additional providers (Claude, GPT-4, etc.)\n");

    std::result::Result::Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[serial_test::serial]
    async fn test_init_creates_directory_structure() {
        // Test: Validates init command creates .rigger directory structure.
        // Justification: Ensures directory scaffold is correct for Rigger operation.

        // Create temp directory for test
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Run init
        let result = super::execute().await;
        std::assert!(result.is_ok(), "Init should succeed");

        // Verify directory structure
        let rigger_dir = temp_dir.join(".rigger");
        std::assert!(rigger_dir.exists(), ".rigger directory should exist");
        std::assert!(rigger_dir.join("prds").exists(), "prds directory should exist");
        std::assert!(rigger_dir.join("config.json").exists(), "config.json should exist");
        std::assert!(rigger_dir.join("tasks.db").exists(), "tasks.db should exist");

        // Verify config.json content (v3.0 format)
        let config_content = std::fs::read_to_string(rigger_dir.join("config.json")).unwrap();
        let config: rigger_core::RiggerConfig = serde_json::from_str(&config_content).unwrap();
        std::assert_eq!(config.version, "3.0");
        std::assert!(config.providers.contains_key("ollama"));
        std::assert_eq!(config.task_slots.main.provider, "ollama");
        std::assert_eq!(config.task_slots.main.model, "llama3.2");

        // Cleanup (ignore errors if already cleaned)
        let _ = std::env::set_current_dir(original_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_init_fails_if_directory_exists() {
        // Test: Validates init fails gracefully if .rigger already exists.
        // Justification: Prevents accidentally overwriting existing Rigger configuration.

        let temp_dir = std::env::temp_dir().join(std::format!("rigger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();
        let rigger_dir = temp_dir.join(".rigger");
        std::fs::create_dir(&rigger_dir).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Run init - should fail
        let result = super::execute().await;
        std::assert!(result.is_err(), "Init should fail if .rigger exists");

        // Cleanup (ignore errors if already cleaned)
        let _ = std::env::set_current_dir(original_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
