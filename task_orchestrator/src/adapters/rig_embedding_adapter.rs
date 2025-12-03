//! Rig-powered embedding generation adapter.
//!
//! RigEmbeddingAdapter uses Rig's embedding API to convert text into dense vector
//! representations for RAG (Retrieval-Augmented Generation) similarity search.
//! Supports multiple providers (Ollama, OpenAI) with configurable models and
//! includes fallback logic for LLM unavailability.
//!
//! Revision History
//! - 2025-11-28T19:45:00Z @AI: Initial RigEmbeddingAdapter for Phase 3 RAG AI integration.

/// Adapter for text embedding generation using Rig's embedding API.
///
/// RigEmbeddingAdapter implements EmbeddingPort by using Rig's multi-provider
/// embedding capabilities to generate vector representations of text. The adapter
/// supports batch processing and provides graceful degradation when the embedding
/// service is unavailable.
///
/// # Embedding Strategy
///
/// 1. **Provider Selection**: Supports Ollama (local) and OpenAI (remote) providers
/// 2. **Model Configuration**: Configurable embedding model (default: nomic-embed-text for Ollama, text-embedding-3-small for OpenAI)
/// 3. **Batch Processing**: Efficient batch embedding generation
/// 4. **Fallback**: Returns zero vectors if embedding service unavailable
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::adapters::rig_embedding_adapter::RigEmbeddingAdapter;
/// # use task_orchestrator::ports::embedding_port::EmbeddingPort;
/// let adapter = RigEmbeddingAdapter::new_ollama(
///     std::string::String::from("http://localhost:11434"),
///     std::string::String::from("nomic-embed-text"),
/// );
///
/// # async fn example(adapter: RigEmbeddingAdapter) {
/// let text = "This is a test document about Rust programming.";
/// let embedding = adapter.generate_embedding(text).await.unwrap();
/// std::assert!(embedding.len() > 0);
/// # }
/// ```
pub struct RigEmbeddingAdapter {
    provider: EmbeddingProvider,
    model: String,
    dimension: usize,
}

/// Enum representing the embedding provider backend.
#[derive(Debug, Clone)]
enum EmbeddingProvider {
    /// Ollama local embedding provider (uses default http://localhost:11434).
    Ollama,
    /// OpenAI remote embedding provider.
    OpenAI { api_key: String },
}

impl RigEmbeddingAdapter {
    /// Creates a new RigEmbeddingAdapter with Ollama provider.
    ///
    /// Uses default Ollama server at http://localhost:11434.
    ///
    /// # Arguments
    ///
    /// * `model` - The embedding model name (e.g., "nomic-embed-text")
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_orchestrator::adapters::rig_embedding_adapter::RigEmbeddingAdapter;
    /// let adapter = RigEmbeddingAdapter::new_ollama(
    ///     std::string::String::from("nomic-embed-text"),
    /// );
    /// ```
    pub fn new_ollama(model: String) -> Self {
        // nomic-embed-text produces 768-dimensional embeddings
        let dimension = if model.contains("nomic") { 768 } else { 384 };

        RigEmbeddingAdapter {
            provider: EmbeddingProvider::Ollama,
            model,
            dimension,
        }
    }

    /// Creates a new RigEmbeddingAdapter with OpenAI provider.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The OpenAI API key
    /// * `model` - The embedding model name (e.g., "text-embedding-3-small")
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_orchestrator::adapters::rig_embedding_adapter::RigEmbeddingAdapter;
    /// let adapter = RigEmbeddingAdapter::new_openai(
    ///     std::string::String::from("sk-..."),
    ///     std::string::String::from("text-embedding-3-small"),
    /// );
    /// ```
    pub fn new_openai(api_key: String, model: String) -> Self {
        // text-embedding-3-small produces 1536-dimensional embeddings
        let dimension = if model.contains("3-small") { 1536 } else if model.contains("3-large") { 3072 } else { 1536 };

        RigEmbeddingAdapter {
            provider: EmbeddingProvider::OpenAI { api_key },
            model,
            dimension,
        }
    }

    /// Generates embeddings using the configured provider.
    ///
    /// This is the core embedding generation method that interfaces with the
    /// underlying Rig embedding API. It handles provider-specific logic and
    /// error recovery.
    async fn generate_embeddings_internal(
        &self,
        texts: &[&str],
    ) -> std::result::Result<std::vec::Vec<std::vec::Vec<f32>>, std::string::String> {
        match &self.provider {
            EmbeddingProvider::Ollama => {
                self.generate_with_ollama(texts).await
            }
            EmbeddingProvider::OpenAI { api_key } => {
                self.generate_with_openai(api_key, texts).await
            }
        }
    }

