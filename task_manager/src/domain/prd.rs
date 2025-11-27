//! Defines the PRD (Product Requirements Document) domain entity.
//!
//! PRD represents a parsed product requirements document containing project
//! objectives, technical stack, constraints, and other metadata. PRDs serve
//! as the source of truth for task generation in Rigger workflows.
//!
//! Revision History
//! - 2025-11-24T05:00:00Z @AI: Add project_id field to link PRD to Project entity for Phase 1 TUI project architecture.
//! - 2025-11-22T16:00:00Z @AI: Initial PRD entity creation for Rigger Phase 0.

/// Represents a Product Requirements Document parsed from markdown.
///
/// A PRD is the input document that Rigger uses to generate a project's
/// task breakdown. It contains structured sections (Objectives, Tech Stack,
/// Constraints) parsed from markdown headers.
///
/// # Fields
///
/// * `id` - Unique identifier (UUID) for this PRD.
/// * `project_id` - The project this PRD belongs to.
/// * `title` - The title of the product/project.
/// * `objectives` - List of project objectives extracted from ## Objectives section.
/// * `tech_stack` - List of technologies/frameworks from ## Tech Stack section.
/// * `constraints` - List of constraints/requirements from ## Constraints section.
/// * `raw_content` - The original markdown content for reference.
/// * `created_at` - UTC timestamp when PRD was created.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::prd::PRD;
/// let prd = PRD {
///     id: std::string::String::from("prd-123"),
///     project_id: std::string::String::from("project-456"),
///     title: std::string::String::from("Build Rigger Platform"),
///     objectives: std::vec![
///         std::string::String::from("Enable AI agent task decomposition"),
///         std::string::String::from("Support multiple LLM providers"),
///     ],
///     tech_stack: std::vec![
///         std::string::String::from("Rust"),
///         std::string::String::from("Rig framework"),
///     ],
///     constraints: std::vec![
///         std::string::String::from("Must compile with Rust 2024 edition"),
///     ],
///     raw_content: std::string::String::from("# Sample PRD\n\n## Objectives\n..."),
///     created_at: chrono::Utc::now(),
/// };
///
/// std::assert_eq!(prd.objectives.len(), 2);
/// std::assert_eq!(prd.tech_stack.len(), 2);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PRD {
    /// Unique identifier for this PRD (UUID v4).
    pub id: String,

    /// The project ID this PRD belongs to.
    pub project_id: String,

    /// The title of the product or project.
    pub title: String,

    /// Project objectives parsed from the ## Objectives section.
    pub objectives: std::vec::Vec<String>,

    /// Technologies and frameworks from the ## Tech Stack section.
    pub tech_stack: std::vec::Vec<String>,

    /// Constraints and requirements from the ## Constraints section.
    pub constraints: std::vec::Vec<String>,

    /// The original markdown content for reference and debugging.
    pub raw_content: String,

    /// UTC timestamp when this PRD was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl PRD {
    /// Creates a new PRD with generated UUID and current timestamp.
    ///
    /// This constructor is a convenience method for creating PRDs with
    /// auto-generated metadata. Use this when parsing markdown content
    /// into a structured PRD entity.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project this PRD belongs to.
    /// * `title` - The product/project title.
    /// * `objectives` - List of project objectives.
    /// * `tech_stack` - List of technologies.
    /// * `constraints` - List of constraints.
    /// * `raw_content` - Original markdown content.
    ///
    /// # Returns
    ///
    /// A new PRD with generated UUID and current timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::prd::PRD;
    /// let prd = PRD::new(
    ///     std::string::String::from("project-123"),
    ///     std::string::String::from("Build Rigger"),
    ///     std::vec![std::string::String::from("Enable task decomposition")],
    ///     std::vec![std::string::String::from("Rust")],
    ///     std::vec![std::string::String::from("Rust 2024 edition")],
    ///     std::string::String::from("# Rigger PRD\n\n..."),
    /// );
    ///
    /// std::assert!(!prd.id.is_empty());
    /// std::assert_eq!(prd.project_id, "project-123");
    /// std::assert_eq!(prd.title, "Build Rigger");
    /// ```
    pub fn new(
        project_id: String,
        title: String,
        objectives: std::vec::Vec<String>,
        tech_stack: std::vec::Vec<String>,
        constraints: std::vec::Vec<String>,
        raw_content: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            title,
            objectives,
            tech_stack,
            constraints,
            raw_content,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_prd_creation() {
        // Test: Validates PRD constructor generates valid UUID and timestamp.
        // Justification: Ensures PRD entities are created with required metadata.
        let prd = super::PRD::new(
            std::string::String::from("project-123"),
            std::string::String::from("Test Project"),
            std::vec![
                std::string::String::from("Objective 1"),
                std::string::String::from("Objective 2"),
            ],
            std::vec![std::string::String::from("Rust")],
            std::vec![std::string::String::from("Must be fast")],
            std::string::String::from("# Test PRD\n\n## Objectives\n..."),
        );

        std::assert!(!prd.id.is_empty());
        std::assert_eq!(prd.project_id, "project-123");
        std::assert_eq!(prd.title, "Test Project");
        std::assert_eq!(prd.objectives.len(), 2);
        std::assert_eq!(prd.tech_stack.len(), 1);
        std::assert_eq!(prd.constraints.len(), 1);
        std::assert!(!prd.raw_content.is_empty());
    }

    #[test]
    fn test_prd_serialization() {
        // Test: Validates PRD can be serialized to JSON and deserialized back.
        // Justification: Ensures PRD persistence and API compatibility.
        let prd = super::PRD::new(
            std::string::String::from("project-456"),
            std::string::String::from("Serialization Test"),
            std::vec![std::string::String::from("Test objective")],
            std::vec![std::string::String::from("Rust")],
            std::vec![],
            std::string::String::from("Raw markdown"),
        );

        let json = serde_json::to_string(&prd).unwrap();
        let deserialized: super::PRD = serde_json::from_str(&json).unwrap();

        std::assert_eq!(deserialized.id, prd.id);
        std::assert_eq!(deserialized.project_id, prd.project_id);
        std::assert_eq!(deserialized.title, prd.title);
        std::assert_eq!(deserialized.objectives, prd.objectives);
    }

    #[test]
    fn test_prd_with_empty_sections() {
        // Test: Validates PRD handles empty sections gracefully.
        // Justification: Some PRDs may not have all sections populated.
        let prd = super::PRD::new(
            std::string::String::from("project-789"),
            std::string::String::from("Minimal PRD"),
            std::vec![],
            std::vec![],
            std::vec![],
            std::string::String::from("Minimal content"),
        );

        std::assert_eq!(prd.project_id, "project-789");
        std::assert!(prd.objectives.is_empty());
        std::assert!(prd.tech_stack.is_empty());
        std::assert!(prd.constraints.is_empty());
    }

    #[test]
    fn test_prd_uuid_uniqueness() {
        // Test: Validates each PRD gets a unique UUID.
        // Justification: Critical for PRD identification and task linkage.
        let prd1 = super::PRD::new(
            std::string::String::from("project-abc"),
            std::string::String::from("PRD 1"),
            std::vec![],
            std::vec![],
            std::vec![],
            std::string::String::from("Content 1"),
        );

        let prd2 = super::PRD::new(
            std::string::String::from("project-abc"),
            std::string::String::from("PRD 2"),
            std::vec![],
            std::vec![],
            std::vec![],
            std::string::String::from("Content 2"),
        );

        std::assert_ne!(prd1.id, prd2.id);
    }
}
