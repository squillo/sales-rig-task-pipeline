//! Defines the Artifact domain entity for RAG knowledge storage.
//!
//! Artifact represents a chunk of knowledge extracted from various sources
//! (PRDs, files, web research, user input) along with its vector embedding
//! for semantic similarity search. Artifacts enable retrieval-augmented
//! generation (RAG) by providing relevant context to LLM agents.
//!
//! Revision History
//! - 2025-11-30T10:00:00Z @AI: Add Image and PDF artifact types with binary storage support. Added binary_content (base64), mime_type, source_url, and page_number fields for vision-capable LLM processing. Images and PDFs can now be stored with their base64 content for re-processing and audit trails.
//! - 2025-11-28T19:00:00Z @AI: Initial Artifact entity creation for Phase 1 of RAG implementation.

/// Represents a knowledge artifact with vector embedding for semantic search.
///
/// An Artifact is a chunk of text extracted from a source document or input,
/// stored with its vector embedding to enable similarity-based retrieval.
/// Artifacts are the fundamental unit of the RAG (Retrieval-Augmented Generation)
/// system, providing context to agents during task generation and execution.
///
/// # Fields
///
/// * `id` - Unique identifier (UUID) for this artifact.
/// * `project_id` - The project this artifact belongs to for scoping searches.
/// * `source_id` - Identifier of the source (PRD ID, file path, task ID, URL).
/// * `source_type` - Type of source this artifact was extracted from.
/// * `content` - The actual text content of this knowledge chunk.
/// * `embedding` - Vector representation of the content for similarity search.
/// * `metadata` - Optional JSON metadata (page numbers, line ranges, URLs, etc.).
/// * `created_at` - UTC timestamp when this artifact was created.
/// * `binary_content` - Base64-encoded binary data for images/PDFs.
/// * `mime_type` - MIME type of binary content (e.g., "image/png", "application/pdf").
/// * `source_url` - Original URL or path where media was sourced.
/// * `page_number` - For multi-page PDFs, the page number this artifact represents.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::artifact::{Artifact, ArtifactType};
/// let artifact = Artifact::new(
///     std::string::String::from("project-123"),
///     std::string::String::from("prd-456"),
///     ArtifactType::PRD,
///     std::string::String::from("The system must support real-time task updates..."),
///     std::vec![0.1, 0.2, 0.3], // embedding vector
///     std::option::Option::Some(std::string::String::from(r#"{"section":"Requirements","page":2}"#)),
/// );
///
/// std::assert_eq!(artifact.project_id, "project-123");
/// std::assert_eq!(artifact.source_type, ArtifactType::PRD);
/// std::assert!(artifact.embedding.len() > 0);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, hexser::HexEntity)]
pub struct Artifact {
    /// Unique identifier for this artifact (UUID v4).
    pub id: String,

    /// The project ID this artifact belongs to.
    pub project_id: String,

    /// Identifier of the source (PRD ID, file path, task ID, URL).
    pub source_id: String,

    /// Type of source this artifact was extracted from.
    pub source_type: ArtifactType,

    /// The actual text content of this knowledge chunk.
    pub content: String,

    /// Vector representation of the content for similarity search.
    pub embedding: std::vec::Vec<f32>,

    /// Optional JSON metadata (page numbers, line ranges, URLs, etc.).
    pub metadata: std::option::Option<String>,

    /// UTC timestamp when this artifact was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Base64-encoded binary content for images and PDFs.
    /// Only populated for Image and PDF artifact types.
    pub binary_content: std::option::Option<String>,

    /// MIME type of binary content (e.g., "image/png", "image/jpeg", "application/pdf").
    /// Only populated when binary_content is present.
    pub mime_type: std::option::Option<String>,

    /// Original URL or file path where the media was sourced from.
    /// Useful for cache invalidation and re-fetching.
    pub source_url: std::option::Option<String>,

    /// For multi-page PDFs, the page number this artifact represents (1-indexed).
    /// Allows correlating multiple artifacts to the same source PDF.
    pub page_number: std::option::Option<u32>,
}

/// Enumerates the types of sources from which artifacts can be extracted.
///
/// ArtifactType categorizes the origin of an artifact to enable filtering
/// and scoped searches. Different source types may have different metadata
/// schemas in the artifact's metadata field.
///
/// # Variants
///
/// * `PRD` - Extracted from a Product Requirements Document.
/// * `File` - Extracted from a file in the project repository.
/// * `WebResearch` - Extracted from web research or documentation.
/// * `UserInput` - Directly provided by the user as knowledge.
/// * `Image` - Image artifact with base64 binary content and vision LLM description.
/// * `PDF` - PDF page artifact with base64 binary content and vision LLM description.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::artifact::ArtifactType;
/// let source = ArtifactType::PRD;
/// std::assert_eq!(std::format!("{:?}", source), "PRD");
///
/// let image = ArtifactType::Image;
/// std::assert_eq!(std::format!("{:?}", image), "Image");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum ArtifactType {
    /// Artifact extracted from a Product Requirements Document.
    PRD,
    /// Artifact extracted from a file in the project repository.
    File,
    /// Artifact extracted from web research or documentation.
    WebResearch,
    /// Artifact directly provided by the user as knowledge.
    UserInput,
    /// Image artifact with base64 content and vision LLM generated description.
    Image,
    /// PDF page artifact with base64 content and vision LLM generated description.
    PDF,
}