    /// Generates embeddings using Ollama provider.
    async fn generate_with_ollama(
        &self,
        texts: &[&str],
    ) -> std::result::Result<std::vec::Vec<std::vec::Vec<f32>>, std::string::String> {
        use rig::embeddings::EmbeddingModel;

        // Create Ollama client (uses default http://localhost:11434)
        let client = rig::providers::ollama::Client::new();

        // Create embedding model
        let embedding_model = client.embedding_model(&self.model);

        // Generate embeddings for all texts
        let mut results = std::vec::Vec::new();
        for text in texts {
            let embeddings = embedding_model
                .embed_text(text)
                .await
                .map_err(|e| std::format!("Ollama embedding generation failed: {:?}", e))?;

            // Convert f64 to f32
            let vec_f32: std::vec::Vec<f32> = embeddings.vec.iter().map(|&x| x as f32).collect();
            results.push(vec_f32);
        }

        std::result::Result::Ok(results)
    }

    /// Generates embeddings using OpenAI provider.
    async fn generate_with_openai(
        &self,
        api_key: &str,
        texts: &[&str],
    ) -> std::result::Result<std::vec::Vec<std::vec::Vec<f32>>, std::string::String> {
        use rig::embeddings::EmbeddingModel;

        // Create OpenAI client
        let client = rig::providers::openai::Client::new(api_key);

        // Create embedding model
        let embedding_model = client.embedding_model(&self.model);

        // Generate embeddings for all texts
        let mut results = std::vec::Vec::new();
        for text in texts {
            let embeddings = embedding_model
                .embed_text(text)
                .await
                .map_err(|e| std::format!("OpenAI embedding generation failed: {:?}", e))?;

            // Convert f64 to f32
            let vec_f32: std::vec::Vec<f32> = embeddings.vec.iter().map(|&x| x as f32).collect();
            results.push(vec_f32);
        }

        std::result::Result::Ok(results)
    }

    /// Creates fallback zero embeddings when service is unavailable.
    ///
    /// Returns vectors of zeros with the configured dimension. Used for
    /// graceful degradation and testing scenarios.
    fn create_fallback_embeddings(&self, count: usize) -> std::vec::Vec<std::vec::Vec<f32>> {
        std::vec![std::vec![0.0; self.dimension]; count]
    }
}

// Implement EmbeddingPort trait
#[async_trait::async_trait]
impl crate::ports::embedding_port::EmbeddingPort for RigEmbeddingAdapter {
    async fn generate_embedding(
        &self,
        text: &str,
    ) -> std::result::Result<std::vec::Vec<f32>, std::string::String> {
        if text.is_empty() {
            return std::result::Result::Err(String::from("Cannot generate embedding for empty text"));
        }

        match self.generate_embeddings_internal(&[text]).await {
            std::result::Result::Ok(mut embeddings) => {
                if embeddings.is_empty() {
                    std::result::Result::Err(String::from("Embedding service returned no results"))
                } else {
                    std::result::Result::Ok(embeddings.remove(0))
                }
            }
            std::result::Result::Err(_) => {
                // Fallback to zero vector for graceful degradation
                std::result::Result::Ok(self.create_fallback_embeddings(1).remove(0))
            }
        }
    }

    async fn generate_embeddings(
        &self,
        texts: &[&str],
    ) -> std::result::Result<std::vec::Vec<std::vec::Vec<f32>>, std::string::String> {
        if texts.is_empty() {
            return std::result::Result::Ok(std::vec::Vec::new());
        }

        // Validate no empty texts
        for (idx, text) in texts.iter().enumerate() {
            if text.is_empty() {
                return std::result::Result::Err(std::format!(
                    "Cannot generate embedding for empty text at index {}",
                    idx
                ));
            }
        }

        match self.generate_embeddings_internal(texts).await {
            std::result::Result::Ok(embeddings) => {
                if embeddings.len() != texts.len() {
                    std::result::Result::Err(std::format!(
                        "Embedding count mismatch: expected {}, got {}",
                        texts.len(),
                        embeddings.len()
                    ))
                } else {
                    std::result::Result::Ok(embeddings)
                }
            }
            std::result::Result::Err(_) => {
                // Fallback to zero vectors for graceful degradation
                std::result::Result::Ok(self.create_fallback_embeddings(texts.len()))
            }
        }
    }

