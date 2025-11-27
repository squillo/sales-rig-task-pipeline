//! Integration test for full PRD → Tasks workflow.
//!
//! Tests the complete pipeline:
//! 1. riginit
//! 2. rigparse <PRD_FILE>
//! 3. riglist
//! 4. rigdo <TASK_ID>
//!
//! This test requires Ollama running on localhost:11434 with llama3.1 model.
//! Run with: `cargo test --test integration_prd_workflow -- --ignored`
//!
//! Revision History
//! - 2025-11-26T05:10:00Z @AI: Add test_project_creation_from_prd to verify Project entity is created and linked when PRD is parsed.
//! - 2025-11-22T18:00:00Z @AI: Create integration test for Rigger Phase 0 Sprint 0.3.

#[tokio::test]
#[ignore] // Requires Ollama running
async fn test_full_prd_to_tasks_workflow() {
    // Test: Validates complete PRD parsing and task execution pipeline.
    // Justification: End-to-end verification of Rigger Phase 0 functionality.

    // 1. Setup: Create temp directory
    let temp_dir = std::env::temp_dir().join(std::format!(
        "taskmaster_integration_test_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&temp_dir).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    // 2. Create sample PRD file
    let prd_content = r#"# Test Project PRD

## Objectives
- Build a simple CLI tool
- Implement basic CRUD operations
- Add unit tests

## Tech Stack
- Rust
- Clap
- SQLite

## Constraints
- Must be cross-platform
- No unsafe code
"#;
    let prd_path = temp_dir.join("test_prd.md");
    std::fs::write(&prd_path, prd_content).unwrap();

    // 3. Initialize .rigger
    let init_result = rigger_cli::commands::init::execute().await;
    std::assert!(
        init_result.is_ok(),
        "Init should succeed: {:?}",
        init_result.err()
    );
    std::assert!(temp_dir.join(".rigger").exists());
    std::assert!(temp_dir.join(".rigger/tasks.db").exists());
    std::assert!(temp_dir.join(".rigger/config.json").exists());

    // 4. Parse PRD to generate tasks
    let parse_result = rigger_cli::commands::parse::execute(prd_path.to_str().unwrap()).await;
    std::assert!(
        parse_result.is_ok(),
        "Parse should succeed: {:?}",
        parse_result.err()
    );

    // 5. List tasks to verify they were created
    let db_path = temp_dir.join(".rigger/tasks.db");
    let db_url = std::format!("sqlite:{}", db_path.display());

    let mut adapter =
        task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&db_url)
            .await
            .unwrap();

    let filter = task_manager::ports::task_repository_port::TaskFilter::All;
    let find_options = hexser::ports::repository::FindOptions {
        sort: std::option::Option::Some(std::vec![hexser::ports::repository::Sort {
            key: task_manager::ports::task_repository_port::TaskSortKey::CreatedAt,
            direction: hexser::ports::repository::Direction::Asc,
        }]),
        limit: std::option::Option::None,
        offset: std::option::Option::None,
    };
    let tasks: std::vec::Vec<task_manager::domain::task::Task> = {
        use hexser::ports::repository::QueryRepository;
        adapter.find(&filter, find_options).unwrap()
    };

    std::assert!(
        !tasks.is_empty(),
        "Should have generated at least one task from PRD"
    );
    std::println!("✓ Generated {} tasks from PRD", tasks.len());

    // 6. Verify task structure
    let first_task = &tasks[0];
    std::assert!(!first_task.id.is_empty(), "Task should have an ID");
    std::assert!(!first_task.title.is_empty(), "Task should have a title");
    std::assert_eq!(
        first_task.status,
        task_manager::domain::task_status::TaskStatus::Todo
    );
    std::assert!(
        first_task.source_prd_id.is_some(),
        "Task should be linked to PRD"
    );

    // 7. Execute first task (basic completion without orchestration)
    let do_result = rigger_cli::commands::do_task::execute(&first_task.id).await;
    std::assert!(
        do_result.is_ok(),
        "Do command should succeed: {:?}",
        do_result.err()
    );

    // 8. Verify task was marked as completed
    let updated_task: std::option::Option<task_manager::domain::task::Task> = {
        use hexser::ports::repository::QueryRepository;
        let filter =
            task_manager::ports::task_repository_port::TaskFilter::ById(first_task.id.clone());
        adapter.find_one(&filter).unwrap()
    };

    std::assert!(updated_task.is_some(), "Task should still exist");
    let updated_task = updated_task.unwrap();
    std::assert_eq!(
        updated_task.status,
        task_manager::domain::task_status::TaskStatus::Completed
    );
    std::println!("✓ Task marked as completed successfully");

    // Cleanup
    std::env::set_current_dir(original_dir).unwrap();
    std::fs::remove_dir_all(&temp_dir).unwrap();

    std::println!("✓ Full PRD → Tasks workflow test passed");
}

#[tokio::test]
async fn test_init_creates_directory_structure() {
    // Test: Validates init command creates proper directory structure.
    // Justification: Basic smoke test that doesn't require Ollama.

    let temp_dir = std::env::temp_dir().join(std::format!(
        "taskmaster_init_test_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&temp_dir).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let result = rigger_cli::commands::init::execute().await;
    if let std::result::Result::Err(ref e) = result {
        std::eprintln!("Init error: {}", e);
    }
    std::assert!(result.is_ok(), "Init failed: {:?}", result.err());

    // Verify structure
    std::assert!(temp_dir.join(".rigger").exists());
    std::assert!(temp_dir.join(".rigger/prds").exists());
    std::assert!(temp_dir.join(".rigger/tasks.db").exists());
    std::assert!(temp_dir.join(".rigger/config.json").exists());

    // Verify config content
    let config_content = std::fs::read_to_string(temp_dir.join(".rigger/config.json")).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    std::assert_eq!(config["provider"], "ollama");

    // Cleanup
    std::env::set_current_dir(original_dir).unwrap();
    std::fs::remove_dir_all(&temp_dir).unwrap();
}

#[tokio::test]
async fn test_init_fails_if_already_exists() {
    // Test: Validates init command fails gracefully if .rigalready exists.
    // Justification: Prevents accidental re-initialization.

    let temp_dir = std::env::temp_dir().join(std::format!(
        "taskmaster_init_exists_test_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&temp_dir).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    // First init should succeed
    let result1 = rigger_cli::commands::init::execute().await;
    if let std::result::Result::Err(ref e) = result1 {
        std::eprintln!("Init error: {}", e);
    }
    std::assert!(result1.is_ok(), "Init failed: {:?}", result1.err());

    // Second init should fail
    let result2 = rigger_cli::commands::init::execute().await;
    std::assert!(result2.is_err());
    std::assert!(result2
        .unwrap_err()
        .to_string()
        .contains("already exists"));

    // Cleanup
    std::env::set_current_dir(original_dir).unwrap();
    std::fs::remove_dir_all(&temp_dir).unwrap();
}

#[tokio::test]
#[ignore] // Requires Ollama running
async fn test_project_creation_from_prd() {
    // Test: Validates that parsing a PRD creates a Project entity and links all tasks.
    // Justification: Ensures Project → PRD → Tasks hierarchy is established correctly.

    use sqlx::Row;

    // 1. Setup: Create temp directory
    let temp_dir = std::env::temp_dir().join(std::format!(
        "taskmaster_project_test_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&temp_dir).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    // 2. Create sample PRD file
    let prd_content = r#"# E-Commerce Platform PRD

## Objectives
- Build scalable product catalog
- Implement shopping cart functionality
- Add payment processing

## Tech Stack
- Rust
- PostgreSQL
- React

## Constraints
- Must support 10k concurrent users
- PCI DSS compliant
"#;
    let prd_path = temp_dir.join("ecommerce_prd.md");
    std::fs::write(&prd_path, prd_content).unwrap();

    // 3. Initialize .rigger
    let init_result = rigger_cli::commands::init::execute().await;
    std::assert!(
        init_result.is_ok(),
        "Init should succeed: {:?}",
        init_result.err()
    );

    // 4. Parse PRD to generate tasks (this should create Project)
    let parse_result = rigger_cli::commands::parse::execute(prd_path.to_str().unwrap()).await;
    std::assert!(
        parse_result.is_ok(),
        "Parse should succeed: {:?}",
        parse_result.err()
    );

    // 5. Connect to database and verify Project was created
    let db_path = temp_dir.join(".rigger/tasks.db");
    let db_url = std::format!("sqlite:{}", db_path.display());

    let adapter =
        task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&db_url)
            .await
            .unwrap();

    // 6. Query projects table
    let projects_result = sqlx::query("SELECT id, name, description, created_at, prd_ids_json FROM projects")
        .fetch_all(adapter.pool())
        .await;

    std::assert!(
        projects_result.is_ok(),
        "Should be able to query projects table: {:?}",
        projects_result.err()
    );

    let projects = projects_result.unwrap();
    std::assert!(
        !projects.is_empty(),
        "Should have created at least one project from PRD"
    );

    let project_row = &projects[0];
    let project_id: String = project_row.get(0);
    let project_name: String = project_row.get(1);
    let project_description: std::option::Option<String> = project_row.get(2);

    std::println!("✓ Found Project: id={}, name={}", project_id, project_name);
    std::assert_eq!(project_name, "E-Commerce Platform PRD", "Project name should match PRD title");
    std::assert!(project_description.is_some(), "Project should have auto-generated description");
    std::assert!(
        project_description.unwrap().contains("ecommerce_prd.md"),
        "Description should reference PRD filename"
    );

    // 7. Query prds table and verify linkage
    let prds_result = sqlx::query("SELECT id, project_id, title FROM prds")
        .fetch_all(adapter.pool())
        .await;

    std::assert!(
        prds_result.is_ok(),
        "Should be able to query prds table: {:?}",
        prds_result.err()
    );

    let prds = prds_result.unwrap();
    std::assert!(!prds.is_empty(), "Should have created PRD entity");

    let prd_row = &prds[0];
    let prd_id: String = prd_row.get(0);
    let prd_project_id: String = prd_row.get(1);
    let prd_title: String = prd_row.get(2);

    std::println!("✓ Found PRD: id={}, project_id={}, title={}", prd_id, prd_project_id, prd_title);
    std::assert_eq!(prd_project_id, project_id, "PRD should be linked to Project");
    std::assert_eq!(prd_title, "E-Commerce Platform PRD");

    // 8. Verify all tasks are linked to the PRD
    let filter = task_manager::ports::task_repository_port::TaskFilter::All;
    let find_options = hexser::ports::repository::FindOptions {
        sort: std::option::Option::None,
        limit: std::option::Option::None,
        offset: std::option::Option::None,
    };

    let tasks: std::vec::Vec<task_manager::domain::task::Task> = {
        use hexser::ports::repository::QueryRepository;
        adapter.find(&filter, find_options).unwrap()
    };

    std::assert!(
        !tasks.is_empty(),
        "Should have generated tasks from PRD"
    );

    for task in &tasks {
        std::assert!(
            task.source_prd_id.is_some(),
            "Task '{}' should have source_prd_id set",
            task.title
        );
        std::assert_eq!(
            task.source_prd_id.as_ref().unwrap(),
            &prd_id,
            "Task '{}' should be linked to the correct PRD",
            task.title
        );
    }

    std::println!(
        "✓ All {} tasks are correctly linked to PRD {}",
        tasks.len(),
        prd_id
    );

    // 9. Verify complete hierarchy: Project → PRD → Tasks
    std::println!("✓ Complete hierarchy verified:");
    std::println!("  Project: {} (id={})", project_name, project_id);
    std::println!("    └─ PRD: {} (id={})", prd_title, prd_id);
    for task in &tasks {
        std::println!("       └─ Task: {} (id={})", task.title, task.id);
    }

    // Cleanup
    std::env::set_current_dir(original_dir).unwrap();
    std::fs::remove_dir_all(&temp_dir).unwrap();

    std::println!("✓ Project creation from PRD test passed");
}
