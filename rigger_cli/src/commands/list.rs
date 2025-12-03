//! Implementation of the 'rig list' command.
//!
//! Lists tasks from the SQLite database with optional filtering and sorting.
//!
//! Revision History
//! - 2025-11-23T14:30:00Z @AI: Rename taskmaster to rigger throughout codebase.
//! - 2025-11-22T16:50:00Z @AI: Initial list command implementation for Rigger Phase 0 Sprint 0.2.

/// Executes the 'rig list' command.
///
/// Reads tasks from the SQLite database in .rigger/tasks.db and displays
/// them in a formatted table with optional filtering and sorting.
///
/// # Arguments
///
/// * `status` - Optional status filter (e.g., "todo", "in_progress", "completed")
/// * `assignee` - Optional assignee filter
/// * `sort` - Sort field (created_at, updated_at, title, due_date, status)
/// * `limit` - Maximum number of tasks to display
/// * `offset` - Number of tasks to skip (for pagination)
///
/// # Errors
///
/// Returns an error if:
/// - .rigger directory doesn't exist (run 'rig init' first)
/// - Database connection fails
/// - Query execution fails
pub async fn execute(
    status: std::option::Option<&str>,
    assignee: std::option::Option<&str>,
    sort: &str,
    limit: std::option::Option<&str>,
    offset: std::option::Option<&str>,
) -> anyhow::Result<()> {
    // Check if .rigger exists
    let current_dir = std::env::current_dir()?;
    let rigger_dir = current_dir.join(".rigger");

    if !rigger_dir.exists() {
        anyhow::bail!(
            ".rigger directory not found.\nRun 'rig init' first to initialize the project."
        );
    }

    // Connect to database
    let db_path = rigger_dir.join("tasks.db");
    let db_url = std::format!("sqlite:{}", db_path.display());

    let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&db_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    // Build filter
    let filter = if let std::option::Option::Some(status_str) = status {
        // Parse status string to TaskStatus enum
        let task_status = match status_str.to_lowercase().as_str() {
            "todo" => task_manager::domain::task_status::TaskStatus::Todo,
            "in_progress" | "inprogress" => task_manager::domain::task_status::TaskStatus::InProgress,
            "completed" => task_manager::domain::task_status::TaskStatus::Completed,
            "archived" => task_manager::domain::task_status::TaskStatus::Archived,
            _ => {
                anyhow::bail!(
                    "Invalid status: '{}'. Valid values: todo, in_progress, completed, archived",
                    status_str
                );
            }
        };
        task_manager::ports::task_repository_port::TaskFilter::ByStatus(task_status)
    } else if let std::option::Option::Some(assignee_str) = assignee {
        task_manager::ports::task_repository_port::TaskFilter::ByAgentPersona(std::string::String::from(assignee_str))
    } else {
        task_manager::ports::task_repository_port::TaskFilter::All
    };

    // Build sort options
    let sort_key = match sort {
        "created_at" => task_manager::ports::task_repository_port::TaskSortKey::CreatedAt,
        "updated_at" => task_manager::ports::task_repository_port::TaskSortKey::UpdatedAt,
        "title" => task_manager::ports::task_repository_port::TaskSortKey::Title,
        "status" => task_manager::ports::task_repository_port::TaskSortKey::Status,
        "due_date" => task_manager::ports::task_repository_port::TaskSortKey::DueDate,
        other => {
            anyhow::bail!(
                "Invalid sort field: '{}'. Valid values: created_at, updated_at, title, status, due_date",
                other
            );
        }
    };

    // Parse limit and offset
    let limit_u32 = if let std::option::Option::Some(l_str) = limit {
        std::option::Option::Some(l_str.parse::<u32>().map_err(|_| {
            anyhow::anyhow!("Invalid limit value: '{}'. Must be a positive integer.", l_str)
        })?)
    } else {
        std::option::Option::None
    };

    let offset_u64 = if let std::option::Option::Some(o_str) = offset {
        std::option::Option::Some(o_str.parse::<u64>().map_err(|_| {
            anyhow::anyhow!("Invalid offset value: '{}'. Must be a positive integer.", o_str)
        })?)
    } else {
        std::option::Option::None
    };

    let find_options = hexser::ports::repository::FindOptions {
        sort: std::option::Option::Some(std::vec![hexser::ports::repository::Sort {
            key: sort_key,
            direction: hexser::ports::repository::Direction::Desc,
        }]),
        limit: limit_u32,
        offset: offset_u64,
    };

    // Query tasks
    let tasks = {
        use hexser::ports::repository::QueryRepository;
        adapter.find(&filter, find_options)?
    };

    // Display tasks
    crate::display::task_table::display_tasks_table(&tasks);

    std::result::Result::Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_fails_without_init() {
        // Test: Validates list command fails if .rigger doesn't exist.
        // Justification: User must run init before using other commands.
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = super::execute(std::option::Option::None, std::option::Option::None, "created_at", std::option::Option::None, std::option::Option::None).await;
        std::assert!(result.is_err(), "List should fail if .rigger doesn't exist");

        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn test_list_with_empty_database() {
        // Test: Validates list handles empty database gracefully.
        // Justification: Newly initialized projects have no tasks yet.
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Initialize
        crate::commands::init::execute().await.unwrap();

        // List tasks
        let result = super::execute(std::option::Option::None, std::option::Option::None, "created_at", std::option::Option::None, std::option::Option::None).await;
        std::assert!(result.is_ok(), "List should succeed with empty database");

        // Cleanup (ignore errors if already cleaned)
        let _ = std::env::set_current_dir(original_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
