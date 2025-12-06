//! Tool for listing artifacts in a project.
//!
//! ListProjectArtifactsTool allows Rig agents to see what knowledge artifacts
//! are available for a project, including their source types and content previews.
//!
//! Revision History
//! - 2025-12-03T00:00:00Z @AI: Create ListProjectArtifactsTool for LLM agent artifact browsing.

/// Error type for artifact listing operations.
#[derive(Debug, Clone)]
pub enum ListProjectArtifactsError {
    /// Repository query failed
    RepositoryError(std::string::String),
    /// Invalid parameters
    InvalidParameters(std::string::String),
}

impl std::fmt::Display for ListProjectArtifactsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListProjectArtifactsError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            ListProjectArtifactsError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
        }
    }
}

impl std::error::Error for ListProjectArtifactsError {}

/// Arguments for list_project_artifacts tool.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct ListProjectArtifactsArgs {
    /// Optional project ID to filter by (None = current project)
    #[serde(default)]
    pub project_id: std::option::Option<std::string::String>,

    /// Maximum number of artifacts to return (default: 20, max: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

/// Tool for listing project artifacts.
///
/// This tool enables Rig agents to browse available knowledge artifacts,
/// seeing what information has been ingested from PRDs, files, and web research.
///
/// # Examples
///
/// ```ignore
/// let tool = ListProjectArtifactsTool::new(artifact_repo, Some("project-123"));
/// let list = tool.call(ListProjectArtifactsArgs {
///     project_id: None, // Uses current project
///     limit: 20,
/// }).await?;
/// ```
#[derive(Clone)]
pub struct ListProjectArtifactsTool {
    artifact_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
    current_project_id: std::option::Option<std::string::String>,
}

impl ListProjectArtifactsTool {
    /// Creates a new ListProjectArtifactsTool.
    ///
    /// # Arguments
    ///
    /// * `artifact_repository` - Repository for artifact storage and queries
    /// * `current_project_id` - Optional current project ID from app context
    ///
    /// # Returns
    ///
    /// A new ListProjectArtifactsTool instance.
    pub fn new(
        artifact_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
        current_project_id: std::option::Option<std::string::String>,
    ) -> Self {
        Self {
            artifact_repository,
            current_project_id,
        }
    }

    /// Lists artifacts for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Optional project ID (None = use current_project_id)
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Formatted string containing artifact list.
    pub async fn list_artifacts(
        &self,
        project_id: std::option::Option<std::string::String>,
        limit: usize,
    ) -> std::result::Result<std::string::String, ListProjectArtifactsError> {
        // Validate parameters
        if limit == 0 || limit > 100 {
            return std::result::Result::Err(ListProjectArtifactsError::InvalidParameters(
                std::format!("Limit must be between 1 and 100, got {}", limit)
            ));
        }

        // Determine which project to query
        let target_project_id = project_id.or_else(|| self.current_project_id.clone());

        // Query repository
        let repo = self.artifact_repository.lock()
            .map_err(|e| ListProjectArtifactsError::RepositoryError(std::format!("Lock error: {}", e)))?;

        let filter = if let std::option::Option::Some(ref proj_id) = target_project_id {
            task_manager::ports::artifact_repository_port::ArtifactFilter::ByProjectId(proj_id.clone())
        } else {
            task_manager::ports::artifact_repository_port::ArtifactFilter::All
        };

        let options = hexser::ports::repository::FindOptions {
            sort: std::option::Option::Some(std::vec![
                hexser::ports::repository::Sort {
                    key: task_manager::ports::artifact_repository_port::ArtifactSortKey::CreatedAt,
                    direction: hexser::ports::repository::Direction::Desc,
                }
            ]),
            limit: std::option::Option::Some(limit as u32),
            offset: std::option::Option::None,
        };

        let artifacts = repo.find(&filter, options)
            .map_err(|e| ListProjectArtifactsError::RepositoryError(std::format!("{:?}", e)))?;

        // Format output
        if artifacts.is_empty() {
            let proj_msg = if target_project_id.is_some() {
                std::format!(" for project {}", target_project_id.unwrap())
            } else {
                std::string::String::from(" in the system")
            };
            return std::result::Result::Ok(std::format!("No artifacts found{}.", proj_msg));
        }

        let mut result = std::format!("Found {} artifact(s):\n\n", artifacts.len());

        for (i, artifact) in artifacts.iter().enumerate() {
            let source_type_str = self.format_artifact_type(&artifact.source_type);

            result.push_str(&std::format!(
                "{}. [{}] {} ({})\n",
                i + 1,
                &artifact.id[..8], // Show first 8 chars of ID
                source_type_str,
                artifact.source_id
            ));

            // Content preview
            let preview = if artifact.content.len() > 100 {
                std::format!("{}...", &artifact.content[..100])
            } else {
                artifact.content.clone()
            };
            result.push_str(&std::format!("   {}\n\n", preview));
        }

        std::result::Result::Ok(result)
    }

