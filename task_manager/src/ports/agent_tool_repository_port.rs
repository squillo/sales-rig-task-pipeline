//! Defines the AgentToolRepositoryPort output port for agent tool persistence.
//!
//! This port represents the interface for persisting and querying agent tools using
//! the HEXSER framework's Repository pattern. It extends the standard HEXSER
//! Repository and QueryRepository traits to provide type-safe persistence operations
//! for tool capabilities.
//!
//! Revision History
//! - 2025-11-26T07:15:00Z @AI: Initial AgentToolRepositoryPort trait definition for Phase 2 persona management.

/// Filter criteria for querying agent tools.
///
/// ToolFilter defines the available filter operations for agent tool queries.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe tool filtering.
#[derive(Debug, Clone)]
pub enum ToolFilter {
    /// Filter by unique tool ID.
    ById(String),

    /// Filter by tool category.
    ByCategory(crate::domain::agent_tool::ToolCategory),

    /// Filter by risk level.
    ByRiskLevel(crate::domain::agent_tool::RiskLevel),

    /// Filter to only default tools.
    DefaultOnly,

    /// Return all tools (no filtering).
    All,
}

/// Sort key options for agent tool queries.
///
/// ToolSortKey defines the available fields by which agent tools can be sorted.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe tool sorting.
#[derive(Debug, Clone)]
pub enum ToolSortKey {
    /// Sort by tool ID alphabetically.
    Id,

    /// Sort by tool name alphabetically.
    Name,

    /// Sort by category.
    Category,

    /// Sort by risk level (Safe → Moderate → High).
    RiskLevel,
}

/// Port (interface) for agent tool persistence and retrieval operations.
///
/// AgentToolRepositoryPort extends HEXSER's standard Repository and QueryRepository
/// traits to provide comprehensive agent tool storage capabilities. Any concrete
/// adapter implementing this trait gains access to standard CRUD operations
/// plus filtering and sorting capabilities.
///
/// # Examples
///
/// ```no_run
/// # use task_manager::ports::agent_tool_repository_port::{AgentToolRepositoryPort, ToolFilter, ToolSortKey};
/// # use task_manager::domain::agent_tool::AgentTool;
/// # use hexser::ports::Repository;
/// # use hexser::ports::repository::QueryRepository;
/// # fn example<R: AgentToolRepositoryPort>(repo: &mut R, tool: AgentTool) {
/// // Save a tool using HEXSER Repository trait (takes ownership)
/// repo.save(tool)?;
///
/// // Query default tools
/// let defaults = repo.find(&ToolFilter::DefaultOnly, hexser::ports::repository::FindOptions::default())?;
///
/// // Query by category
/// use task_manager::domain::agent_tool::ToolCategory;
/// let dev_tools = repo.find(&ToolFilter::ByCategory(ToolCategory::Development), hexser::ports::repository::FindOptions::default())?;
/// # std::result::Result::Ok::<(), String>(())
/// # }
/// ```
pub trait AgentToolRepositoryPort:
    hexser::ports::Repository<crate::domain::agent_tool::AgentTool>
    + hexser::ports::repository::QueryRepository<
        crate::domain::agent_tool::AgentTool,
        Filter = ToolFilter,
        SortKey = ToolSortKey,
    >
    + Send
    + Sync
{
    // Marker trait - all methods provided by HEXSER Repository traits
}
