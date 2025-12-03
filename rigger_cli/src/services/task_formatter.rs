//! Task formatting service for copy/paste operations.
//!
//! Provides functions to format task data into human-readable and
//! machine-parseable formats (Markdown, plain text, etc.) for clipboard
//! operations and sharing.
//!
//! Revision History
//! - 2025-11-24T18:00:00Z @AI: Add Errored status formatting support.
//! - 2025-11-24T00:30:00Z @AI: Create task formatter service with comprehensive tests.

/// Formats a task as Markdown for sharing.
///
/// Creates a well-formatted Markdown representation of a task including
/// ID, title, reasoning, status, assignee, and complexity.
///
/// # Arguments
///
/// * `task` - The task to format
///
/// # Returns
///
/// A Markdown-formatted string representation of the task
///
/// # Examples
///
/// ```
/// use task_manager::domain::task::Task;
/// use task_manager::domain::task_status::TaskStatus;
/// use rigger_cli::services::task_formatter::format_task_as_markdown;
///
/// let task = Task {
///     id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
///     title: "Implement clipboard".to_string(),
///     assignee: Some("Alice".to_string()),
///     due_date: None,
///     status: TaskStatus::InProgress,
///     source_transcript_id: None,
///     source_prd_id: None,
///     parent_task_id: None,
///     subtask_ids: vec![],
///     created_at: chrono::Utc::now(),
///     updated_at: chrono::Utc::now(),
///     enhancements: None,
///     comprehension_tests: None,
///     complexity: Some(3),
///     reasoning: Some("Add copy/paste support to TUI".to_string()),
///     context_files: vec![],
///     dependencies: vec![],
/// };
///
/// let markdown = format_task_as_markdown(&task);
/// assert!(markdown.contains("# Implement clipboard"));
/// assert!(markdown.contains("**Status:** IN PROGRESS"));
/// ```
pub fn format_task_as_markdown(task: &task_manager::domain::task::Task) -> String {
    let mut output = String::new();

    // Title
    output.push_str(&std::format!("# {}\n\n", task.title));

    // Metadata
    output.push_str(&std::format!("**ID:** `{}`\n", task.id));
    output.push_str(&std::format!("**Status:** {}\n", format_status(&task.status)));

    if let std::option::Option::Some(ref assignee) = task.agent_persona {
        output.push_str(&std::format!("**Assignee:** {}\n", assignee));
    }

    if let std::option::Option::Some(complexity) = task.complexity {
        output.push_str(&std::format!("**Complexity:** {}/10\n", complexity));
    }

    if let std::option::Option::Some(ref due_date) = task.due_date {
        output.push_str(&std::format!("**Due Date:** {}\n", due_date));
    }

    // Reasoning/Context
    if let std::option::Option::Some(ref reasoning) = task.reasoning {
        output.push_str("\n## Reasoning\n\n");
        output.push_str(reasoning);
        output.push('\n');
    }

    // Dependencies
    if !task.dependencies.is_empty() {
        output.push_str("\n## Dependencies\n\n");
        for dep in &task.dependencies {
            output.push_str(&std::format!("- {}\n", dep));
        }
    }

    output
}

/// Formats a task as plain text for sharing.
///
/// Creates a simple plain text representation suitable for pasting
/// into emails, chat messages, or other plain text contexts.
///
/// # Arguments
///
/// * `task` - The task to format
///
/// # Returns
///
/// A plain text formatted string representation of the task
pub fn format_task_as_plain_text(task: &task_manager::domain::task::Task) -> String {
    let mut output = String::new();

    // Title
    output.push_str(&std::format!("Task: {}\n", task.title));

    // Metadata
    output.push_str(&std::format!("ID: {}\n", task.id));
    output.push_str(&std::format!("Status: {}\n", format_status(&task.status)));

    if let std::option::Option::Some(ref assignee) = task.agent_persona {
        output.push_str(&std::format!("Assignee: {}\n", assignee));
    }

    if let std::option::Option::Some(complexity) = task.complexity {
        output.push_str(&std::format!("Complexity: {}/10\n", complexity));
    }

    // Reasoning
    if let std::option::Option::Some(ref reasoning) = task.reasoning {
        output.push_str(&std::format!("\nReasoning:\n{}\n", reasoning));
    }

    output
}

