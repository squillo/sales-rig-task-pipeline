//! Artifact ingestion service for RAG knowledge base management.
//!
//! ArtifactService coordinates the ingestion of text documents (PRDs, files, etc.)
//! into the RAG knowledge base by chunking content, generating embeddings, and
//! persisting artifacts with their vector representations for semantic search.
//!
//! # Chunking Strategy
//!
//! The service uses paragraph-based chunking as the default strategy:
//! - Splits text on double newlines (\n\n) to preserve semantic boundaries
//! - Filters out empty chunks
//! - Maintains chunk order with metadata
//! - Future: Can be enhanced with sentence-boundary detection, sliding windows, etc.
//!
//! # Examples
//!
//! ```no_run
//! # use task_orchestrator::services::artifact_service::ArtifactService;
//! # use std::sync::Arc;
//! # async fn example(
//! #     service: ArtifactService,
//! # ) {
//! let artifacts = service.ingest_prd(
//!     String::from("project-123"),
//!     String::from("prd-456"),
//!     String::from("This is a PRD.\n\nIt has multiple paragraphs.\n\nEach will be chunked."),
//! ).await.unwrap();
//! std::assert_eq!(artifacts.len(), 3);
//! # }
//! ```
//!
//! Revision History
//! - 2025-11-30T11:30:00Z @AI: Add missing binary_content fields for Phase 5 Artifact extension compatibility.
//! - 2025-11-28T20:15:00Z @AI: Create ArtifactService for Phase 3 RAG implementation (Task 4.1).

/// Service for ingesting and managing artifacts in the RAG knowledge base.
///
/// ArtifactService coordinates the complex workflow of:
/// 1. Chunking large text documents into semantically meaningful pieces
/// 2. Generating vector embeddings for each chunk via EmbeddingPort
/// 3. Creating Artifact domain entities with embeddings
/// 4. Persisting artifacts via ArtifactRepositoryPort
///
/// This service decouples the chunking strategy from the storage mechanism,
/// allowing both to evolve independently.
///
/// Note: The repository is wrapped in a Mutex to allow mutable access for the
/// Repository::save method while maintaining thread-safe shared ownership.
pub struct ArtifactService {
    artifact_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
    embedding_port: std::sync::Arc<dyn crate::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>,
}

impl ArtifactService {
    /// Creates a new ArtifactService with the given repository and embedding port.
    ///
    /// # Arguments
    ///
    /// * `artifact_repository` - Repository for persisting artifacts
    /// * `embedding_port` - Port for generating text embeddings
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::services::artifact_service::ArtifactService;
    /// # use std::sync::Arc;
    /// # fn example(
    /// #     repo: Arc<Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + Send>>,
    /// #     embeddings: Arc<dyn task_orchestrator::ports::embedding_port::EmbeddingPort + Send + Sync>,
    /// # ) {
    /// let service = ArtifactService::new(repo, embeddings);
    /// # }
    /// ```
    pub fn new(
        artifact_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
        embedding_port: std::sync::Arc<dyn crate::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>,
    ) -> Self {
        Self {
            artifact_repository,
            embedding_port,
        }
    }

