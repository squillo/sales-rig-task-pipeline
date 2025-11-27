//! Defines the AgentTool domain entity for agent capability configuration.
//!
//! AgentTool represents an available capability that can be enabled or disabled
//! for personas. Tools are categorized by type and risk level to help users
//! understand what permissions they're granting to agents.
//!
//! Revision History
//! - 2025-11-26T07:30:00Z @AI: Add HexEntity derive for HEXSER Repository pattern compatibility.
//! - 2025-11-26T07:00:00Z @AI: Initial AgentTool entity with ToolCategory and RiskLevel enums.

/// Represents an agent tool/capability that can be enabled for personas.
///
/// Tools define what actions an agent is permitted to perform. Each tool has
/// a category, risk level, and default enablement status. The risk level helps
/// users understand the potential impact of enabling a tool.
///
/// # Fields
///
/// * `id` - Unique identifier (e.g., "code_search", "bash_exec").
/// * `name` - Human-readable name (e.g., "Code Search", "Bash Execute").
/// * `description` - Brief description of what the tool does.
/// * `category` - Categorization for grouping in UI.
/// * `risk_level` - Safety classification of the tool.
/// * `is_default` - Whether this tool is enabled by default for new personas.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::agent_tool::{AgentTool, ToolCategory, RiskLevel};
/// let tool = AgentTool {
///     id: std::string::String::from("code_search"),
///     name: std::string::String::from("Code Search"),
///     description: std::string::String::from("Search codebase semantically"),
///     category: ToolCategory::Development,
///     risk_level: RiskLevel::Safe,
///     is_default: true,
/// };
///
/// std::assert_eq!(tool.id, "code_search");
/// std::assert_eq!(tool.risk_level, RiskLevel::Safe);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, hexser::HexEntity)]
pub struct AgentTool {
    /// Unique identifier for this tool (lowercase snake_case).
    pub id: String,

    /// Human-readable name for display in UI.
    pub name: String,

    /// Brief description of what the tool does.
    pub description: String,

    /// Category for organizational grouping.
    pub category: ToolCategory,

    /// Risk level classification for user awareness.
    pub risk_level: RiskLevel,

    /// Whether this tool is included in the default set for new personas.
    pub is_default: bool,
}

/// Category classification for agent tools.
///
/// Tools are grouped by category to help users understand their purpose
/// and find related tools when configuring personas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum ToolCategory {
    /// Code-related tools (search, read, edit, git operations).
    Development,

    /// Information gathering tools (web search, documentation).
    Research,

    /// File system operations (read, write, delete).
    FileSystem,

    /// Network operations (API calls, external services).
    Network,

    /// Database operations (queries, migrations).
    Database,

    /// Communication tools (email, Slack, notifications).
    Communication,
}

impl ToolCategory {
    /// Returns the display name for this category.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::agent_tool::ToolCategory;
    /// std::assert_eq!(ToolCategory::Development.display_name(), "Development");
    /// std::assert_eq!(ToolCategory::Research.display_name(), "Research");
    /// ```
    pub fn display_name(&self) -> &str {
        match self {
            ToolCategory::Development => "Development",
            ToolCategory::Research => "Research",
            ToolCategory::FileSystem => "FileSystem",
            ToolCategory::Network => "Network",
            ToolCategory::Database => "Database",
            ToolCategory::Communication => "Communication",
        }
    }

    /// Returns the icon/emoji for this category for UI display.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::agent_tool::ToolCategory;
    /// std::assert_eq!(ToolCategory::Development.icon(), "📂");
    /// std::assert_eq!(ToolCategory::Research.icon(), "🔍");
    /// ```
    pub fn icon(&self) -> &str {
        match self {
            ToolCategory::Development => "📂",
            ToolCategory::Research => "🔍",
            ToolCategory::FileSystem => "📁",
            ToolCategory::Network => "🌐",
            ToolCategory::Database => "💾",
            ToolCategory::Communication => "💬",
        }
    }
}

/// Risk level classification for agent tools.
///
/// Risk levels help users understand the potential impact of enabling a tool.
/// Safe tools are read-only with no side effects. Moderate tools have limited
/// write capabilities. High-risk tools can perform destructive operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
pub enum RiskLevel {
    /// Read-only operations with no side effects.
    ///
    /// Examples: code_search, code_read, web_search, doc_search
    Safe,