    /// Formats ArtifactType enum as human-readable string.
    fn format_artifact_type(&self, artifact_type: &task_manager::domain::artifact::ArtifactType) -> &'static str {
        match artifact_type {
            task_manager::domain::artifact::ArtifactType::PRD => "PRD",
            task_manager::domain::artifact::ArtifactType::File => "File",
            task_manager::domain::artifact::ArtifactType::WebResearch => "Web Research",
            task_manager::domain::artifact::ArtifactType::UserInput => "User Input",
            task_manager::domain::artifact::ArtifactType::Image => "Image",
            task_manager::domain::artifact::ArtifactType::PDF => "PDF",
        }
    }
}

#[allow(refining_impl_trait)]
impl rig::tool::Tool for ListProjectArtifactsTool {
    const NAME: &'static str = "list_project_artifacts";

    type Error = ListProjectArtifactsError;
    type Args = ListProjectArtifactsArgs;
    type Output = std::string::String;

    fn definition(&self, _prompt: std::string::String) -> impl std::future::Future<Output = rig::completion::ToolDefinition> + Send + Sync {
        async {
            rig::completion::ToolDefinition {
                name: Self::NAME.to_string(),
                description: "Lists knowledge artifacts available for a project, including their source types and content previews. Use this to see what information has been ingested.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {
                            "type": "string",
                            "description": "Optional project ID to filter by. If not provided, uses the current project."
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of artifacts to return (1-100, default: 20)",
                            "minimum": 1,
                            "maximum": 100,
                            "default": 20
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
            let handle = tokio::spawn(async move {
                tool.list_artifacts(args.project_id, args.limit).await
            });
            handle.await
                .map_err(|e| ListProjectArtifactsError::RepositoryError(std::format!("Task join error: {}", e)))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock artifact repository for testing.
    struct MockArtifactRepository {
        artifacts: std::vec::Vec<task_manager::domain::artifact::Artifact>,
    }

    impl hexser::ports::Repository<task_manager::domain::artifact::Artifact> for MockArtifactRepository {
        fn save(&mut self, entity: task_manager::domain::artifact::Artifact) -> hexser::HexResult<()> {
            self.artifacts.push(entity);
            std::result::Result::Ok(())
        }
    }

    impl hexser::ports::repository::QueryRepository<task_manager::domain::artifact::Artifact> for MockArtifactRepository {
        type Filter = task_manager::ports::artifact_repository_port::ArtifactFilter;
        type SortKey = task_manager::ports::artifact_repository_port::ArtifactSortKey;

        fn find_one(&self, _filter: &Self::Filter) -> hexser::HexResult<std::option::Option<task_manager::domain::artifact::Artifact>> {
            std::result::Result::Ok(std::option::Option::None)
        }

        fn find(&self, filter: &Self::Filter, _options: hexser::ports::repository::FindOptions<Self::SortKey>) -> hexser::HexResult<std::vec::Vec<task_manager::domain::artifact::Artifact>> {
            let filtered: std::vec::Vec<_> = match filter {
                task_manager::ports::artifact_repository_port::ArtifactFilter::ByProjectId(proj_id) => {
                    self.artifacts.iter().filter(|a| &a.project_id == proj_id).cloned().collect()
                }
                task_manager::ports::artifact_repository_port::ArtifactFilter::All => {
                    self.artifacts.clone()
                }
                _ => std::vec::Vec::new(),
            };
            std::result::Result::Ok(filtered)
        }
    }

    impl task_manager::ports::artifact_repository_port::ArtifactRepositoryPort for MockArtifactRepository {
        fn find_similar(
            &self,
            _query_embedding: &[f32],
            _limit: usize,
            _threshold: std::option::Option<f32>,
            _project_id: std::option::Option<String>,
        ) -> std::result::Result<std::vec::Vec<task_manager::ports::artifact_repository_port::SimilarArtifact>, String> {
            std::result::Result::Ok(std::vec::Vec::new())
        }
    }

    #[tokio::test]
    async fn test_list_artifacts() {
        // Test: Validates artifact listing works.
        // Justification: Core functionality.
        let mut repo = MockArtifactRepository { artifacts: std::vec::Vec::new() };

        hexser::ports::Repository::save(&mut repo, task_manager::domain::artifact::Artifact::new(
            std::string::String::from("proj-1"),
            std::string::String::from("prd-1"),
            task_manager::domain::artifact::ArtifactType::PRD,
            std::string::String::from("This is a PRD artifact about authentication."),
            std::vec![0.1, 0.2, 0.3],
            std::option::Option::None,
        )).unwrap();

        let tool = ListProjectArtifactsTool::new(
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
            std::option::Option::Some(std::string::String::from("proj-1")),
        );

        let result = tool.list_artifacts(std::option::Option::None, 20).await;
        std::assert!(result.is_ok());

        let output = result.unwrap();
        std::assert!(output.contains("Found 1 artifact"));
        std::assert!(output.contains("authentication"));
    }
}
