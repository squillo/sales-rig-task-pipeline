//! Defines the EmbeddingPort output port for generating text embeddings.
//!
//! This port represents the interface for converting text into vector embeddings
//! using LLM-based embedding models. Embeddings enable semantic similarity search
//! for RAG (Retrieval-Augmented Generation) systems.
//!
//! Revision History
//! - 2025-11-28T19:10:00Z @AI: Initial EmbeddingPort for Phase 1 RAG implementation.

/// Port (interface) for generating text embeddings.
///
/// EmbeddingPort defines the contract for adapters that can convert text into
/// dense vector representations. Implementations typically use LLM embedding models
/// (e.g., OpenAI text-embedding-ada-002, Ollama nomic-embed-text) to generate
/// fixed-dimension vectors that capture semantic meaning.
///
/// # Object Safety
///
/// This trait is object-safe and uses async_trait to support async methods
/// in trait objects. All methods require Send + Sync for concurrent usage.
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::ports::embedding_port::EmbeddingPort;
/// # async fn example<E: EmbeddingPort>(embedder: &E) {
/// let text = "This is a test document about Rust programming.";
/// let embedding = embedder.generate_embedding(text).await.unwrap();
/// println!("Generated {}-dimensional embedding", embedding.len());
/// # }
/// ```
#[async_trait::async_trait]
pub trait EmbeddingPort: std::marker::Send + std::marker::Sync {
    /// Generates a vector embedding for a single text string.
    ///
    /// This method converts input text into a dense vector representation
    /// that captures its semantic meaning. The dimensionality of the output
    /// vector depends on the underlying embedding model (typically 384, 768,
    /// 1536, or other fixed sizes).
    ///
    /// # Arguments
    ///
    /// * `text` - The input text to embed
    ///
    /// # Returns
    ///
    /// A Result containing the embedding vector (f32 values), or an error if generation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - LLM embedding request fails
    /// - Text is empty or invalid
    /// - Model is unavailable
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::ports::embedding_port::EmbeddingPort;
    /// # async fn example<E: EmbeddingPort>(embedder: &E) {
    /// let embedding = embedder.generate_embedding("Hello world").await.unwrap();
    /// std::assert!(embedding.len() > 0);
    /// std::assert!(embedding.iter().all(|&x| x.is_finite()));
    /// # }
    /// ```
    async fn generate_embedding(
        &self,
        text: &str,
    ) -> std::result::Result<std::vec::Vec<f32>, std::string::String>;

    /// Generates vector embeddings for multiple text strings in batch.
    ///
    /// This method converts multiple input texts into vector representations
    /// in a single batch operation. This is typically more efficient than
    /// calling `generate_embedding` repeatedly for large collections.
    ///
    /// All output vectors will have the same dimensionality.
    ///
    /// # Arguments
    ///
    /// * `texts` - A slice of input texts to embed
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of embedding vectors (one per input text),
    /// or an error if generation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - LLM embedding request fails
    /// - Any text is empty or invalid
    /// - Model is unavailable
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::ports::embedding_port::EmbeddingPort;
    /// # async fn example<E: EmbeddingPort>(embedder: &E) {
    /// let texts = &["Document 1", "Document 2", "Document 3"];
    /// let embeddings = embedder.generate_embeddings(texts).await.unwrap();
    /// std::assert_eq!(embeddings.len(), 3);
    /// std::assert_eq!(embeddings[0].len(), embeddings[1].len());
    /// # }
    /// ```
    async fn generate_embeddings(
        &self,
        texts: &[&str],
    ) -> std::result::Result<std::vec::Vec<std::vec::Vec<f32>>, std::string::String>;

    /// Returns the dimensionality of embeddings produced by this adapter.
    ///
    /// This is useful for validation and ensuring compatibility with stored
    /// artifacts. The dimension is model-specific and fixed.
    ///
    /// # Returns
    ///
    /// The number of dimensions in produced embedding vectors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::ports::embedding_port::EmbeddingPort;
    /// # async fn example<E: EmbeddingPort>(embedder: &E) {
    /// let dim = embedder.embedding_dimension().await;
    /// println!("Model produces {}-dimensional embeddings", dim);
    /// # }
    /// ```
    async fn embedding_dimension(&self) -> usize;
}
