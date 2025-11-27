//! Defines the Persona domain entity for agent assignee management.
//!
//! Persona represents a configured agent identity with specific capabilities,
//! LLM configuration, and enabled tools. Personas can be assigned to tasks
//! to control what the agent can do when executing that task.
//!
//! Revision History
//! - 2025-11-26T09:00:00Z @AI: Add project_id to scope personas to projects. Personas are now project-specific, enabling per-project agent teams. Updated all constructors and validation.
//! - 2025-11-26T07:05:00Z @AI: Initial Persona entity with tool management and validation methods.

/// Represents a persona (agent identity) with specific capabilities and configuration.
///
/// A Persona defines an agent's identity, role, LLM configuration, and which tools
/// it's permitted to use. Personas are scoped to a specific project, allowing different
/// agent teams per project. Only one persona per project can be marked as default.
///
/// # Fields
///
/// * `id` - Unique identifier (UUID).
/// * `project_id` - Optional project scope (None = global persona).
/// * `name` - Display name (e.g., "Alice", "DevBot", "ResearchAssistant").
/// * `role` - Role description (e.g., "Senior Developer", "QA Specialist").
/// * `description` - Detailed description of the persona's purpose and capabilities.
/// * `llm_provider` - Optional LLM provider override (e.g., "ollama", "rig", "candle").
/// * `llm_model` - Optional LLM model override (e.g., "llama3.1", "gpt-4o").
/// * `enabled_tools` - List of tool IDs this persona can use.
/// * `is_default` - Whether this is the default persona for new tasks in this project.
/// * `created_at` - UTC timestamp when persona was created.
/// * `updated_at` - UTC timestamp of last modification.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::persona::Persona;
/// let persona = Persona::with_default_tools(
///     std::string::String::from("persona-001"),
///     std::string::String::from("Alice"),
///     std::string::String::from("Senior Developer"),
///     std::string::String::from("Rust expert specializing in backend systems"),
/// );
///
/// std::assert_eq!(persona.name, "Alice");
/// std::assert!(persona.has_tool("code_search"));
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, hexser::HexEntity)]
pub struct Persona {
    /// Unique identifier for this persona (UUID v4).
    pub id: String,

    /// Optional project scope (None = global persona available to all projects).
    pub project_id: std::option::Option<String>,

    /// Display name for the persona.
    pub name: String,

    /// Role or job title for this persona.
    pub role: String,

    /// Detailed description of persona's purpose and capabilities.
    pub description: String,

    /// Optional LLM provider override (falls back to global task_tools config if None).
    pub llm_provider: std::option::Option<String>,

    /// Optional LLM model override (falls back to global task_tools config if None).
    pub llm_model: std::option::Option<String>,

    /// List of tool IDs this persona is permitted to use.
    pub enabled_tools: std::vec::Vec<String>,

    /// Whether this persona is the default for new tasks (only one can be true).
    pub is_default: bool,

