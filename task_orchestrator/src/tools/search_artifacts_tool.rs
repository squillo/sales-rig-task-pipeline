//! Semantic search tool for retrieving knowledge artifacts using RAG.
//!
//! SearchArtifactsTool allows Rig agents to query the knowledge base for relevant
//! information using vector similarity search. The tool converts text queries into
//! embeddings and retrieves the most similar artifacts from the database.
//!
//! Revision History
//! - 2025-11-30T11:35:00Z @AI: Add missing binary_content fields for Phase 5 Artifact extension compatibility.
//! - 2025-11-28T21:30:00Z @AI: Fix Sync requirement using tokio::spawn for Rig Tool trait compatibility (Task 5.1).
//! - 2025-11-28T21:15:00Z @AI: Create SearchArtifactsTool for Phase 5 RAG retrieval (Task 5.1).

/// Error type for artifact search operations.
#[derive(Debug, Clone)]
pub enum SearchArtifactsError {
    /// Embedding generation failed
    EmbeddingError(std::string::String),
    /// Repository query failed
    RepositoryError(std::string::String),
    /// Invalid search parameters
    InvalidParameters(std::string::String),
}

impl std::fmt::Display for SearchArtifactsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchArtifactsError::EmbeddingError(msg) => write!(f, "Embedding error: {}", msg),
            SearchArtifactsError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            SearchArtifactsError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
        }
    }
}

impl std::error::Error for SearchArtifactsError {}

/// Arguments for search_artifacts tool.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct SearchArtifactsArgs {
    /// Natural language query to search for (e.g., "authentication requirements", "API endpoints")
    pub query: std::string::String,

    /// Maximum number of results to return (default: 5, max: 20)
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Minimum similarity threshold (0.0-1.0). Lower distance = more similar (default: 0.5)
    #[serde(default = "default_threshold")]
    pub threshold: f32,
}

fn default_limit() -> usize {
    5
}

fn default_threshold() -> f32 {
    0.5
}

/// Semantic search tool for RAG knowledge retrieval.
///
/// This tool enables Rig agents to search the knowledge base for relevant information
/// using vector similarity. It converts natural language queries into embeddings and
/// retrieves the most semantically similar artifacts.
///
/// # Examples
///
/// ```ignore
/// let tool = SearchArtifactsTool::new(embedding_port, artifact_repo, Some("project-123"));
/// let results = tool.call(SearchArtifactsArgs {
///     query: "What are the authentication requirements?".to_string(),
///     limit: 5,
///     threshold: 0.5,
/// }).await?;
/// ```
#[derive(Clone)]
pub struct SearchArtifactsTool {
    embedding_port: std::sync::Arc<dyn crate::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>,
    artifact_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
    project_id: std::option::Option<std::string::String>,
}

impl SearchArtifactsTool {
    /// Creates a new SearchArtifactsTool.
    ///
    /// # Arguments
    ///
    /// * `embedding_port` - Port for generating query embeddings
    /// * `artifact_repository` - Repository for artifact storage and search
    /// * `project_id` - Optional project ID to scope search (None = search all projects)
    ///
    /// # Returns
    ///
    /// A new SearchArtifactsTool instance.
    pub fn new(
        embedding_port: std::sync::Arc<dyn crate::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>,
        artifact_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
        project_id: std::option::Option<std::string::String>,
    ) -> Self {
        Self {
            embedding_port,
            artifact_repository,
            project_id,
        }
    }