impl Artifact {
    /// Creates a new Artifact with generated UUID and current timestamp.
    ///
    /// This constructor generates a UUID v4 identifier and sets the creation
    /// timestamp to the current UTC time. The embedding vector should be
    /// pre-computed by the embedding adapter before calling this constructor.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project this artifact belongs to
    /// * `source_id` - Identifier of the source (PRD ID, file path, etc.)
    /// * `source_type` - Type of source this artifact was extracted from
    /// * `content` - The actual text content of this knowledge chunk
    /// * `embedding` - Pre-computed vector representation of the content
    /// * `metadata` - Optional JSON metadata string
    ///
    /// # Returns
    ///
    /// A new Artifact instance with generated ID and timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::artifact::{Artifact, ArtifactType};
    /// let artifact = Artifact::new(
    ///     std::string::String::from("proj-1"),
    ///     std::string::String::from("prd-2"),
    ///     ArtifactType::PRD,
    ///     std::string::String::from("Feature description..."),
    ///     std::vec![0.5; 384], // example 384-dim embedding
    ///     std::option::Option::None,
    /// );
    ///
    /// std::assert!(artifact.id.len() > 0);
    /// std::assert_eq!(artifact.embedding.len(), 384);
    /// ```
    pub fn new(
        project_id: String,
        source_id: String,
        source_type: ArtifactType,
        content: String,
        embedding: std::vec::Vec<f32>,
        metadata: std::option::Option<String>,
    ) -> Self {
        Artifact {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            source_id,
            source_type,
            content,
            embedding,
            metadata,
            created_at: chrono::Utc::now(),
            binary_content: std::option::Option::None,
            mime_type: std::option::Option::None,
            source_url: std::option::Option::None,
            page_number: std::option::Option::None,
        }
    }

    /// Creates a new Image or PDF Artifact with binary content.
    ///
    /// This constructor is specifically for creating artifacts from images or PDFs
    /// that have been processed by a vision-capable LLM. The content field contains
    /// the LLM-generated description, while binary_content holds the base64 data.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project this artifact belongs to
    /// * `source_id` - Identifier of the source PRD or document
    /// * `source_type` - Must be ArtifactType::Image or ArtifactType::PDF
    /// * `content` - Vision LLM generated description of the image/PDF
    /// * `embedding` - Pre-computed vector representation of the description
    /// * `binary_content` - Base64-encoded image or PDF data
    /// * `mime_type` - MIME type (e.g., "image/png", "application/pdf")
    /// * `source_url` - Original URL or path where media was fetched
    /// * `page_number` - For PDFs, the page number (1-indexed); None for images
    ///
    /// # Returns
    ///
    /// A new Artifact instance with binary content fields populated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::artifact::{Artifact, ArtifactType};
    /// let image_artifact = Artifact::new_media(
    ///     std::string::String::from("proj-1"),
    ///     std::string::String::from("prd-2"),
    ///     ArtifactType::Image,
    ///     std::string::String::from("Architecture diagram showing microservices..."),
    ///     std::vec![0.5; 384],
    ///     std::string::String::from("iVBORw0KGgo..."), // base64 PNG data
    ///     std::string::String::from("image/png"),
    ///     std::string::String::from("https://example.com/arch.png"),
    ///     std::option::Option::None,
    /// );
    ///
    /// std::assert!(image_artifact.binary_content.is_some());
    /// std::assert_eq!(image_artifact.mime_type, std::option::Option::Some(std::string::String::from("image/png")));
    /// ```
    pub fn new_media(
        project_id: String,
        source_id: String,
        source_type: ArtifactType,
        content: String,
        embedding: std::vec::Vec<f32>,
        binary_content: String,
        mime_type: String,
        source_url: String,
        page_number: std::option::Option<u32>,
    ) -> Self {
        Artifact {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            source_id,
            source_type,
            content,
            embedding,
            metadata: std::option::Option::None,
            created_at: chrono::Utc::now(),
            binary_content: std::option::Option::Some(binary_content),
            mime_type: std::option::Option::Some(mime_type),
            source_url: std::option::Option::Some(source_url),
            page_number,
        }
    }