    /// UTC timestamp when this persona was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// UTC timestamp of the last modification to this persona.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Persona {
    /// Creates a new Persona with empty tool list.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier (typically UUID v4).
    /// * `project_id` - Optional project scope (None = global persona).
    /// * `name` - Display name for the persona.
    /// * `role` - Role or job title.
    /// * `description` - Detailed description.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::persona::Persona;
    /// let persona = Persona::new(
    ///     std::string::String::from("persona-001"),
    ///     std::option::Option::Some(std::string::String::from("project-1")),
    ///     std::string::String::from("DevBot"),
    ///     std::string::String::from("QA Specialist"),
    ///     std::string::String::from("Automated testing and quality assurance"),
    /// );
    ///
    /// std::assert_eq!(persona.name, "DevBot");
    /// std::assert!(persona.enabled_tools.is_empty());
    /// std::assert!(!persona.is_default);
    /// ```
    pub fn new(id: String, project_id: std::option::Option<String>, name: String, role: String, description: String) -> Self {
        let now = chrono::Utc::now();
        Persona {
            id,
            project_id,
            name,
            role,
            description,
            llm_provider: std::option::Option::None,
            llm_model: std::option::Option::None,
            enabled_tools: std::vec::Vec::new(),
            is_default: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new Persona with default safe tools enabled.
    ///
    /// Default tools are: code_search, code_read, grep_search,
    /// web_search, web_fetch, doc_search (all Safe risk level).
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier.
    /// * `project_id` - Optional project scope (None = global persona).
    /// * `name` - Display name.
    /// * `role` - Role or job title.
    /// * `description` - Detailed description.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::persona::Persona;
    /// let persona = Persona::with_default_tools(
    ///     std::string::String::from("persona-002"),
    ///     std::option::Option::Some(std::string::String::from("project-1")),
    ///     std::string::String::from("ResearchBot"),
    ///     std::string::String::from("Research Assistant"),
    ///     std::string::String::from("Information gathering and documentation search"),
    /// );
    ///
    /// std::assert_eq!(persona.enabled_tools.len(), 6);
    /// std::assert!(persona.has_tool("web_search"));
    /// ```
    pub fn with_default_tools(id: String, project_id: std::option::Option<String>, name: String, role: String, description: String) -> Self {
        let mut persona = Self::new(id, project_id, name, role, description);
        persona.enabled_tools = std::vec![
            String::from("code_search"),
            String::from("code_read"),
            String::from("grep_search"),
            String::from("web_search"),
            String::from("web_fetch"),
            String::from("doc_search"),
        ];
        persona
    }

    /// Creates a new Persona with LLM configuration.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier.
    /// * `project_id` - Optional project scope (None = global persona).
    /// * `name` - Display name.
    /// * `role` - Role or job title.
    /// * `description` - Detailed description.
    /// * `llm_provider` - LLM provider (e.g., "ollama", "rig", "candle").
    /// * `llm_model` - LLM model (e.g., "llama3.1", "gpt-4o").
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::persona::Persona;
    /// let persona = Persona::with_llm_config(
    ///     std::string::String::from("persona-003"),
    ///     std::option::Option::Some(std::string::String::from("project-1")),
    ///     std::string::String::from("GPT-Researcher"),
    ///     std::string::String::from("Research Specialist"),
    ///     std::string::String::from("Uses GPT-4 for complex research tasks"),
    ///     String::from("rig"),
    ///     String::from("gpt-4o"),
    /// );
    ///
    /// std::assert_eq!(persona.llm_provider, std::option::Option::Some(String::from("rig")));
    /// std::assert_eq!(persona.llm_model, std::option::Option::Some(String::from("gpt-4o")));
    /// ```
    pub fn with_llm_config(
        id: String,
        project_id: std::option::Option<String>,
        name: String,
        role: String,
        description: String,
        llm_provider: String,
        llm_model: String,
    ) -> Self {
        let mut persona = Self::with_default_tools(id, project_id, name, role, description);
        persona.llm_provider = std::option::Option::Some(llm_provider);
        persona.llm_model = std::option::Option::Some(llm_model);
        persona
    }

    /// Checks if this persona has a specific tool enabled.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The tool ID to check (e.g., "code_search").
    ///
    /// # Returns
    ///
    /// `true` if the tool is in the enabled_tools list, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::persona::Persona;
    /// let persona = Persona::with_default_tools(
    ///     String::from("p1"),
    ///     String::from("Alice"),
    ///     String::from("Dev"),
    ///     String::from("Developer"),
    /// );
    ///
    /// std::assert!(persona.has_tool("code_search"));
    /// std::assert!(!persona.has_tool("bash_exec"));
    /// ```
    pub fn has_tool(&self, tool_id: &str) -> bool {
        self.enabled_tools.iter().any(|t| t == tool_id)
    }

    /// Enables a tool for this persona if not already enabled.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The tool ID to enable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::persona::Persona;
    /// let mut persona = Persona::new(
    ///     String::from("p1"),
    ///     String::from("Alice"),
    ///     String::from("Dev"),
    ///     String::from("Developer"),
    /// );
    ///
    /// persona.enable_tool(String::from("file_edit"));
    /// std::assert!(persona.has_tool("file_edit"));
    ///
    /// // Enabling again has no effect (no duplicates)
    /// persona.enable_tool(String::from("file_edit"));
    /// std::assert_eq!(persona.enabled_tools.len(), 1);
    /// ```
    pub fn enable_tool(&mut self, tool_id: String) {
        if !self.has_tool(&tool_id) {
            self.enabled_tools.push(tool_id);
            self.updated_at = chrono::Utc::now();
        }
    }

    /// Disables a tool for this persona.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The tool ID to disable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::persona::Persona;
    /// let mut persona = Persona::with_default_tools(
    ///     String::from("p1"),
    ///     String::from("Alice"),
    ///     String::from("Dev"),
    ///     String::from("Developer"),
    /// );
    ///
    /// let initial_count = persona.enabled_tools.len();
    /// persona.disable_tool("code_search");
    /// std::assert!(!persona.has_tool("code_search"));
    /// std::assert_eq!(persona.enabled_tools.len(), initial_count - 1);
    /// ```
    pub fn disable_tool(&mut self, tool_id: &str) {
        if let std::option::Option::Some(pos) = self.enabled_tools.iter().position(|t| t == tool_id) {
            self.enabled_tools.remove(pos);
            self.updated_at = chrono::Utc::now();
        }
    }

    /// Resets tools to the default safe tool set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::persona::Persona;
    /// let mut persona = Persona::new(
    ///     String::from("p1"),
    ///     String::from("Alice"),
    ///     String::from("Dev"),
    ///     String::from("Developer"),
    /// );
    ///
    /// persona.enable_tool(String::from("bash_exec"));
    /// persona.enable_tool(String::from("file_delete"));
    ///
    /// persona.reset_to_default_tools();
    /// std::assert_eq!(persona.enabled_tools.len(), 6);
    /// std::assert!(persona.has_tool("code_search"));
    /// std::assert!(!persona.has_tool("bash_exec"));
    /// ```
    pub fn reset_to_default_tools(&mut self) {
        self.enabled_tools = std::vec![
            String::from("code_search"),
            String::from("code_read"),
            String::from("grep_search"),
            String::from("web_search"),
            String::from("web_fetch"),
            String::from("doc_search"),
        ];
        self.updated_at = chrono::Utc::now();
    }

    /// Validates that the persona name is not empty.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(String)` with error message if invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::persona::Persona;
    /// let valid = Persona::new(
    ///     String::from("p1"),
    ///     String::from("Alice"),
    ///     String::from("Dev"),
    ///     String::from("Developer"),
    /// );
    /// std::assert!(valid.validate().is_ok());
    ///
    /// let invalid = Persona::new(
    ///     String::from("p1"),
    ///     String::from(""),
    ///     String::from("Dev"),
    ///     String::from("Developer"),
    /// );
    /// std::assert!(invalid.validate().is_err());
    /// ```
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.name.trim().is_empty() {
            return std::result::Result::Err(String::from("Persona name cannot be empty"));
        }
        if self.role.trim().is_empty() {
            return std::result::Result::Err(String::from("Persona role cannot be empty"));
        }
        std::result::Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_creation() {
        // Test: Validates Persona can be created with basic fields.
        // Justification: Ensures constructor works correctly.
        let persona = Persona::new(
            String::from("p1"),
            std::option::Option::Some(String::from("project-1")),
            String::from("Alice"),
            String::from("Developer"),
            String::from("Senior Rust developer"),
        );

        std::assert_eq!(persona.id, "p1");
        std::assert_eq!(persona.project_id, std::option::Option::Some(String::from("project-1")));
        std::assert_eq!(persona.name, "Alice");
        std::assert_eq!(persona.role, "Developer");
        std::assert!(persona.enabled_tools.is_empty());
        std::assert!(!persona.is_default);
    }

    #[test]
    fn test_persona_with_default_tools() {
        // Test: Validates default tool set is applied correctly.
        // Justification: Ensures new personas get safe tools by default.
        let persona = Persona::with_default_tools(
            String::from("p1"),
            std::option::Option::Some(String::from("project-1")),
            String::from("DevBot"),
            String::from("QA"),
            String::from("Quality Assurance"),
        );

        std::assert_eq!(persona.enabled_tools.len(), 6);
        std::assert!(persona.has_tool("code_search"));
        std::assert!(persona.has_tool("web_search"));
        std::assert!(!persona.has_tool("bash_exec"));
    }

    #[test]
    fn test_tool_management() {
        // Test: Validates enable/disable/reset tool operations.
        // Justification: Ensures tool configuration methods work correctly.
        let mut persona = Persona::new(
            String::from("p1"),
            std::option::Option::Some(String::from("project-1")),
            String::from("Alice"),
            String::from("Dev"),
            String::from("Developer"),
        );

        // Enable tool
        persona.enable_tool(String::from("file_edit"));
        std::assert!(persona.has_tool("file_edit"));

        // Enabling again doesn't duplicate
        persona.enable_tool(String::from("file_edit"));
        std::assert_eq!(persona.enabled_tools.len(), 1);

        // Disable tool
        persona.disable_tool("file_edit");
        std::assert!(!persona.has_tool("file_edit"));

        // Reset to defaults
        persona.enable_tool(String::from("bash_exec"));
        persona.reset_to_default_tools();
        std::assert_eq!(persona.enabled_tools.len(), 6);
        std::assert!(!persona.has_tool("bash_exec"));
    }

    #[test]
    fn test_persona_validation() {
        // Test: Validates validation logic for required fields.
        // Justification: Ensures empty names/roles are rejected.
        let valid = Persona::new(
            String::from("p1"),
            std::option::Option::Some(String::from("project-1")),
            String::from("Alice"),
            String::from("Dev"),
            String::from("Developer"),
        );
        std::assert!(valid.validate().is_ok());

        let invalid_name = Persona::new(
            String::from("p1"),
            std::option::Option::Some(String::from("project-1")),
            String::from(""),
            String::from("Dev"),
            String::from("Developer"),
        );
        std::assert!(invalid_name.validate().is_err());

        let invalid_role = Persona::new(
            String::from("p1"),
            std::option::Option::Some(String::from("project-1")),
            String::from("Alice"),
            String::from(""),
            String::from("Developer"),
        );
        std::assert!(invalid_role.validate().is_err());
    }

    #[test]
    fn test_persona_with_llm_config() {
        // Test: Validates LLM configuration is stored correctly.
        // Justification: Ensures persona-specific LLM overrides work.
        let persona = Persona::with_llm_config(
            String::from("p1"),
            std::option::Option::Some(String::from("project-1")),
            String::from("GPT-Bot"),
            String::from("Research"),
            String::from("GPT-4 powered research"),
            String::from("rig"),
            String::from("gpt-4o"),
        );

        std::assert_eq!(persona.llm_provider, std::option::Option::Some(String::from("rig")));
        std::assert_eq!(persona.llm_model, std::option::Option::Some(String::from("gpt-4o")));
        std::assert_eq!(persona.enabled_tools.len(), 6); // Still gets default tools
    }
}