/// Formats task status as a human-readable string.
fn format_status(status: &task_manager::domain::task_status::TaskStatus) -> String {
    match status {
        task_manager::domain::task_status::TaskStatus::Todo => "TODO".to_string(),
        task_manager::domain::task_status::TaskStatus::InProgress => "IN PROGRESS".to_string(),
        task_manager::domain::task_status::TaskStatus::Completed => "COMPLETED".to_string(),
        task_manager::domain::task_status::TaskStatus::Archived => "ARCHIVED".to_string(),
        task_manager::domain::task_status::TaskStatus::PendingEnhancement => "PENDING ENHANCEMENT".to_string(),
        task_manager::domain::task_status::TaskStatus::PendingComprehensionTest => "PENDING TEST".to_string(),
        task_manager::domain::task_status::TaskStatus::PendingFollowOn => "PENDING FOLLOW-ON".to_string(),
        task_manager::domain::task_status::TaskStatus::PendingDecomposition => "PENDING DECOMPOSITION".to_string(),
        task_manager::domain::task_status::TaskStatus::Decomposed => "DECOMPOSED".to_string(),
        task_manager::domain::task_status::TaskStatus::OrchestrationComplete => "ORCHESTRATION COMPLETE".to_string(),
        task_manager::domain::task_status::TaskStatus::Errored => "ERRORED".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task() -> task_manager::domain::task::Task {
        task_manager::domain::task::Task {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "Implement clipboard support".to_string(),
            description: "Add copy/paste functionality to the TUI".to_string(),
            agent_persona: std::option::Option::Some("Backend Developer".to_string()),
            due_date: std::option::Option::None,
            status: task_manager::domain::task_status::TaskStatus::InProgress,
            source_transcript_id: std::option::Option::None,
            source_prd_id: std::option::Option::None,
            parent_task_id: std::option::Option::None,
            subtask_ids: std::vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            enhancements: std::option::Option::None,
            comprehension_tests: std::option::Option::None,
            complexity: std::option::Option::Some(5),
            reasoning: std::option::Option::Some("Add copy/paste functionality to the TUI".to_string()),
            completion_summary: std::option::Option::None,
            context_files: std::vec![],
            dependencies: std::vec!["task-123".to_string()],
            sort_order: std::option::Option::Some(0),
        }
    }

    #[test]
    fn test_format_task_as_markdown_includes_title() {
        // Test: Validates that Markdown output includes task title as H1.
        // Justification: Title is the most important identifier for users.
        let task = create_test_task();
        let markdown = format_task_as_markdown(&task);

        std::assert!(markdown.contains("# Implement clipboard support"));
    }

    #[test]
    fn test_format_task_as_markdown_includes_id() {
        // Test: Validates that Markdown output includes task ID in code format.
        // Justification: ID is needed for cross-referencing and automation.
        let task = create_test_task();
        let markdown = format_task_as_markdown(&task);

        std::assert!(markdown.contains("**ID:** `550e8400-e29b-41d4-a716-446655440000`"));
    }

    #[test]
    fn test_format_task_as_markdown_includes_status() {
        // Test: Validates that status is formatted correctly.
        // Justification: Status indicates current state of work.
        let task = create_test_task();
        let markdown = format_task_as_markdown(&task);

        std::assert!(markdown.contains("**Status:** IN PROGRESS"));
    }

    #[test]
    fn test_format_task_as_markdown_includes_assignee() {
        // Test: Validates that assignee is included when present.
        // Justification: Assignee helps identify ownership.
        let task = create_test_task();
        let markdown = format_task_as_markdown(&task);

        std::assert!(markdown.contains("**Assignee:** Backend Developer"));
    }

    #[test]
    fn test_format_task_as_markdown_includes_complexity() {
        // Test: Validates that complexity is formatted with scale indicator.
        // Justification: Complexity helps with effort estimation.
        let task = create_test_task();
        let markdown = format_task_as_markdown(&task);

        std::assert!(markdown.contains("**Complexity:** 5/10"));
    }

    #[test]
    fn test_format_task_as_markdown_includes_reasoning() {
        // Test: Validates that reasoning is included in dedicated section.
        // Justification: Reasoning provides context for the task.
        let task = create_test_task();
        let markdown = format_task_as_markdown(&task);

        std::assert!(markdown.contains("## Reasoning"));
        std::assert!(markdown.contains("Add copy/paste functionality to the TUI"));
    }

    #[test]
    fn test_format_task_as_markdown_includes_dependencies() {
        // Test: Validates that dependencies are formatted as bullet list.
        // Justification: Dependencies enable dependency tracking.
        let task = create_test_task();
        let markdown = format_task_as_markdown(&task);

        std::assert!(markdown.contains("## Dependencies"));
        std::assert!(markdown.contains("- task-123"));
    }

    #[test]
    fn test_format_task_as_markdown_handles_minimal_task() {
        // Test: Validates formatting of task with only required fields.
        // Justification: Not all tasks have optional metadata.
        let task = task_manager::domain::task::Task {
            id: "minimal-123".to_string(),
            title: "Minimal task".to_string(),
            description: String::new(),
            agent_persona: std::option::Option::None,
            due_date: std::option::Option::None,
            status: task_manager::domain::task_status::TaskStatus::Todo,
            source_transcript_id: std::option::Option::None,
            source_prd_id: std::option::Option::None,
            parent_task_id: std::option::Option::None,
            subtask_ids: std::vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            enhancements: std::option::Option::None,
            comprehension_tests: std::option::Option::None,
            complexity: std::option::Option::None,
            reasoning: std::option::Option::None,
            context_files: std::vec![],
            dependencies: std::vec![],
            completion_summary: std::option::Option::None,
            sort_order: std::option::Option::Some(0),
        };

        let markdown = format_task_as_markdown(&task);

        std::assert!(markdown.contains("# Minimal task"));
        std::assert!(markdown.contains("**Status:** TODO"));
        std::assert!(!markdown.contains("Assignee"));
        std::assert!(!markdown.contains("Complexity"));
        std::assert!(!markdown.contains("Reasoning"));
    }

    #[test]
    fn test_format_task_as_plain_text_basic_structure() {
        // Test: Validates plain text format has correct structure.
        // Justification: Plain text must be readable without Markdown rendering.
        let task = create_test_task();
        let text = format_task_as_plain_text(&task);

        std::assert!(text.contains("Task: Implement clipboard support"));
        std::assert!(text.contains("ID: 550e8400-e29b-41d4-a716-446655440000"));
        std::assert!(text.contains("Status: IN PROGRESS"));
    }

    #[test]
    fn test_format_task_as_plain_text_includes_reasoning() {
        // Test: Validates reasoning is included with label.
        // Justification: Reasoning provides essential context.
        let task = create_test_task();
        let text = format_task_as_plain_text(&task);

        std::assert!(text.contains("Reasoning:"));
        std::assert!(text.contains("Add copy/paste functionality to the TUI"));
    }

    #[test]
    fn test_format_status_basic_variants() {
        // Test: Validates basic status variants are formatted correctly.
        // Justification: Ensures consistent status representation.
        std::assert_eq!(
            format_status(&task_manager::domain::task_status::TaskStatus::Todo),
            "TODO"
        );
        std::assert_eq!(
            format_status(&task_manager::domain::task_status::TaskStatus::InProgress),
            "IN PROGRESS"
        );
        std::assert_eq!(
            format_status(&task_manager::domain::task_status::TaskStatus::Completed),
            "COMPLETED"
        );
        std::assert_eq!(
            format_status(&task_manager::domain::task_status::TaskStatus::Archived),
            "ARCHIVED"
        );
    }
}