    /// Returns the dimensionality of the embedding vector.
    ///
    /// This is useful for validation and ensuring all artifacts in a collection
    /// use the same embedding model and dimension size.
    ///
    /// # Returns
    ///
    /// The number of dimensions in the embedding vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::artifact::{Artifact, ArtifactType};
    /// let artifact = Artifact::new(
    ///     std::string::String::from("proj-1"),
    ///     std::string::String::from("src-1"),
    ///     ArtifactType::File,
    ///     std::string::String::from("Content"),
    ///     std::vec![0.0; 768],
    ///     std::option::Option::None,
    /// );
    ///
    /// std::assert_eq!(artifact.embedding_dim(), 768);
    /// ```
    pub fn embedding_dim(&self) -> usize {
        self.embedding.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_new() {
        let artifact = Artifact::new(
            String::from("project-abc"),
            String::from("prd-123"),
            ArtifactType::PRD,
            String::from("This is a test artifact content."),
            vec![0.1, 0.2, 0.3, 0.4],
            Option::Some(String::from(r#"{"page":1}"#)),
        );

        assert_eq!(artifact.project_id, "project-abc");
        assert_eq!(artifact.source_id, "prd-123");
        assert_eq!(artifact.source_type, ArtifactType::PRD);
        assert_eq!(artifact.content, "This is a test artifact content.");
        assert_eq!(artifact.embedding.len(), 4);
        assert!(artifact.metadata.is_some());
        assert!(artifact.id.len() > 0);
    }

    #[test]
    fn test_artifact_type_equality() {
        assert_eq!(ArtifactType::PRD, ArtifactType::PRD);
        assert_ne!(ArtifactType::PRD, ArtifactType::File);
        assert_ne!(ArtifactType::WebResearch, ArtifactType::UserInput);
    }

    #[test]
    fn test_embedding_dim() {
        let artifact = Artifact::new(
            String::from("proj-1"),
            String::from("src-1"),
            ArtifactType::File,
            String::from("Content"),
            vec![0.0; 768],
            Option::None,
        );

        assert_eq!(artifact.embedding_dim(), 768);
    }

    #[test]
    fn test_artifact_with_no_metadata() {
        let artifact = Artifact::new(
            String::from("proj-1"),
            String::from("task-99"),
            ArtifactType::UserInput,
            String::from("User provided context"),
            vec![0.5; 384],
            Option::None,
        );

        assert!(artifact.metadata.is_none());
        assert_eq!(artifact.source_type, ArtifactType::UserInput);
    }

    #[test]
    fn test_artifact_new_has_none_binary_fields() {
        // Test: Verifies that new() constructor initializes binary fields to None.
        // Justification: Regular text artifacts should not have binary content.
        let artifact = Artifact::new(
            String::from("proj-1"),
            String::from("prd-1"),
            ArtifactType::PRD,
            String::from("Text content"),
            vec![0.1; 384],
            Option::None,
        );

        assert!(artifact.binary_content.is_none());
        assert!(artifact.mime_type.is_none());
        assert!(artifact.source_url.is_none());
        assert!(artifact.page_number.is_none());
    }

    #[test]
    fn test_artifact_new_media_image() {
        // Test: Verifies new_media() correctly creates an Image artifact.
        // Justification: Image artifacts require binary content for vision processing.
        let artifact = Artifact::new_media(
            String::from("proj-1"),
            String::from("prd-1"),
            ArtifactType::Image,
            String::from("Architecture diagram showing three-tier system"),
            vec![0.5; 768],
            String::from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJ"),
            String::from("image/png"),
            String::from("https://example.com/arch.png"),
            Option::None,
        );

        assert_eq!(artifact.source_type, ArtifactType::Image);
        assert!(artifact.binary_content.is_some());
        assert_eq!(artifact.mime_type, Option::Some(String::from("image/png")));
        assert_eq!(artifact.source_url, Option::Some(String::from("https://example.com/arch.png")));
        assert!(artifact.page_number.is_none());
    }

    #[test]
    fn test_artifact_new_media_pdf_page() {
        // Test: Verifies new_media() correctly creates a PDF page artifact.
        // Justification: PDF artifacts need page numbers to correlate chunks.
        let artifact = Artifact::new_media(
            String::from("proj-1"),
            String::from("prd-1"),
            ArtifactType::PDF,
            String::from("Page 3 contains the data flow diagrams..."),
            vec![0.5; 768],
            String::from("JVBERi0xLjQKJeLjz9MKMSAwIG9iago..."),
            String::from("application/pdf"),
            String::from("https://example.com/spec.pdf"),
            Option::Some(3),
        );

        assert_eq!(artifact.source_type, ArtifactType::PDF);
        assert!(artifact.binary_content.is_some());
        assert_eq!(artifact.mime_type, Option::Some(String::from("application/pdf")));
        assert_eq!(artifact.page_number, Option::Some(3));
    }

    #[test]
    fn test_artifact_type_image_pdf_variants() {
        // Test: Verifies Image and PDF type variants exist and are distinct.
        // Justification: Image and PDF are new types for vision processing.
        assert_eq!(ArtifactType::Image, ArtifactType::Image);
        assert_eq!(ArtifactType::PDF, ArtifactType::PDF);
        assert_ne!(ArtifactType::Image, ArtifactType::PDF);
        assert_ne!(ArtifactType::Image, ArtifactType::PRD);
        assert_ne!(ArtifactType::PDF, ArtifactType::File);
    }
}
