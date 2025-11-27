//! Defines the ProjectRepositoryPort output port for project persistence.
//!
//! This port represents the interface for persisting and querying projects using
//! the HEXSER framework's Repository pattern. It extends the standard HEXSER
//! Repository and QueryRepository traits to provide type-safe persistence operations.
//!
//! Revision History
//! - 2025-11-24T05:00:00Z @AI: Initial ProjectRepositoryPort trait definition for Phase 1 TUI project architecture.

/// Filter criteria for querying projects.
///
/// ProjectFilter defines the available filter operations for project queries.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe project filtering.
#[derive(Debug, Clone)]
pub enum ProjectFilter {
    /// Filter by unique project ID.
    ById(String),

    /// Filter by project name (exact match).
    ByName(String),

    /// Return all projects (no filtering).
    All,
}

/// Sort key options for project queries.
///
/// ProjectSortKey defines the available fields by which projects can be sorted.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe project sorting.
#[derive(Debug, Clone)]
pub enum ProjectSortKey {
    /// Sort by creation timestamp.
    CreatedAt,

    /// Sort by project name alphabetically.
    Name,
}

/// Port (interface) for project persistence and retrieval operations.
///
/// ProjectRepositoryPort extends HEXSER's standard Repository and QueryRepository
/// traits to provide comprehensive project storage capabilities. Any concrete
/// adapter implementing this trait gains access to standard CRUD operations
/// plus filtering and sorting capabilities.
///
/// # Examples
///
/// ```no_run
/// # use task_manager::ports::project_repository_port::{ProjectRepositoryPort, ProjectFilter, ProjectSortKey};
/// # use task_manager::domain::project::Project;
/// # use hexser::ports::Repository;
/// # use hexser::ports::repository::QueryRepository;
/// # fn example<R: ProjectRepositoryPort>(repo: &mut R, project: Project) {
/// // Save a project using HEXSER Repository trait (takes ownership)
/// repo.save(project).unwrap();
///
/// // Query projects using HEXSER find() with filters and options
/// let all_projects = repo.find(&ProjectFilter::All, hexser::ports::repository::FindOptions::default()).unwrap();
/// # }
/// ```
pub trait ProjectRepositoryPort:
    hexser::ports::Repository<crate::domain::project::Project>
    + hexser::ports::repository::QueryRepository<
        crate::domain::project::Project,
        Filter = ProjectFilter,
        SortKey = ProjectSortKey,
    >
    + Send
    + Sync
{
    // Marker trait - all methods provided by HEXSER Repository traits
}
