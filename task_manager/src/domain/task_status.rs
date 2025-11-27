//! Defines the TaskStatus enum for task lifecycle states.
//!
//! TaskStatus represents the current state of a task in its lifecycle from
//! creation to completion or archival. This enum enables tracking task
//! progress and filtering tasks by their current status.
//!
//! Revision History
//! - 2025-11-24T18:00:00Z @AI: Add Errored variant for task failure tracking. Enables tracking tasks that encountered errors during execution or orchestration.
//! - 2025-11-23T15:30:00Z @AI: Add PendingDecomposition and Decomposed variants for Phase 2 Sprint 4 Task 2.2.
//! - 2025-11-12T20:28:00Z @AI: Add PendingEnhancement, PendingComprehensionTest, PendingFollowOn, OrchestrationComplete variants.
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
/// # use task_manager::domain::task_status::TaskStatus;
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

    /// Task is awaiting enhancement generation by the orchestrator.
    PendingEnhancement,

    /// Task is awaiting comprehension test generation.
    PendingComprehensionTest,

    /// Task is awaiting follow-on actions after testing.
    PendingFollowOn,

    /// Task is awaiting decomposition into subtasks (classified as high complexity).
    PendingDecomposition,

    /// Task has been decomposed into subtasks successfully.
    Decomposed,

    /// Orchestration has completed for this task.
    OrchestrationComplete,

    /// Task has been completed successfully.
    Completed,

    /// Task has been archived for long-term storage.
    Archived,

    /// Task encountered an error and failed.
    Errored,
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

    #[test]
    fn test_task_status_decomposition_states() {
        // Test: Validates new decomposition workflow states exist and behave correctly.
        // Justification: Ensures PendingDecomposition and Decomposed states work for Phase 2 intelligent routing.
        assert_eq!(TaskStatus::PendingDecomposition, TaskStatus::PendingDecomposition);
        assert_eq!(TaskStatus::Decomposed, TaskStatus::Decomposed);
        assert_ne!(TaskStatus::PendingDecomposition, TaskStatus::Decomposed);
        assert_ne!(TaskStatus::PendingDecomposition, TaskStatus::Todo);

        // Verify cloning works
        let pending = TaskStatus::PendingDecomposition;
        assert_eq!(pending.clone(), TaskStatus::PendingDecomposition);

        let decomposed = TaskStatus::Decomposed;
        assert_eq!(decomposed.clone(), TaskStatus::Decomposed);
    }
}
