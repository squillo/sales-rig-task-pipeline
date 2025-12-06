//! Tool for retrieving PRD (Product Requirements Document) summary.
//!
//! GetPRDSummaryTool allows Rig agents to fetch PRD objectives, tech stack,
//! and constraints. This enables agents to understand project context and requirements.
//!
//! Revision History
//! - 2025-12-03T00:00:00Z @AI: Create GetPRDSummaryTool for LLM agent PRD inspection.

/// Error type for PRD summary operations.
#[derive(Debug, Clone)]
pub enum GetPRDSummaryError {
    /// PRD not found
    NotFound(std::string::String),
    /// Invalid parameters
    InvalidParameters(std::string::String),
}

impl std::fmt::Display for GetPRDSummaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetPRDSummaryError::NotFound(msg) => write!(f, "Not found: {}", msg),
            GetPRDSummaryError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
        }
    }
}

impl std::error::Error for GetPRDSummaryError {}

/// Arguments for get_prd_summary tool.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct GetPRDSummaryArgs {
    /// PRD ID to retrieve (optional - if not provided, returns current/default PRD)
    #[serde(default)]
    pub prd_id: std::option::Option<std::string::String>,
}

/// Tool for retrieving PRD summary information.
///
/// This tool enables Rig agents to fetch PRD objectives, tech stack, and constraints.
/// Note: This is a simplified version that works with in-memory PRD data.
/// Full repository integration will be added when PRDRepositoryPort is implemented.
///
/// # Examples
///
/// ```ignore
/// let tool = GetPRDSummaryTool::new(prds);
/// let summary = tool.call(GetPRDSummaryArgs {
///     prd_id: Some("prd-123".to_string()),
/// }).await?;
/// ```
#[derive(Clone)]
pub struct GetPRDSummaryTool {
    prds: std::sync::Arc<std::sync::Mutex<std::vec::Vec<task_manager::domain::prd::PRD>>>,
}

impl GetPRDSummaryTool {
    /// Creates a new GetPRDSummaryTool.
    ///
    /// # Arguments
    ///
    /// * `prds` - Shared vector of PRDs (from app state)
    ///
    /// # Returns
    ///
    /// A new GetPRDSummaryTool instance.
    pub fn new(
        prds: std::sync::Arc<std::sync::Mutex<std::vec::Vec<task_manager::domain::prd::PRD>>>,
    ) -> Self {
        Self { prds }
    }

    /// Retrieves PRD summary.
    ///
    /// # Arguments
    ///
    /// * `prd_id` - Optional PRD ID (None = first/default PRD)
    ///
    /// # Returns
    ///
    /// Formatted string containing PRD summary.
    pub async fn get_summary(
        &self,
        prd_id: std::option::Option<std::string::String>,
    ) -> std::result::Result<std::string::String, GetPRDSummaryError> {
        let prds = self.prds.lock()
            .map_err(|_| GetPRDSummaryError::InvalidParameters(std::string::String::from("Lock error")))?;

        if prds.is_empty() {
            return std::result::Result::Err(GetPRDSummaryError::NotFound(
                std::string::String::from("No PRDs available in the system")
            ));
        }

        // Find PRD by ID or use first one
        let prd = if let std::option::Option::Some(ref id) = prd_id {
            prds.iter()
                .find(|p| &p.id == id)
                .ok_or_else(|| GetPRDSummaryError::NotFound(
                    std::format!("No PRD found with ID: {}", id)
                ))?
        } else {
            &prds[0]
        };

        // Format output
        let mut result = std::format!("# PRD: {}\n\n", prd.title);
        result.push_str(&std::format!("**ID:** `{}`\n", prd.id));
        result.push_str(&std::format!("**Project ID:** `{}`\n\n", prd.project_id));

        // Objectives
        if !prd.objectives.is_empty() {
            result.push_str("## Objectives\n\n");
            for (i, obj) in prd.objectives.iter().enumerate() {
                result.push_str(&std::format!("{}. {}\n", i + 1, obj));
            }
            result.push('\n');
        } else {
            result.push_str("## Objectives\n\n*No objectives defined*\n\n");
        }

        // Tech Stack
        if !prd.tech_stack.is_empty() {
            result.push_str("## Tech Stack\n\n");
            for tech in &prd.tech_stack {
                result.push_str(&std::format!("- {}\n", tech));
            }
            result.push('\n');
        } else {
            result.push_str("## Tech Stack\n\n*No tech stack defined*\n\n");
        }

        // Constraints
        if !prd.constraints.is_empty() {
            result.push_str("## Constraints\n\n");
            for constraint in &prd.constraints {
                result.push_str(&std::format!("- {}\n", constraint));
            }
            result.push('\n');
        } else {
            result.push_str("## Constraints\n\n*No constraints defined*\n\n");
        }

        result.push_str(&std::format!(
            "---\n*Created:* {}\n",
            prd.created_at.format("%Y-%m-%d %H:%M")
        ));

        std::result::Result::Ok(result)
    }
}

#[allow(refining_impl_trait)]
impl rig::tool::Tool for GetPRDSummaryTool {
    const NAME: &'static str = "get_prd_summary";

    type Error = GetPRDSummaryError;
    type Args = GetPRDSummaryArgs;
    type Output = std::string::String;

    fn definition(&self, _prompt: std::string::String) -> impl std::future::Future<Output = rig::completion::ToolDefinition> + Send + Sync {
        async {
            rig::completion::ToolDefinition {
                name: Self::NAME.to_string(),
                description: "Retrieves a summary of a Product Requirements Document (PRD) including objectives, tech stack, and constraints. Use this to understand project requirements and context.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "prd_id": {
                            "type": "string",
                            "description": "Optional PRD ID. If not provided, returns the current/default PRD."
                        }
                    },
                    "required": []
                }),
            }
        }
    }

    fn call(&self, args: Self::Args) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send + Sync>> {
        let tool = self.clone();
        std::boxed::Box::pin(async move {
            tool.get_summary(args.prd_id).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_prd_summary() {
        // Test: Validates PRD summary retrieval.
        // Justification: Core functionality.
        let prds = std::vec![task_manager::domain::prd::PRD {
            id: std::string::String::from("prd-1"),
            project_id: std::string::String::from("proj-1"),
            title: std::string::String::from("Test PRD"),
            objectives: std::vec![
                std::string::String::from("Build authentication"),
                std::string::String::from("Implement database"),
            ],
            tech_stack: std::vec![
                std::string::String::from("Rust"),
                std::string::String::from("PostgreSQL"),
            ],
            constraints: std::vec![
                std::string::String::from("Must be secure"),
            ],
            raw_content: std::string::String::new(),
            created_at: chrono::Utc::now(),
        }];

        let tool = GetPRDSummaryTool::new(
            std::sync::Arc::new(std::sync::Mutex::new(prds)),
        );

        let result = tool.get_summary(std::option::Option::Some(std::string::String::from("prd-1"))).await;
        std::assert!(result.is_ok());

        let output = result.unwrap();
        std::assert!(output.contains("Test PRD"));
        std::assert!(output.contains("Build authentication"));
        std::assert!(output.contains("Rust"));
        std::assert!(output.contains("Must be secure"));
    }

    #[tokio::test]
    async fn test_get_prd_summary_not_found() {
        // Test: Validates error when PRD doesn't exist.
        // Justification: Must handle missing PRDs gracefully.
        let prds = std::vec![];
        let tool = GetPRDSummaryTool::new(
            std::sync::Arc::new(std::sync::Mutex::new(prds)),
        );

        let result = tool.get_summary(std::option::Option::None).await;
        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().to_string().contains("No PRDs available"));
    }
}