    /// Performs semantic search for artifacts.
    ///
    /// # Arguments
    ///
    /// * `query` - Natural language search query
    /// * `limit` - Maximum number of results
    /// * `threshold` - Minimum similarity threshold
    ///
    /// # Returns
    ///
    /// Formatted string containing search results with distances.
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
    ) -> std::result::Result<std::string::String, SearchArtifactsError> {
        // Validate parameters
        if query.is_empty() {
            return std::result::Result::Err(SearchArtifactsError::InvalidParameters(
                String::from("Query cannot be empty")
            ));
        }

        if limit == 0 || limit > 20 {
            return std::result::Result::Err(SearchArtifactsError::InvalidParameters(
                std::format!("Limit must be between 1 and 20, got {}", limit)
            ));
        }

        if threshold < 0.0 || threshold > 1.0 {
            return std::result::Result::Err(SearchArtifactsError::InvalidParameters(
                std::format!("Threshold must be between 0.0 and 1.0, got {}", threshold)
            ));
        }

        // 1. Generate embedding for query
        let query_embedding = self.embedding_port
            .generate_embedding(query)
            .await
            .map_err(|e| SearchArtifactsError::EmbeddingError(e))?;

        // 2. Search for similar artifacts
        let repo = self.artifact_repository.lock()
            .map_err(|e| SearchArtifactsError::RepositoryError(std::format!("Lock error: {}", e)))?;

        let similar_artifacts = repo
            .find_similar(
                &query_embedding,
                limit,
                std::option::Option::Some(threshold),
                self.project_id.clone(),
            )
            .map_err(|e| SearchArtifactsError::RepositoryError(e))?;

        // 3. Format results
        if similar_artifacts.is_empty() {
            return std::result::Result::Ok(String::from("No relevant artifacts found matching your query."));
        }

        let mut result = std::format!("Found {} relevant artifacts:\n\n", similar_artifacts.len());

        for (i, similar) in similar_artifacts.iter().enumerate() {
            let artifact = &similar.artifact;
            let distance = similar.distance;
            let similarity = 1.0 - distance; // Convert distance to similarity score

            result.push_str(&std::format!(
                "{}. [Similarity: {:.2}%] Source: {:?}\n   {}\n\n",
                i + 1,
                similarity * 100.0,
                artifact.source_type,
                artifact.content.chars().take(200).collect::<String>()
            ));

            // Truncate content if too long
            if artifact.content.len() > 200 {
                result.push_str("   ...\n\n");
            }
        }

        std::result::Result::Ok(result)
    }
}

#[allow(refining_impl_trait)]
impl rig::tool::Tool for SearchArtifactsTool {
    const NAME: &'static str = "search_artifacts";

    type Error = SearchArtifactsError;
    type Args = SearchArtifactsArgs;
    type Output = std::string::String;

