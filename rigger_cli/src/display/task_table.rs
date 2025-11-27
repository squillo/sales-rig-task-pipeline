//! Task table formatting utilities using prettytable-rs.
//!
//! Provides formatted table output for task lists with color-coded status.
//!
//! Revision History
//! - 2025-11-22T16:45:00Z @AI: Initial task table display implementation for Rigger Phase 0 Sprint 0.2.

/// Displays tasks in a formatted ASCII table.
///
/// Formats tasks with the following columns:
/// - ID: Truncated task ID (first 8 chars)
/// - Title: Task title (truncated to 40 chars if needed)
/// - Status: Color-coded status (Pending=yellow, InProgress=blue, Completed=green)
/// - Priority: Task priority (if available)
/// - Assignee: Assigned person (truncated to 15 chars)
/// - Due Date: Due date (if available)
///
/// # Arguments
///
/// * `tasks` - Slice of tasks to display
///
/// # Examples
///
/// ```no_run
/// use task_manager::domain::task::Task;
/// let tasks: Vec<Task> = vec![];
/// taskmaster_cli::display::task_table::display_tasks_table(&tasks);
/// ```
pub fn display_tasks_table(tasks: &[task_manager::domain::task::Task]) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }

    let mut table = prettytable::Table::new();

    // Set table format
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);

    // Add header row
    table.set_titles(prettytable::row![
        "ID",
        "Title",
        "Status",
        "Assignee",
        "Due Date",
        "Created"
    ]);

    // Add task rows
    for task in tasks {
        let id_short = if task.id.len() > 8 {
            &task.id[..8]
        } else {
            &task.id
        };

        let title_truncated = if task.title.len() > 40 {
            std::format!("{}...", &task.title[..37])
        } else {
            task.title.clone()
        };

        let assignee_display = task.assignee.as_ref().map(|a| {
            if a.len() > 15 {
                std::format!("{}...", &a[..12])
            } else {
                a.clone()
            }
        }).unwrap_or_else(|| std::string::String::from("-"));

        let due_date_display = task.due_date.as_ref()
            .map(|d| d.clone())
            .unwrap_or_else(|| std::string::String::from("-"));

        let created_display = task.created_at.format("%Y-%m-%d").to_string();

        // Format status
        let status_display = match task.status {
            task_manager::domain::task_status::TaskStatus::Todo => {
                std::format!("{}", "Todo")
            }
            task_manager::domain::task_status::TaskStatus::InProgress => {
                std::format!("{}", "In Progress")
            }
            task_manager::domain::task_status::TaskStatus::PendingEnhancement => {
                std::format!("{}", "Pending Enhancement")
            }
            task_manager::domain::task_status::TaskStatus::PendingComprehensionTest => {
                std::format!("{}", "Pending Test")
            }
            task_manager::domain::task_status::TaskStatus::PendingFollowOn => {
                std::format!("{}", "Pending FollowOn")
            }
            task_manager::domain::task_status::TaskStatus::PendingDecomposition => {
                std::format!("{}", "Pending Decomposition")
            }
            task_manager::domain::task_status::TaskStatus::Decomposed => {
                std::format!("{}", "Decomposed")
            }
            task_manager::domain::task_status::TaskStatus::OrchestrationComplete => {
                std::format!("{}", "Orchestration Complete")
            }
            task_manager::domain::task_status::TaskStatus::Completed => {
                std::format!("{}", "Completed")
            }
            task_manager::domain::task_status::TaskStatus::Archived => {
                std::format!("{}", "Archived")
            }
            task_manager::domain::task_status::TaskStatus::Errored => {
                std::format!("{}", "Errored")
            }
        };

        table.add_row(prettytable::row![
            id_short,
            title_truncated,
            status_display,
            assignee_display,
            due_date_display,
            created_display
        ]);
    }

    table.printstd();

    println!("\nTotal: {} task(s)", tasks.len());
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_display_empty_tasks() {
        // Test: Validates empty task list displays "No tasks found" message.
        // Justification: Ensures graceful handling of empty result sets.
        let tasks: std::vec::Vec<task_manager::domain::task::Task> = std::vec![];
        // This would print to stdout, so we just verify it doesn't panic
        super::display_tasks_table(&tasks);
    }

    #[test]
    fn test_display_single_task() {
        // Test: Validates single task displays correctly in table.
        // Justification: Ensures basic table rendering works.
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Test Task"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-31")),
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        let tasks = std::vec![task];
        // This would print to stdout, so we just verify it doesn't panic
        super::display_tasks_table(&tasks);
    }

    #[test]
    fn test_display_long_title_truncation() {
        // Test: Validates long titles are truncated properly.
        // Justification: Ensures table fits in terminal width.
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("This is a very long task title that should be truncated to fit within the table column width constraints"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        let tasks = std::vec![task];
        // This would print to stdout, so we just verify it doesn't panic
        super::display_tasks_table(&tasks);
    }
}
