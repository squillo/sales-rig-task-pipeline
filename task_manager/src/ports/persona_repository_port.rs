//! Defines the PersonaRepositoryPort output port for persona persistence.
//!
//! This port represents the interface for persisting and querying personas using
//! the HEXSER framework's Repository pattern. It extends the standard HEXSER
//! Repository and QueryRepository traits with persona-specific operations like
//! default persona management and tool enablement tracking.
//!
//! Revision History
//! - 2025-11-26T09:15:00Z @AI: Add ByProject filter variant for project-scoped persona queries.
//! - 2025-11-26T07:20:00Z @AI: Initial PersonaRepositoryPort trait definition for Phase 2 persona management.

/// Filter criteria for querying personas.
///
/// PersonaFilter defines the available filter operations for persona queries.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe persona filtering.
#[derive(Debug, Clone)]
pub enum PersonaFilter {
    /// Filter by unique persona ID.
    ById(String),

    /// Filter by persona name (exact match).
    ByName(String),

    /// Filter by project ID (returns all personas scoped to this project).
    ByProject(String),

    /// Filter to only the default persona.
    DefaultOnly,

    /// Return all personas (no filtering).
    All,
}

/// Sort key options for persona queries.
///
/// PersonaSortKey defines the available fields by which personas can be sorted.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe persona sorting.
#[derive(Debug, Clone)]
pub enum PersonaSortKey {
    /// Sort by persona name alphabetically.
    Name,

    /// Sort by role alphabetically.
    Role,

    /// Sort by creation timestamp.
    CreatedAt,

    /// Sort by last update timestamp.
    UpdatedAt,
}

/// Port (interface) for persona persistence and retrieval operations.
///
/// PersonaRepositoryPort extends HEXSER's standard Repository and QueryRepository
/// traits to provide comprehensive persona storage capabilities. It adds
/// persona-specific operations for managing default personas and tracking
/// enabled tools via the persona_tools junction table.
///
/// # Examples
///
/// ```no_run
/// # use task_manager::ports::persona_repository_port::{PersonaRepositoryPort, PersonaFilter, PersonaSortKey};
/// # use task_manager::domain::persona::Persona;
/// # use hexser::ports::Repository;
/// # use hexser::ports::repository::QueryRepository;
/// # fn example<R: PersonaRepositoryPort>(repo: &mut R, persona: Persona) {
/// // Save a persona using HEXSER Repository trait (takes ownership)
/// repo.save(persona)?;
///
/// // Query default persona
/// let default = repo.find_default()?;
///
/// // Set a persona as default (clears all others)
/// repo.set_default("persona-123")?;
///
/// // Get enabled tools for a persona
/// let tools = repo.get_enabled_tools("persona-123")?;
///
/// // Enable a tool for a persona
/// repo.set_tool_enabled("persona-123", "bash_exec", true)?;
/// # std::result::Result::Ok::<(), String>(())
/// # }
/// ```
pub trait PersonaRepositoryPort:
    hexser::ports::Repository<crate::domain::persona::Persona>
    + hexser::ports::repository::QueryRepository<
        crate::domain::persona::Persona,
        Filter = PersonaFilter,
        SortKey = PersonaSortKey,
    >
    + Send
    + Sync
{
    /// Retrieves the default persona if one is set.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Persona))` if a default persona exists.
    /// * `Ok(None)` if no default persona is set.
    /// * `Err(String)` if the query fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::ports::persona_repository_port::PersonaRepositoryPort;
    /// # fn example<R: PersonaRepositoryPort>(repo: &mut R) {
    /// match repo.find_default()? {
    ///     std::option::Option::Some(persona) => std::println!("Default: {}", persona.name),
    ///     std::option::Option::None => std::println!("No default persona set"),
    /// }
    /// # std::result::Result::Ok::<(), String>(())
    /// # }
    /// ```
    fn find_default(&mut self) -> Result<std::option::Option<crate::domain::persona::Persona>, String>;

    /// Sets the specified persona as the default, clearing any previous default.
    ///
    /// Only one persona can be default at a time. This method updates the
    /// `is_default` field to `true` for the specified persona and `false`
    /// for all others.
    ///
    /// # Arguments
    ///
    /// * `persona_id` - The unique ID of the persona to set as default.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the default was set successfully.
    /// * `Err(String)` if the persona doesn't exist or the update fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::ports::persona_repository_port::PersonaRepositoryPort;
    /// # fn example<R: PersonaRepositoryPort>(repo: &mut R) {
    /// repo.set_default("persona-123")?;
    /// # std::result::Result::Ok::<(), String>(())
    /// # }
    /// ```
    fn set_default(&mut self, persona_id: &str) -> Result<(), String>;

    /// Retrieves the list of tool IDs enabled for a specific persona.
    ///
    /// This method queries the `persona_tools` junction table to find all
    /// tools that have `enabled = true` for the specified persona.
    ///
    /// # Arguments
    ///
    /// * `persona_id` - The unique ID of the persona.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` containing the tool IDs enabled for this persona.
    /// * `Err(String)` if the query fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::ports::persona_repository_port::PersonaRepositoryPort;
    /// # fn example<R: PersonaRepositoryPort>(repo: &mut R) {
    /// let tools = repo.get_enabled_tools("persona-123")?;
    /// for tool_id in tools {
    ///     std::println!("Tool enabled: {}", tool_id);
    /// }
    /// # std::result::Result::Ok::<(), String>(())
    /// # }
    /// ```
    fn get_enabled_tools(&mut self, persona_id: &str) -> Result<std::vec::Vec<String>, String>;

    /// Enables or disables a specific tool for a persona.
    ///
    /// This method manages the `persona_tools` junction table. If a row exists,
    /// it updates the `enabled` field. If no row exists, it inserts a new one.
    ///
    /// # Arguments
    ///
    /// * `persona_id` - The unique ID of the persona.
    /// * `tool_id` - The unique ID of the tool.
    /// * `enabled` - `true` to enable the tool, `false` to disable it.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeds.
    /// * `Err(String)` if the persona or tool doesn't exist, or the update fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::ports::persona_repository_port::PersonaRepositoryPort;
    /// # fn example<R: PersonaRepositoryPort>(repo: &mut R) {
    /// // Enable bash_exec for a persona
    /// repo.set_tool_enabled("persona-123", "bash_exec", true)?;
    ///
    /// // Disable file_delete for a persona
    /// repo.set_tool_enabled("persona-123", "file_delete", false)?;
    /// # std::result::Result::Ok::<(), String>(())
    /// # }
    /// ```
    fn set_tool_enabled(
        &mut self,
        persona_id: &str,
        tool_id: &str,
        enabled: bool,
    ) -> Result<(), String>;
}