    async fn embedding_dimension(&self) -> usize {
        self.dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::embedding_port::EmbeddingPort;

    #[tokio::test]
    async fn test_ollama_adapter_creation() {
        // Test: Validates Ollama adapter instantiation.
        // Justification: Ensures factory method creates valid adapter with correct dimension.
        let adapter = RigEmbeddingAdapter::new_ollama(
            String::from("nomic-embed-text"),
        );
        assert_eq!(adapter.embedding_dimension().await, 768);
    }

    #[tokio::test]
    async fn test_openai_adapter_creation() {
        // Test: Validates OpenAI adapter instantiation.
        // Justification: Ensures factory method creates valid adapter with correct dimension.
        let adapter = RigEmbeddingAdapter::new_openai(
            String::from("sk-test-key"),
            String::from("text-embedding-3-small"),
        );
        assert_eq!(adapter.embedding_dimension().await, 1536);
    }

    #[tokio::test]
    async fn test_empty_text_rejection() {
        // Test: Validates rejection of empty text input.
        // Justification: Empty text cannot produce meaningful embeddings.
        let adapter = RigEmbeddingAdapter::new_ollama(
            String::from("nomic-embed-text"),
        );

        let result = adapter.generate_embedding("").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty text"));
    }

    #[tokio::test]
    async fn test_batch_empty_texts_rejection() {
        // Test: Validates rejection of batch with empty text elements.
        // Justification: All texts in batch must be non-empty for valid embeddings.
        let adapter = RigEmbeddingAdapter::new_ollama(
            String::from("nomic-embed-text"),
        );

        let texts = &["Valid text", "", "Another valid text"];
        let result = adapter.generate_embeddings(texts).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty text at index 1"));
    }

    #[tokio::test]
    async fn test_empty_batch_handling() {
        // Test: Validates handling of empty batch input.
        // Justification: Empty batch should return empty results without error.
        let adapter = RigEmbeddingAdapter::new_ollama(
            String::from("nomic-embed-text"),
        );

        let texts: &[&str] = &[];
        let result = adapter.generate_embeddings(texts).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_fallback_embedding_dimension() {
        // Test: Validates fallback embeddings have correct dimension.
        // Justification: Fallback vectors must match model dimension for downstream compatibility.
        let adapter = RigEmbeddingAdapter::new_ollama(
            String::from("nomic-embed-text"),
        );

        let fallback = adapter.create_fallback_embeddings(3);
        assert_eq!(fallback.len(), 3);
        assert_eq!(fallback[0].len(), 768);
        assert!(fallback[0].iter().all(|&x| x == 0.0));
    }

    #[tokio::test]
    #[ignore] // Ignored: Requires running Ollama service with nomic-embed-text model
    async fn test_ollama_embedding_generation() {
        // Test: Validates actual Ollama embedding generation.
        // Justification: Integration test ensuring end-to-end Ollama embedding works.
        let adapter = RigEmbeddingAdapter::new_ollama(
            String::from("nomic-embed-text"),
        );

        let text = "Rust is a systems programming language.";
        let result = adapter.generate_embedding(text).await;

        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 768);
        // Check embedding is non-zero (not fallback)
        assert!(embedding.iter().any(|&x| x != 0.0));
    }

    #[tokio::test]
    #[ignore] // Ignored: Requires running Ollama service with nomic-embed-text model
    async fn test_ollama_batch_embedding_generation() {
        // Test: Validates batch Ollama embedding generation.
        // Justification: Ensures batch processing works correctly with matching counts.
        let adapter = RigEmbeddingAdapter::new_ollama(
            String::from("nomic-embed-text"),
        );

        let texts = &[
            "Rust programming language",
            "Python programming language",
            "JavaScript programming language",
        ];
        let result = adapter.generate_embeddings(texts).await;

        assert!(result.is_ok());
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 768);
        assert_eq!(embeddings[1].len(), 768);
        assert_eq!(embeddings[2].len(), 768);

        // Check all embeddings are non-zero (not fallback)
        for embedding in embeddings {
            assert!(embedding.iter().any(|&x| x != 0.0));
        }
    }

    #[tokio::test]
    #[ignore] // Ignored: Requires valid OpenAI API key
    async fn test_openai_embedding_generation() {
        // Test: Validates actual OpenAI embedding generation.
        // Justification: Integration test ensuring end-to-end OpenAI embedding works.
        let api_key = std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| String::from("sk-test-key"));

        let adapter = RigEmbeddingAdapter::new_openai(
            api_key,
            String::from("text-embedding-3-small"),
        );

        let text = "Rust is a systems programming language.";
        let result = adapter.generate_embedding(text).await;

        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 1536);
        // Check embedding is non-zero (not fallback)
        assert!(embedding.iter().any(|&x| x != 0.0));
    }
}