    /// Limited write operations, typically with confirmation prompts.
    ///
    /// Examples: file_edit, git_commit, db_query, api_call
    Moderate,

    /// Destructive operations that can cause data loss or security issues.
    ///
    /// Examples: file_delete, bash_exec, db_write, git_push
    High,
}

impl RiskLevel {
    /// Returns the display name for this risk level.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::agent_tool::RiskLevel;
    /// std::assert_eq!(RiskLevel::Safe.display_name(), "Safe");
    /// std::assert_eq!(RiskLevel::High.display_name(), "High");
    /// ```
    pub fn display_name(&self) -> &str {
        match self {
            RiskLevel::Safe => "Safe",
            RiskLevel::Moderate => "Moderate",
            RiskLevel::High => "High",
        }
    }

    /// Returns the color for UI display based on risk level.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::agent_tool::RiskLevel;
    /// std::assert_eq!(RiskLevel::Safe.color(), "green");
    /// std::assert_eq!(RiskLevel::High.color(), "red");
    /// ```
    pub fn color(&self) -> &str {
        match self {
            RiskLevel::Safe => "green",
            RiskLevel::Moderate => "yellow",
            RiskLevel::High => "red",
        }
    }
}

impl AgentTool {
    /// Creates a new AgentTool with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier (lowercase snake_case recommended).
    /// * `name` - Human-readable name for display.
    /// * `description` - Brief description of the tool's purpose.
    /// * `category` - Category classification.
    /// * `risk_level` - Risk level classification.
    /// * `is_default` - Whether to enable by default for new personas.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::agent_tool::{AgentTool, ToolCategory, RiskLevel};
    /// let tool = AgentTool::new(
    ///     std::string::String::from("web_search"),
    ///     std::string::String::from("Web Search"),
    ///     std::string::String::from("Search internet for information"),
    ///     ToolCategory::Research,
    ///     RiskLevel::Safe,
    ///     true,
    /// );
    ///
    /// std::assert_eq!(tool.id, "web_search");
    /// std::assert!(tool.is_default);
    /// ```
    pub fn new(
        id: String,
        name: String,
        description: String,
        category: ToolCategory,
        risk_level: RiskLevel,
        is_default: bool,
    ) -> Self {
        AgentTool {
            id,
            name,
            description,
            category,
            risk_level,
            is_default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_tool_creation() {
        // Test: Validates AgentTool struct can be created with all fields.
        // Justification: Ensures basic constructor works correctly.
        let tool = AgentTool::new(
            String::from("code_search"),
            String::from("Code Search"),
            String::from("Search codebase semantically"),
            ToolCategory::Development,
            RiskLevel::Safe,
            true,
        );

        std::assert_eq!(tool.id, "code_search");
        std::assert_eq!(tool.name, "Code Search");
        std::assert_eq!(tool.category, ToolCategory::Development);
        std::assert_eq!(tool.risk_level, RiskLevel::Safe);
        std::assert!(tool.is_default);
    }

    #[test]
    fn test_tool_category_display() {
        // Test: Validates category display names and icons.
        // Justification: Ensures UI rendering has correct strings.
        std::assert_eq!(ToolCategory::Development.display_name(), "Development");
        std::assert_eq!(ToolCategory::Development.icon(), "📂");
        std::assert_eq!(ToolCategory::Research.icon(), "🔍");
        std::assert_eq!(ToolCategory::FileSystem.icon(), "📁");
    }

    #[test]
    fn test_risk_level_ordering() {
        // Test: Validates risk levels are ordered Safe < Moderate < High.
        // Justification: Ensures sorting and comparison work as expected.
        std::assert!(RiskLevel::Safe < RiskLevel::Moderate);
        std::assert!(RiskLevel::Moderate < RiskLevel::High);
        std::assert!(RiskLevel::Safe < RiskLevel::High);
    }

    #[test]
    fn test_risk_level_colors() {
        // Test: Validates risk level color mappings for UI.
        // Justification: Ensures correct visual indicators in TUI.
        std::assert_eq!(RiskLevel::Safe.color(), "green");
        std::assert_eq!(RiskLevel::Moderate.color(), "yellow");
        std::assert_eq!(RiskLevel::High.color(), "red");
    }
}
