//! Defines the Project domain entity for top-level context organization.
//!
//! Project represents the highest level of organization in Rigger, serving as
//! a container for related PRDs and their derived tasks. Projects provide scope
//! and context for filtering and navigation in the TUI.
//!
//! Revision History
//! - 2025-11-24T05:00:00Z @AI: Initial Project entity creation for Phase 1 of TUI project architecture refactoring.

/// Represents a project as the top-level organizational context.
///
/// A Project is the primary container that groups related PRDs together.
/// It provides scope for filtering tasks and PRDs in the TUI, and serves
/// as the "current context" when working with Rigger.
///
/// # Fields
///
/// * `id` - Unique identifier (UUID) for this project.
/// * `name` - The display name of the project.
/// * `description` - Optional longer description of the project's purpose.
/// * `created_at` - UTC timestamp when project was created.
/// * `prd_ids` - List of PRD IDs that belong to this project.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::project::Project;
/// let project = Project::new(
///     std::string::String::from("rig-task-pipeline"),
///     std::option::Option::Some(std::string::String::from("AI-driven task orchestration system")),
/// );
///
/// std::assert_eq!(project.name, "rig-task-pipeline");
/// std::assert!(project.id.len() > 0);
/// std::assert_eq!(project.prd_ids.len(), 0);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, hexser::HexEntity)]
pub struct Project {
    /// Unique identifier for this project (UUID v4).
    pub id: String,

    /// The display name of the project.
    pub name: String,

    /// Optional longer description of the project's purpose and scope.
    pub description: Option<String>,

    /// UTC timestamp when this project was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// List of PRD IDs that belong to this project.
    pub prd_ids: std::vec::Vec<String>,
}

impl Project {
    /// Creates a new Project with generated UUID and current timestamp.
    ///
    /// This constructor generates a UUID v4 identifier and sets the creation
    /// timestamp to the current UTC time. The prd_ids list is initialized empty.
    ///
    /// # Arguments
    ///
    /// * `name` - The display name for this project
    /// * `description` - Optional description of the project
    ///
    /// # Returns
    ///
    /// A new Project instance with generated ID and timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::project::Project;
    /// let project = Project::new(
    ///     std::string::String::from("my-project"),
    ///     std::option::Option::None,
    /// );
    ///
    /// std::assert_eq!(project.name, "my-project");
    /// std::assert!(project.description.is_none());
    /// std::assert_eq!(project.prd_ids.len(), 0);
    /// ```
    pub fn new(name: String, description: Option<String>) -> Self {
        Project {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            created_at: chrono::Utc::now(),
            prd_ids: std::vec::Vec::new(),
        }
    }

    /// Adds a PRD ID to this project's list of PRDs.
    ///
    /// This method appends a new PRD ID to the project's prd_ids vector.
    /// It does not check for duplicates.
    ///
    /// # Arguments
    ///
    /// * `prd_id` - The PRD ID to add to this project
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::project::Project;
    /// let mut project = Project::new(
    ///     std::string::String::from("my-project"),
    ///     std::option::Option::None,
    /// );
    ///
    /// project.add_prd(std::string::String::from("prd-123"));
    /// std::assert_eq!(project.prd_ids.len(), 1);
    /// std::assert_eq!(project.prd_ids[0], "prd-123");
    /// ```
    pub fn add_prd(&mut self, prd_id: String) {
        self.prd_ids.push(prd_id);
    }

