//! Defines the Task domain entity for trackable work items.
//!
//! Task represents a persisted, trackable work item derived from an ActionItem.
//! It includes full lifecycle management with timestamps, status tracking, and
//! links back to the source transcript for traceability.
//!
//! Revision History
//! - 2025-11-30T21:30:00Z @AI: Add sort_order field for manual task prioritization within TODO column. Lower values appear first, None values sort by created_at.
//! - 2025-11-29T15:00:00Z @AI: Rename assignee to agent_persona for better LLM inference. Field name "assignee" caused LLMs to default to placeholder human names (Alice, Bob, Charlie). New name primes LLM to produce role-based outputs (Backend Architect, Security Analyst, etc.).
//! - 2025-11-26T09:45:00Z @AI: Add completion_summary field to store LLM's summary when task is completed.
//! - 2025-11-24T08:45:00Z @AI: Add description field for detailed task information (Phase 8 prerequisite).
//! - 2025-11-23T15:15:00Z @AI: Add intelligence fields (complexity, reasoning, context_files, dependencies) for Phase 2 Sprint 4.
//! - 2025-11-22T16:10:00Z @AI: Add Rigger fields (source_prd_id, parent_task_id, subtask_ids) for Phase 0.
//! - 2025-11-12T20:28:00Z @AI: Add enhancements and comprehension_tests fields for orchestration Phase 1.
//! - 2025-11-06T19:01:00Z @AI: Moved to task_manager crate, updated from_action_item to use transcript_extractor::ActionItem.
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
/// * `title` - The task's title or short summary.
/// * `description` - Detailed description of the task.
/// * `agent_persona` - Optional agent persona/role responsible for completing the task.
/// * `due_date` - Optional deadline in string format.
/// * `status` - Current lifecycle status of the task.
/// * `source_transcript_id` - Optional link to the originating transcript.
/// * `source_prd_id` - Optional link to the PRD that generated this task.
/// * `parent_task_id` - Optional parent task ID for subtask hierarchies.
/// * `subtask_ids` - List of subtask IDs if this task was decomposed.
/// * `created_at` - UTC timestamp when task was created.
/// * `updated_at` - UTC timestamp of last modification.
/// * `complexity` - Optional complexity score (1-10 scale, higher = more complex).
/// * `reasoning` - Optional LLM's chain-of-thought explanation for enhancements.
/// * `completion_summary` - Optional LLM-generated summary of what was done when completing the task.
/// * `context_files` - List of relevant codebase files for context engineering.
/// * `dependencies` - List of task IDs this task depends on.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::task::Task;
/// # use transcript_extractor::domain::action_item::ActionItem;
/// let action = ActionItem {
///     title: std::string::String::from("Review code"),
///     assignee: Some(std::string::String::from("Backend Developer")),
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

    /// The title or short summary of the task.
    pub title: String,

    /// Detailed description of the task.
    pub description: String,

    /// The agent persona/role assigned to complete this task.
    pub agent_persona: Option<String>,

    /// The deadline for this task in string format.
    pub due_date: Option<String>,

    /// The current status of this task in its lifecycle.
    pub status: crate::domain::task_status::TaskStatus,

    /// Optional link to the source transcript this task was extracted from.
    pub source_transcript_id: Option<String>,

    /// Optional link to the PRD that generated this task.
    pub source_prd_id: Option<String>,

    /// Optional parent task ID for subtask hierarchies.
    pub parent_task_id: Option<String>,

    /// List of subtask IDs if this task was decomposed.
    pub subtask_ids: std::vec::Vec<String>,

    /// UTC timestamp when this task was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// UTC timestamp of the last modification to this task.
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Optional list of enhancements generated for this task during orchestration.
    pub enhancements: Option<std::vec::Vec<crate::domain::enhancement::Enhancement>>,

    /// Optional list of comprehension tests associated with this task.
    pub comprehension_tests: Option<std::vec::Vec<crate::domain::comprehension_test::ComprehensionTest>>,

    /// Optional complexity score (1-10 scale, higher = more complex).
    pub complexity: Option<u8>,

    /// Optional LLM's chain-of-thought explanation for enhancements or reasoning.
    pub reasoning: Option<String>,

    /// Optional LLM-generated summary of what was done when completing the task.
    pub completion_summary: Option<String>,

    /// List of relevant codebase files for context engineering.
    pub context_files: std::vec::Vec<String>,

    /// List of task IDs this task depends on.
    pub dependencies: std::vec::Vec<String>,

    /// Optional sort order for manual prioritization within TODO column.
    /// Lower values appear first. Tasks without sort_order use created_at for ordering.
    pub sort_order: std::option::Option<i32>,
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
    /// # use task_manager::domain::task::Task;
    /// # use transcript_extractor::domain::action_item::ActionItem;
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
        action: &transcript_extractor::domain::action_item::ActionItem,
        transcript_id: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now();

        Task {
            id: uuid::Uuid::new_v4().to_string(),
            title: action.title.clone(),
            description: String::new(),
            agent_persona: action.assignee.clone(),
            due_date: action.due_date.clone(),
            status: crate::domain::task_status::TaskStatus::Todo,
            source_transcript_id: transcript_id,
            source_prd_id: std::option::Option::None,
            parent_task_id: std::option::Option::None,
            subtask_ids: std::vec::Vec::new(),
            created_at: now,
            updated_at: now,
            enhancements: std::option::Option::None,
            comprehension_tests: std::option::Option::None,
            complexity: std::option::Option::None,
            reasoning: std::option::Option::None,
            completion_summary: std::option::Option::None,
            context_files: std::vec::Vec::new(),
            dependencies: std::vec::Vec::new(),
            sort_order: std::option::Option::None,
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
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Test action"),
            assignee: Some(std::string::String::from("Bob")),
            due_date: Some(std::string::String::from("2025-11-30")),
        };

        let task = Task::from_action_item(&action, Some(std::string::String::from("transcript-456")));

        assert!(!task.id.is_empty());
        assert_eq!(task.title, "Test action");
        assert_eq!(task.agent_persona, Some(std::string::String::from("Bob")));
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
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Minimal task"),
            assignee: None,
            due_date: None,
        };

        let task = Task::from_action_item(&action, None);

        assert!(!task.id.is_empty());
        assert_eq!(task.title, "Minimal task");
        assert!(task.agent_persona.is_none());
        assert!(task.due_date.is_none());
        assert!(task.source_transcript_id.is_none());
    }

    #[test]
    fn test_task_uuid_uniqueness() {
        // Test: Validates that each Task generated from the same ActionItem receives a unique UUID.
        // Justification: Ensures UUID generation works correctly and prevents potential ID collision bugs
        // that could corrupt task storage or tracking. This is critical for data integrity.
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("UUID test"),
            assignee: None,
            due_date: None,
        };

        let task1 = Task::from_action_item(&action, None);
        let task2 = Task::from_action_item(&action, None);

        assert_ne!(task1.id, task2.id);
    }
}