    /// Ingests a PRD document by chunking, embedding, and storing artifacts.
    ///
    /// This method orchestrates the full artifact ingestion pipeline:
    /// 1. Chunks the PRD content into paragraphs
    /// 2. Generates embeddings for all chunks in batch
    /// 3. Creates Artifact entities with metadata
    /// 4. Persists artifacts to the repository
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project this PRD belongs to
    /// * `prd_id` - Unique identifier for the source PRD document
    /// * `content` - Full text content of the PRD
    ///
    /// # Returns
    ///
    /// Returns a vector of created Artifact entities with their embeddings.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Embedding generation fails
    /// - Repository persistence fails
    /// - Content is empty or invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::services::artifact_service::ArtifactService;
    /// # async fn example(service: ArtifactService) {
    /// let artifacts = service.ingest_prd(
    ///     String::from("proj-123"),
    ///     String::from("prd-456"),
    ///     String::from("# Product Requirements\n\nBuild a RAG system.\n\nIt should support vector search."),
    /// ).await.unwrap();
    ///
    /// std::assert!(artifacts.len() > 0);
    /// std::assert!(artifacts[0].embedding.len() > 0);
    /// # }
    /// ```
    pub async fn ingest_prd(
        &self,
        project_id: String,
        prd_id: String,
        content: String,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::artifact::Artifact>, std::string::String> {
        if content.is_empty() {
            return std::result::Result::Err(String::from("Cannot ingest empty PRD content"));
        }

        // 1. Chunk the content into paragraphs
        let chunks = self.chunk_text(&content);
        if chunks.is_empty() {
            return std::result::Result::Err(String::from("Chunking produced no results"));
        }

        // 2. Generate embeddings for all chunks in batch
        let chunk_refs: std::vec::Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let embeddings = self.embedding_port.generate_embeddings(&chunk_refs).await?;

        if embeddings.len() != chunks.len() {
            return std::result::Result::Err(std::format!(
                "Embedding count mismatch: expected {}, got {}",
                chunks.len(),
                embeddings.len()
            ));
        }

        // 3. Create Artifact entities for each chunk
        let mut artifacts = std::vec::Vec::new();
        for (i, (chunk, embedding)) in chunks.into_iter().zip(embeddings.into_iter()).enumerate() {
            let artifact = task_manager::domain::artifact::Artifact {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: project_id.clone(),
                source_id: prd_id.clone(),
                source_type: task_manager::domain::artifact::ArtifactType::PRD,
                content: chunk,
                embedding,
                metadata: std::option::Option::Some(std::format!("{{\"chunk_index\": {}}}", i)),
                created_at: chrono::Utc::now(),
                binary_content: std::option::Option::None,
                mime_type: std::option::Option::None,
                source_url: std::option::Option::None,
                page_number: std::option::Option::None,
            };
            artifacts.push(artifact);
        }

        // 4. Persist all artifacts to the repository
        let mut repo = self.artifact_repository.lock()
            .map_err(|e| std::format!("Failed to acquire repository lock: {}", e))?;

        for artifact in artifacts.clone() {
            repo.save(artifact)
                .map_err(|e| std::format!("Failed to save artifact: {}", e))?;
        }

        std::result::Result::Ok(artifacts)
    }

    /// Chunks text into semantic units using paragraph boundaries.
    ///
    /// This private method implements the chunking strategy by splitting on
    /// double newlines (\n\n) which typically indicate paragraph breaks in
    /// markdown and plain text documents.
    ///
    /// # Chunking Rules
    ///
    /// - Splits on "\n\n" (double newlines)
    /// - Trims whitespace from each chunk
    /// - Filters out empty chunks
    /// - Preserves chunk order
    ///
    /// # Future Enhancements
    ///
    /// - Sentence-boundary detection for more granular chunking
    /// - Sliding window with overlap for context preservation
    /// - Maximum chunk size enforcement
    /// - Section-header awareness for hierarchical chunking
    ///
    /// # Arguments
    ///
    /// * `text` - The full text to chunk
    ///
    /// # Returns
    ///
    /// Returns a vector of non-empty text chunks.
    fn chunk_text(&self, text: &str) -> std::vec::Vec<String> {
        text.split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock artifact repository for testing.
    struct MockArtifactRepository;

    impl hexser::ports::Repository<task_manager::domain::artifact::Artifact> for MockArtifactRepository {
        fn save(&mut self, _entity: task_manager::domain::artifact::Artifact) -> hexser::HexResult<()> {
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
            _limit: usize,
            _threshold: std::option::Option<f32>,
            _project_id: std::option::Option<String>,
        ) -> std::result::Result<std::vec::Vec<task_manager::ports::artifact_repository_port::SimilarArtifact>, String> {
            std::result::Result::Ok(std::vec::Vec::new())
        }
    }

    /// Mock embedding port for testing.
    struct MockEmbeddingPort {
        dimension: usize,
    }

    #[async_trait::async_trait]
    impl crate::ports::embedding_port::EmbeddingPort for MockEmbeddingPort {
        async fn generate_embedding(&self, _text: &str) -> std::result::Result<std::vec::Vec<f32>, String> {
            std::result::Result::Ok(std::vec![0.1; self.dimension])
        }

        async fn generate_embeddings(&self, texts: &[&str]) -> std::result::Result<std::vec::Vec<std::vec::Vec<f32>>, String> {
            std::result::Result::Ok(std::vec![std::vec![0.1; self.dimension]; texts.len()])
        }

        async fn embedding_dimension(&self) -> usize {
            self.dimension
        }
    }

    #[tokio::test]
    async fn test_chunk_text_paragraphs() {
        // Test: Validates paragraph-based chunking strategy.
        // Justification: Ensures text is split correctly on double newlines.
        let service = ArtifactService::new(
            std::sync::Arc::new(std::sync::Mutex::new(MockArtifactRepository)),
            std::sync::Arc::new(MockEmbeddingPort { dimension: 384 }),
        );

        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = service.chunk_text(text);

        std::assert_eq!(chunks.len(), 3);
        std::assert_eq!(chunks[0], "First paragraph.");
        std::assert_eq!(chunks[1], "Second paragraph.");
        std::assert_eq!(chunks[2], "Third paragraph.");
    }

    #[tokio::test]
    async fn test_chunk_text_filters_empty() {
        // Test: Validates empty chunks are filtered out.
        // Justification: Ensures no meaningless chunks are created.
        let service = ArtifactService::new(
            std::sync::Arc::new(std::sync::Mutex::new(MockArtifactRepository)),
            std::sync::Arc::new(MockEmbeddingPort { dimension: 384 }),
        );

        let text = "First paragraph.\n\n\n\nSecond paragraph.";
        let chunks = service.chunk_text(text);

        std::assert_eq!(chunks.len(), 2);
        std::assert_eq!(chunks[0], "First paragraph.");
        std::assert_eq!(chunks[1], "Second paragraph.");
    }

    #[tokio::test]
    async fn test_ingest_prd_creates_artifacts() {
        // Test: Validates PRD ingestion creates correct number of artifacts.
        // Justification: End-to-end test of ingestion pipeline.
        let service = ArtifactService::new(
            std::sync::Arc::new(std::sync::Mutex::new(MockArtifactRepository)),
            std::sync::Arc::new(MockEmbeddingPort { dimension: 384 }),
        );

        let result = service.ingest_prd(
            String::from("project-123"),
            String::from("prd-456"),
            String::from("First section.\n\nSecond section.\n\nThird section."),
        ).await;

        std::assert!(result.is_ok());
        let artifacts = result.unwrap();
        std::assert_eq!(artifacts.len(), 3);

        // Validate first artifact
        std::assert_eq!(artifacts[0].project_id, "project-123");
        std::assert_eq!(artifacts[0].source_id, "prd-456");
        std::assert_eq!(artifacts[0].source_type, task_manager::domain::artifact::ArtifactType::PRD);
        std::assert_eq!(artifacts[0].content, "First section.");
        std::assert_eq!(artifacts[0].embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_ingest_prd_empty_content_fails() {
        // Test: Validates empty content is rejected.
        // Justification: Empty PRDs cannot produce meaningful artifacts.
        let service = ArtifactService::new(
            std::sync::Arc::new(std::sync::Mutex::new(MockArtifactRepository)),
            std::sync::Arc::new(MockEmbeddingPort { dimension: 384 }),
        );

        let result = service.ingest_prd(
            String::from("project-123"),
            String::from("prd-456"),
            String::from(""),
        ).await;

        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().contains("empty"));
    }

    #[tokio::test]
    async fn test_ingest_prd_adds_chunk_metadata() {
        // Test: Validates chunk metadata includes index.
        // Justification: Metadata enables chunk reconstruction and ordering.
        let service = ArtifactService::new(
            std::sync::Arc::new(std::sync::Mutex::new(MockArtifactRepository)),
            std::sync::Arc::new(MockEmbeddingPort { dimension: 384 }),
        );

        let result = service.ingest_prd(
            String::from("project-123"),
            String::from("prd-456"),
            String::from("First.\n\nSecond."),
        ).await;

        std::assert!(result.is_ok());
        let artifacts = result.unwrap();

        std::assert!(artifacts[0].metadata.is_some());
        std::assert!(artifacts[0].metadata.as_ref().unwrap().contains("\"chunk_index\": 0"));
        std::assert!(artifacts[1].metadata.as_ref().unwrap().contains("\"chunk_index\": 1"));
    }
}