    fn definition(&self, _prompt: std::string::String) -> impl std::future::Future<Output = rig::completion::ToolDefinition> + Send + Sync {
        async {
            rig::completion::ToolDefinition {
                name: Self::NAME.to_string(),
                description: "Searches the knowledge base for relevant information using semantic similarity. Use this to find context from PRDs, documentation, and other project artifacts.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language query describing what information you need (e.g., 'authentication requirements', 'API endpoints', 'database schema')"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (1-20, default: 5)",
                            "minimum": 1,
                            "maximum": 20,
                            "default": 5
                        },
                        "threshold": {
                            "type": "number",
                            "description": "Minimum similarity threshold (0.0-1.0, default: 0.5). Lower values return more results.",
                            "minimum": 0.0,
                            "maximum": 1.0,
                            "default": 0.5
                        }
                    },
                    "required": ["query"]
                }),
            }
        }
    }

    fn call(&self, args: Self::Args) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send + Sync>> {
        let tool = self.clone();
        std::boxed::Box::pin(async move {
            let handle = tokio::spawn(async move {
                tool.search(&args.query, args.limit, args.threshold).await
            });
            handle.await
                .map_err(|e| SearchArtifactsError::RepositoryError(std::format!("Task join error: {}", e)))?
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

        fn find(&self, _filter: &Self::Filter, _options: hexser::ports::repository::FindOptions<Self::SortKey>) -> hexser::HexResult<std::vec::Vec<task_manager::domain::artifact::Artifact>> {
            std::result::Result::Ok(std::vec::Vec::new())
        }
    }

    impl task_manager::ports::artifact_repository_port::ArtifactRepositoryPort for MockArtifactRepository {
        fn find_similar(
            &self,
            _query_embedding: &[f32],
            limit: usize,
            _threshold: std::option::Option<f32>,
            _project_id: std::option::Option<String>,
        ) -> std::result::Result<std::vec::Vec<task_manager::ports::artifact_repository_port::SimilarArtifact>, String> {
            // Return mock results
            let mut results = std::vec::Vec::new();
            for (i, artifact) in self.artifacts.iter().take(limit).enumerate() {
                results.push(task_manager::ports::artifact_repository_port::SimilarArtifact {
                    artifact: artifact.clone(),
                    distance: 0.1 * (i as f32), // Decreasing similarity
                });
            }
            std::result::Result::Ok(results)
        }
    }

    /// Mock embedding port for testing.
    struct MockEmbeddingPort;

    #[async_trait::async_trait]
    impl crate::ports::embedding_port::EmbeddingPort for MockEmbeddingPort {
        async fn generate_embedding(&self, _text: &str) -> std::result::Result<std::vec::Vec<f32>, String> {
            std::result::Result::Ok(std::vec![0.1, 0.2, 0.3])
        }

        async fn generate_embeddings(&self, texts: &[&str]) -> std::result::Result<std::vec::Vec<std::vec::Vec<f32>>, String> {
            std::result::Result::Ok(std::vec![std::vec![0.1, 0.2, 0.3]; texts.len()])
        }

        async fn embedding_dimension(&self) -> usize {
            3
        }
    }

    #[tokio::test]
    async fn test_search_with_results() {
        // Test: Validates successful search returns formatted results.
        // Justification: Core functionality must work correctly.
        let mut repo = MockArtifactRepository {
            artifacts: std::vec![],
        };

        // Add test artifacts
        hexser::ports::Repository::save(&mut repo, task_manager::domain::artifact::Artifact {
            id: String::from("art-1"),
            project_id: String::from("proj-1"),
            source_id: String::from("prd-1"),
            source_type: task_manager::domain::artifact::ArtifactType::PRD,
            content: String::from("This artifact discusses authentication requirements for the API."),
            embedding: std::vec![0.1, 0.2, 0.3],
            metadata: std::option::Option::None,
            created_at: chrono::Utc::now(),
            binary_content: std::option::Option::None,
            mime_type: std::option::Option::None,
            source_url: std::option::Option::None,
            page_number: std::option::Option::None,
        }).unwrap();

        let tool = SearchArtifactsTool::new(
            std::sync::Arc::new(MockEmbeddingPort),
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
            std::option::Option::None,
        );

        let result = tool.search("authentication", 5, 0.5).await;
        std::assert!(result.is_ok());

        let output = result.unwrap();
        std::assert!(output.contains("Found 1 relevant"));
        std::assert!(output.contains("authentication requirements"));
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        // Test: Validates empty query is rejected.
        // Justification: Must validate input parameters.
        let repo = MockArtifactRepository {
            artifacts: std::vec![],
        };

        let tool = SearchArtifactsTool::new(
            std::sync::Arc::new(MockEmbeddingPort),
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
            std::option::Option::None,
        );

        let result = tool.search("", 5, 0.5).await;
        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_search_invalid_limit() {
        // Test: Validates limit bounds are enforced.
        // Justification: Must prevent excessive result sets.
        let repo = MockArtifactRepository {
            artifacts: std::vec![],
        };

        let tool = SearchArtifactsTool::new(
            std::sync::Arc::new(MockEmbeddingPort),
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
            std::option::Option::None,
        );

        let result = tool.search("test", 0, 0.5).await;
        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().to_string().contains("Limit"));

        let result = tool.search("test", 25, 0.5).await;
        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().to_string().contains("Limit"));
    }

    #[tokio::test]
    async fn test_search_invalid_threshold() {
        // Test: Validates threshold bounds are enforced.
        // Justification: Must ensure valid similarity scores.
        let repo = MockArtifactRepository {
            artifacts: std::vec![],
        };

        let tool = SearchArtifactsTool::new(
            std::sync::Arc::new(MockEmbeddingPort),
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
            std::option::Option::None,
        );

        let result = tool.search("test", 5, -0.1).await;
        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().to_string().contains("Threshold"));

        let result = tool.search("test", 5, 1.5).await;
        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().to_string().contains("Threshold"));
    }

    #[tokio::test]
    async fn test_search_no_results() {
        // Test: Validates graceful handling when no artifacts match.
        // Justification: Must handle empty result sets gracefully.
        let repo = MockArtifactRepository {
            artifacts: std::vec![],
        };

        let tool = SearchArtifactsTool::new(
            std::sync::Arc::new(MockEmbeddingPort),
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
            std::option::Option::None,
        );

        let result = tool.search("nonexistent", 5, 0.5).await;
        std::assert!(result.is_ok());
        std::assert!(result.unwrap().contains("No relevant artifacts found"));
    }
}
