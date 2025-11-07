//! Defines the TaskStatus enum for task lifecycle states.
//!
//! TaskStatus represents the current state of a task in its lifecycle from
//! creation to completion or archival. This enum enables tracking task
//! progress and filtering tasks by their current status.
//!
//! Revision History
//! - 2025-11-06T17:41:00Z @AI: Initial TaskStatus enum definition.

/// Represents the current status of a task in its lifecycle.
///
/// TaskStatus defines the discrete states a task can occupy during its
/// lifetime. Tasks typically progress from Todo through InProgress to
/// Completed, but can also be Archived for long-term storage.
///
/// # Variants
///
/// * `Todo` - Task has been created but work has not started.
/// * `InProgress` - Work on the task is currently underway.
/// * `Completed` - Task has been finished successfully.
/// * `Archived` - Task has been moved to archive (completed or cancelled).
///
/// # Examples
///
/// ```
/// # use transcript_processor::domain::task_status::TaskStatus;
/// let status = TaskStatus::Todo;
/// assert_eq!(status, TaskStatus::Todo);
///
/// let active = TaskStatus::InProgress;
/// assert_ne!(active, TaskStatus::Completed);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TaskStatus {
    /// Task has been created but work has not yet started.
    Todo,

    /// Work on the task is currently in progress.
    InProgress,

    /// Task has been completed successfully.
    Completed,

    /// Task has been archived for long-term storage.
    Archived,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_equality() {
        assert_eq!(TaskStatus::Todo, TaskStatus::Todo);
        assert_eq!(TaskStatus::InProgress, TaskStatus::InProgress);
        assert_eq!(TaskStatus::Completed, TaskStatus::Completed);
        assert_eq!(TaskStatus::Archived, TaskStatus::Archived);
    }

    #[test]
    fn test_task_status_inequality() {
        assert_ne!(TaskStatus::Todo, TaskStatus::InProgress);
        assert_ne!(TaskStatus::InProgress, TaskStatus::Completed);
        assert_ne!(TaskStatus::Completed, TaskStatus::Archived);
    }

    #[test]
    fn test_task_status_clone() {
        let status = TaskStatus::Todo;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }
}
