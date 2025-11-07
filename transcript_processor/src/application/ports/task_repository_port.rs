//! Defines the TaskRepositoryPort output port for task persistence.
//!
//! This port represents the interface for persisting and querying tasks using
//! the HEXSER framework's Repository pattern. It extends the standard HEXSER
//! Repository and QueryRepository traits to provide type-safe persistence operations.
//!
//! Revision History
//! - 2025-11-06T18:14:00Z @AI: Refactor to use HEXSER Repository pattern with filters and sort keys.
//! - 2025-11-06T17:41:00Z @AI: Initial TaskRepositoryPort trait definition.

/// Filter criteria for querying tasks.
///
/// TaskFilter defines the available filter operations for task queries.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe task filtering.
#[derive(Debug, Clone)]
pub enum TaskFilter {
    /// Filter by unique task ID.
    ById(String),

    /// Filter by task status.
    ByStatus(crate::domain::task_status::TaskStatus),

    /// Filter by assignee name.
    ByAssignee(String),

    /// Return all tasks (no filtering).
    All,
}

/// Sort key options for task queries.
///
/// TaskSortKey defines the available fields by which tasks can be sorted.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe task sorting.
#[derive(Debug, Clone)]
pub enum TaskSortKey {
    /// Sort by creation timestamp.
    CreatedAt,

    /// Sort by last update timestamp.
    UpdatedAt,

    /// Sort by task status.
    Status,

    /// Sort by task title alphabetically.
    Title,

    /// Sort by due date (if present).
    DueDate,
}

/// Port (interface) for task persistence and retrieval operations.
///
/// TaskRepositoryPort extends HEXSER's standard Repository and QueryRepository
/// traits to provide comprehensive task storage capabilities. Any concrete
/// adapter implementing this trait gains access to standard CRUD operations
/// plus filtering and sorting capabilities.
///
/// # Examples
///
/// ```no_run
/// # use transcript_processor::application::ports::task_repository_port::{TaskRepositoryPort, TaskFilter, TaskSortKey};
/// # use transcript_processor::domain::task::Task;
/// # use hexser::ports::Repository;
/// # use hexser::ports::repository::QueryRepository;
/// # fn example<R: TaskRepositoryPort>(repo: &mut R, task: Task) {
/// // Save a task using HEXSER Repository trait (takes ownership)
/// repo.save(task).unwrap();
///
/// // Query tasks using HEXSER find() with filters and options
/// let filtered = repo.find(&TaskFilter::All, hexser::ports::repository::FindOptions::default()).unwrap();
/// # }
/// ```
pub trait TaskRepositoryPort:
    hexser::ports::Repository<crate::domain::task::Task>
    + hexser::ports::repository::QueryRepository<
        crate::domain::task::Task,
        Filter = TaskFilter,
        SortKey = TaskSortKey,
    >
    + Send
    + Sync
{
    // Marker trait - all methods provided by HEXSER Repository traits
}
