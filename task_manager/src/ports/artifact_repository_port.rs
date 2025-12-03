//! Defines the ArtifactRepositoryPort output port for artifact persistence and retrieval.
//!
//! This port represents the interface for persisting and querying artifacts using
//! the HEXSER framework's Repository pattern, extended with vector similarity search
//! capabilities for RAG (Retrieval-Augmented Generation). It provides type-safe
//! persistence operations plus semantic search via embeddings.
//!
//! Revision History
//! - 2025-11-28T19:05:00Z @AI: Initial ArtifactRepositoryPort trait definition for Phase 1 RAG implementation.

/// Filter criteria for querying artifacts.
///
/// ArtifactFilter defines the available filter operations for artifact queries.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe artifact filtering across different dimensions.
///
/// # Examples
///
/// ```
/// # use task_manager::ports::artifact_repository_port::ArtifactFilter;
/// # use task_manager::domain::artifact::ArtifactType;
/// let by_project = ArtifactFilter::ByProjectId(std::string::String::from("proj-123"));
/// let by_type = ArtifactFilter::BySourceType(ArtifactType::PRD);
/// let all = ArtifactFilter::All;
/// ```
#[derive(Debug, Clone)]
pub enum ArtifactFilter {
    /// Filter by unique artifact ID.
    ById(String),

    /// Filter by project ID to scope artifacts to a specific project.
    ByProjectId(String),

    /// Filter by source ID (PRD ID, file path, task ID, etc.).
    BySourceId(String),

    /// Filter by source type (PRD, File, WebResearch, UserInput).
    BySourceType(crate::domain::artifact::ArtifactType),

    /// Return all artifacts (no filtering).
    All,
}

/// Sort key options for artifact queries.
///
/// ArtifactSortKey defines the available fields by which artifacts can be sorted.
/// This enum is used by the QueryRepository trait to enable flexible,
/// type-safe artifact sorting.
///
/// # Examples
///
/// ```
/// # use task_manager::ports::artifact_repository_port::ArtifactSortKey;
/// let by_time = ArtifactSortKey::CreatedAt;
/// let by_type = ArtifactSortKey::SourceType;
/// ```
#[derive(Debug, Clone)]
pub enum ArtifactSortKey {
    /// Sort by creation timestamp (most recent first).
    CreatedAt,

    /// Sort by source type (alphabetically).
    SourceType,
}

/// Result structure for similarity search queries.
///
/// SimilarArtifact pairs an artifact with its similarity score (distance)
/// from the query embedding. Lower scores indicate higher similarity.
///
/// # Fields
///
/// * `artifact` - The retrieved artifact.
/// * `distance` - Cosine distance from query embedding (0.0 = identical, 2.0 = opposite).
///
/// # Examples
///
/// ```
/// # use task_manager::ports::artifact_repository_port::SimilarArtifact;
/// # use task_manager::domain::artifact::{Artifact, ArtifactType};
/// let artifact = Artifact::new(
///     std::string::String::from("proj-1"),
///     std::string::String::from("src-1"),
///     ArtifactType::PRD,
///     std::string::String::from("Content"),
///     std::vec![0.5; 384],
///     std::option::Option::None,
/// );
/// let similar = SimilarArtifact {
///     artifact,
///     distance: 0.15,
/// };
/// std::assert!(similar.distance < 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct SimilarArtifact {
    /// The retrieved artifact.
    pub artifact: crate::domain::artifact::Artifact,

    /// Cosine distance from query embedding (lower is more similar).
    pub distance: f32,
}

/// Port (interface) for artifact persistence, retrieval, and similarity search.
///
/// ArtifactRepositoryPort extends HEXSER's standard Repository and QueryRepository
/// traits to provide comprehensive artifact storage capabilities, plus a specialized
/// `find_similar` method for vector similarity search using embeddings.
///
/// # Standard Operations
///
/// Via HEXSER Repository trait:
/// - `save(artifact)` - Persist an artifact
/// - `find_by_id(id)` - Retrieve by UUID
/// - `delete(id)` - Remove an artifact
///
/// Via HEXSER QueryRepository trait:
/// - `find(filter, options)` - Query with filters and sorting
///
/// # RAG Operations
///
/// - `find_similar(query_embedding, limit, threshold, project_id)` - Semantic search
///
/// # Examples
///
/// ```no_run
/// # use task_manager::ports::artifact_repository_port::{ArtifactRepositoryPort, ArtifactFilter};
/// # use task_manager::domain::artifact::Artifact;
/// # use hexser::ports::Repository;
/// # fn example<R: ArtifactRepositoryPort>(repo: &mut R, artifact: Artifact) {
/// // Save an artifact using HEXSER Repository trait
/// repo.save(artifact).unwrap();
///
/// // Find similar artifacts via vector search
/// let query_embedding = std::vec![0.1, 0.2, 0.3];
/// let similar = repo.find_similar(
///     &query_embedding,
///     10,
///     std::option::Option::Some(0.8),
///     std::option::Option::Some(std::string::String::from("project-123")),
/// ).unwrap();
/// # }
/// ```
pub trait ArtifactRepositoryPort:
    hexser::ports::Repository<crate::domain::artifact::Artifact>
    + hexser::ports::repository::QueryRepository<
        crate::domain::artifact::Artifact,
        Filter = ArtifactFilter,
        SortKey = ArtifactSortKey,
    >
    + Send
    + Sync
{
    /// Finds artifacts semantically similar to the query embedding.
    ///
    /// This method performs vector similarity search using cosine distance
    /// to find the most relevant artifacts for a given query. It's the core
    /// operation enabling RAG (Retrieval-Augmented Generation) by providing
    /// context to LLM agents.
    ///
    /// # Arguments
    ///
    /// * `query_embedding` - The vector representation of the search query.
    /// * `limit` - Maximum number of results to return.
    /// * `threshold` - Optional maximum distance threshold (artifacts farther are excluded).
    /// * `project_id` - Optional project ID to scope the search.
    ///
    /// # Returns
    ///
    /// A vector of `SimilarArtifact` results sorted by ascending distance (most similar first).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The embedding dimension doesn't match stored artifacts
    /// - Database query fails
    /// - Vector search extension is unavailable
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::ports::artifact_repository_port::ArtifactRepositoryPort;
    /// # fn example<R: ArtifactRepositoryPort>(repo: &R) {
    /// // Find top 5 most similar artifacts with distance < 0.7
    /// let query = std::vec![0.1; 384];
    /// let results = repo.find_similar(
    ///     &query,
    ///     5,
    ///     std::option::Option::Some(0.7),
    ///     std::option::Option::None,
    /// ).unwrap();
    ///
    /// for similar in results {
    ///     println!("Distance: {}, Content: {}", similar.distance, similar.artifact.content);
    /// }
    /// # }
    /// ```
    fn find_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: std::option::Option<f32>,
        project_id: std::option::Option<String>,
    ) -> std::result::Result<std::vec::Vec<SimilarArtifact>, String>;
}