    /// Removes a PRD ID from this project's list.
    ///
    /// This method removes the first occurrence of the given PRD ID.
    /// If the PRD ID is not found, no action is taken.
    ///
    /// # Arguments
    ///
    /// * `prd_id` - The PRD ID to remove from this project
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::project::Project;
    /// let mut project = Project::new(
    ///     std::string::String::from("my-project"),
    ///     std::option::Option::None,
    /// );
    ///
    /// project.add_prd(std::string::String::from("prd-123"));
    /// project.add_prd(std::string::String::from("prd-456"));
    /// std::assert_eq!(project.prd_ids.len(), 2);
    ///
    /// project.remove_prd(&std::string::String::from("prd-123"));
    /// std::assert_eq!(project.prd_ids.len(), 1);
    /// std::assert_eq!(project.prd_ids[0], "prd-456");
    /// ```
    pub fn remove_prd(&mut self, prd_id: &str) {
        if let std::option::Option::Some(index) = self.prd_ids.iter().position(|id| id == prd_id) {
            self.prd_ids.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_project_generates_id_and_timestamp() {
        // Test: Validates that new() generates a UUID and timestamp.
        // Justification: Every project needs a unique identifier and creation time.
        let project = Project::new(
            String::from("test-project"),
            std::option::Option::Some(String::from("A test project")),
        );

        std::assert!(!project.id.is_empty());
        std::assert_eq!(project.name, "test-project");
        std::assert_eq!(project.description, std::option::Option::Some(String::from("A test project")));
        std::assert!(project.created_at <= chrono::Utc::now());
        std::assert_eq!(project.prd_ids.len(), 0);
    }

    #[test]
    fn test_new_project_without_description() {
        // Test: Validates project creation with no description.
        // Justification: Description is optional and must work when omitted.
        let project = Project::new(
            String::from("minimal-project"),
            std::option::Option::None,
        );

        std::assert_eq!(project.name, "minimal-project");
        std::assert!(project.description.is_none());
    }

    #[test]
    fn test_add_prd_appends_to_list() {
        // Test: Validates that add_prd appends PRD IDs to the list.
        // Justification: Projects must track their associated PRDs.
        let mut project = Project::new(
            String::from("test-project"),
            std::option::Option::None,
        );

        project.add_prd(String::from("prd-001"));
        project.add_prd(String::from("prd-002"));

        std::assert_eq!(project.prd_ids.len(), 2);
        std::assert_eq!(project.prd_ids[0], "prd-001");
        std::assert_eq!(project.prd_ids[1], "prd-002");
    }

    #[test]
    fn test_remove_prd_removes_existing_id() {
        // Test: Validates that remove_prd removes a PRD ID from the list.
        // Justification: Projects must support unlinking PRDs.
        let mut project = Project::new(
            String::from("test-project"),
            std::option::Option::None,
        );

        project.add_prd(String::from("prd-001"));
        project.add_prd(String::from("prd-002"));
        project.add_prd(String::from("prd-003"));

        project.remove_prd("prd-002");

        std::assert_eq!(project.prd_ids.len(), 2);
        std::assert_eq!(project.prd_ids[0], "prd-001");
        std::assert_eq!(project.prd_ids[1], "prd-003");
    }

    #[test]
    fn test_remove_prd_handles_nonexistent_id() {
        // Test: Validates that remove_prd safely handles non-existent IDs.
        // Justification: Attempting to remove missing IDs should not panic.
        let mut project = Project::new(
            String::from("test-project"),
            std::option::Option::None,
        );

        project.add_prd(String::from("prd-001"));

        project.remove_prd("prd-999");

        std::assert_eq!(project.prd_ids.len(), 1);
        std::assert_eq!(project.prd_ids[0], "prd-001");
    }

    #[test]
    fn test_serialization_roundtrip() {
        // Test: Validates that Project can be serialized and deserialized.
        // Justification: Projects must persist to database, requiring serialization.
        let original = Project::new(
            String::from("test-project"),
            std::option::Option::Some(String::from("Test description")),
        );

        let serialized = serde_json::to_string(&original)
            .expect("Failed to serialize project");

        let deserialized: Project = serde_json::from_str(&serialized)
            .expect("Failed to deserialize project");

        std::assert_eq!(deserialized.id, original.id);
        std::assert_eq!(deserialized.name, original.name);
        std::assert_eq!(deserialized.description, original.description);
        std::assert_eq!(deserialized.prd_ids, original.prd_ids);
    }

    #[test]
    fn test_multiple_projects_have_unique_ids() {
        // Test: Validates that each project gets a unique ID.
        // Justification: UUID collision would break project tracking.
        let project1 = Project::new(String::from("project-1"), std::option::Option::None);
        let project2 = Project::new(String::from("project-2"), std::option::Option::None);
        let project3 = Project::new(String::from("project-3"), std::option::Option::None);

        std::assert_ne!(project1.id, project2.id);
        std::assert_ne!(project2.id, project3.id);
        std::assert_ne!(project1.id, project3.id);
    }
}
