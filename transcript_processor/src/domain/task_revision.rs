//! Defines the TaskRevision domain entity for task history tracking.
//!
//! TaskRevision captures a snapshot of a task's state at a specific point in time,
//! enabling full audit trails and historical analysis of task changes. Each revision
//! links to its parent task and stores the previous state as JSON.
//!
//! Revision History
//! - 2025-11-06T18:14:00Z @AI: Add HexEntity derive for HEXSER framework alignment.
//! - 2025-11-06T17:41:00Z @AI: Initial TaskRevision struct definition.

/// Represents a single revision in a task's history.
///
/// A TaskRevision records a change to a task, capturing what changed, when it
/// changed, and optionally what the state was before the change. This enables
/// complete auditability and the ability to reconstruct task history.
///
/// # Fields
///
/// * `revision_id` - Unique identifier for this revision (UUID v4).
/// * `task_id` - The ID of the task this revision belongs to.
/// * `timestamp` - UTC timestamp when this revision was created.
/// * `change_description` - Human-readable description of what changed.
/// * `previous_state_json` - Optional JSON serialization of the task before the change.
///
/// # Examples
///
/// ```
/// # use transcript_processor::domain::task_revision::TaskRevision;
/// let revision = TaskRevision {
///     revision_id: uuid::Uuid::new_v4().to_string(),
///     task_id: std::string::String::from("task-123"),
///     timestamp: chrono::Utc::now(),
///     change_description: std::string::String::from("Task created"),
///     previous_state_json: None,
/// };
///
/// assert_eq!(revision.task_id, "task-123");
/// assert_eq!(revision.change_description, "Task created");
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, hexser::HexEntity)]
pub struct TaskRevision {
    /// Unique identifier for this revision (UUID v4).
    pub revision_id: String,

    /// The ID of the task this revision belongs to.
    pub task_id: String,

    /// UTC timestamp when this revision was created.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Human-readable description of what changed.
    pub change_description: String,

    /// Optional JSON serialization of the task state before this change.
    /// None for initial creation revisions.
    pub previous_state_json: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_revision_creation() {
        // Test: Validates that a TaskRevision can be created with full data including previous state.
        // Justification: Ensures the revision entity correctly captures all audit trail data including
        // the JSON snapshot of the previous task state, which is critical for history tracking.
        let now = chrono::Utc::now();
        let revision = TaskRevision {
            revision_id: uuid::Uuid::new_v4().to_string(),
            task_id: std::string::String::from("task-456"),
            timestamp: now,
            change_description: std::string::String::from("Status updated to InProgress"),
            previous_state_json: Some(std::string::String::from(r#"{"status":"Todo"}"#)),
        };

        assert!(!revision.revision_id.is_empty());
        assert_eq!(revision.task_id, "task-456");
        assert_eq!(revision.timestamp, now);
        assert_eq!(revision.change_description, "Status updated to InProgress");
        assert!(revision.previous_state_json.is_some());
    }

    #[test]
    fn test_task_revision_initial_creation() {
        // Test: Validates that a TaskRevision can be created without previous state for initial creation.
        // Justification: Ensures the revision entity handles the special case of task creation where
        // there is no previous state to snapshot, verifying None is correctly used for initial revisions.
        let revision = TaskRevision {
            revision_id: uuid::Uuid::new_v4().to_string(),
            task_id: std::string::String::from("task-789"),
            timestamp: chrono::Utc::now(),
            change_description: std::string::String::from("Task created"),
            previous_state_json: None,
        };

        assert_eq!(revision.change_description, "Task created");
        assert!(revision.previous_state_json.is_none());
    }

    #[test]
    fn test_task_revision_clone() {
        // Test: Validates that TaskRevision implements Clone correctly and creates a deep copy.
        // Justification: Ensures revision history can be safely copied for queries and reporting
        // without mutating the original audit trail, which is critical for data integrity.
        let revision = TaskRevision {
            revision_id: uuid::Uuid::new_v4().to_string(),
            task_id: std::string::String::from("task-clone"),
            timestamp: chrono::Utc::now(),
            change_description: std::string::String::from("Test change"),
            previous_state_json: None,
        };

        let cloned = revision.clone();
        assert_eq!(revision.revision_id, cloned.revision_id);
        assert_eq!(revision.task_id, cloned.task_id);
        assert_eq!(revision.change_description, cloned.change_description);
    }
}
