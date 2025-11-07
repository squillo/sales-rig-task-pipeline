//! Defines the Task domain entity for trackable work items.
//!
//! Task represents a persisted, trackable work item derived from an ActionItem.
//! It includes full lifecycle management with timestamps, status tracking, and
//! links back to the source transcript for traceability.
//!
//! Revision History
//! - 2025-11-06T18:14:00Z @AI: Add HexEntity derive for HEXSER framework alignment.
//! - 2025-11-06T17:41:00Z @AI: Initial Task struct definition with from_action_item constructor.

/// Represents a trackable task with full lifecycle management.
///
/// A Task is the persistent, mutable entity that tracks work items through
/// their lifecycle. Unlike ActionItem (which is a DTO), Task includes unique
/// identifiers, timestamps, status, and links to its data sources.
///
/// # Fields
///
/// * `id` - Unique identifier (UUID) for this task.
/// * `title` - The task's title or description.
/// * `assignee` - Optional person responsible for completing the task.
/// * `due_date` - Optional deadline in string format.
/// * `status` - Current lifecycle status of the task.
/// * `source_transcript_id` - Optional link to the originating transcript.
/// * `created_at` - UTC timestamp when task was created.
/// * `updated_at` - UTC timestamp of last modification.
///
/// # Examples
///
/// ```
/// # use transcript_processor::domain::task::Task;
/// # use transcript_processor::domain::action_item::ActionItem;
/// let action = ActionItem {
///     title: std::string::String::from("Review code"),
///     assignee: Some(std::string::String::from("Alice")),
///     due_date: None,
/// };
///
/// let task = Task::from_action_item(&action, Some(std::string::String::from("transcript-123")));
/// assert_eq!(task.title, "Review code");
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, hexser::HexEntity)]
pub struct Task {
    /// Unique identifier for this task (UUID v4).
    pub id: String,

    /// The title or description of the task.
    pub title: String,

    /// The person assigned to complete this task.
    pub assignee: Option<String>,

    /// The deadline for this task in string format.
    pub due_date: Option<String>,

    /// The current status of this task in its lifecycle.
    pub status: crate::domain::task_status::TaskStatus,

    /// Optional link to the source transcript this task was extracted from.
    pub source_transcript_id: Option<String>,

    /// UTC timestamp when this task was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// UTC timestamp of the last modification to this task.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Task {
    /// Creates a new Task from an ActionItem.
    ///
    /// This constructor generates a unique UUID for the task, sets its initial
    /// status to Todo, and initializes both timestamps to the current UTC time.
    ///
    /// # Arguments
    ///
    /// * `action` - The ActionItem to convert into a Task.
    /// * `transcript_id` - Optional ID of the source transcript.
    ///
    /// # Returns
    ///
    /// A new Task instance with generated ID and timestamps.
    ///
    /// # Examples
    ///
    /// ```
    /// # use transcript_processor::domain::task::Task;
    /// # use transcript_processor::domain::action_item::ActionItem;
    /// let action = ActionItem {
    ///     title: std::string::String::from("Test task"),
    ///     assignee: None,
    ///     due_date: None,
    /// };
    ///
    /// let task = Task::from_action_item(&action, None);
    /// assert!(!task.id.is_empty());
    /// assert_eq!(task.title, "Test task");
    /// ```
    pub fn from_action_item(
        action: &crate::domain::action_item::ActionItem,
        transcript_id: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now();

        Task {
            id: uuid::Uuid::new_v4().to_string(),
            title: action.title.clone(),
            assignee: action.assignee.clone(),
            due_date: action.due_date.clone(),
            status: crate::domain::task_status::TaskStatus::Todo,
            source_transcript_id: transcript_id,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_from_action_item() {
        // Test: Validates that a Task is correctly created from an ActionItem with all fields populated.
        // Justification: Ensures the primary constructor correctly transfers all data from ActionItem
        // to Task and properly initializes UUID, status, and timestamps.
        let action = crate::domain::action_item::ActionItem {
            title: std::string::String::from("Test action"),
            assignee: Some(std::string::String::from("Bob")),
            due_date: Some(std::string::String::from("2025-11-30")),
        };

        let task = Task::from_action_item(&action, Some(std::string::String::from("transcript-456")));

        assert!(!task.id.is_empty());
        assert_eq!(task.title, "Test action");
        assert_eq!(task.assignee, Some(std::string::String::from("Bob")));
        assert_eq!(task.due_date, Some(std::string::String::from("2025-11-30")));
        assert_eq!(task.status, crate::domain::task_status::TaskStatus::Todo);
        assert_eq!(task.source_transcript_id, Some(std::string::String::from("transcript-456")));
        assert_eq!(task.created_at, task.updated_at);
    }

    #[test]
    fn test_task_from_action_item_minimal() {
        // Test: Validates that a Task can be created with only required fields (minimal ActionItem).
        // Justification: Ensures the constructor handles optional fields correctly when they are None,
        // which is a critical edge case for data coming from LLM extraction where fields may be missing.
        let action = crate::domain::action_item::ActionItem {
            title: std::string::String::from("Minimal task"),
            assignee: None,
            due_date: None,
        };

        let task = Task::from_action_item(&action, None);

        assert!(!task.id.is_empty());
        assert_eq!(task.title, "Minimal task");
        assert!(task.assignee.is_none());
        assert!(task.due_date.is_none());
        assert!(task.source_transcript_id.is_none());
    }

    #[test]
    fn test_task_uuid_uniqueness() {
        // Test: Validates that each Task generated from the same ActionItem receives a unique UUID.
        // Justification: Ensures UUID generation works correctly and prevents potential ID collision bugs
        // that could corrupt task storage or tracking. This is critical for data integrity.
        let action = crate::domain::action_item::ActionItem {
            title: std::string::String::from("UUID test"),
            assignee: None,
            due_date: None,
        };

        let task1 = Task::from_action_item(&action, None);
        let task2 = Task::from_action_item(&action, None);

        assert_ne!(task1.id, task2.id);
    }
}
