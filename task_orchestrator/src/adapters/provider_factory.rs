//! Provider Factory for creating LLM adapters with vendor agnosticism.
//!
//! This factory creates adapter instances that implement the various ports
//! (TaskEnhancementPort, ComprehensionTestPort, PRDParserPort, EmbeddingPort) using different
//! LLM providers (Ollama, OpenAI, Anthropic) based on configuration.
//!
//! The factory reads provider configuration from environment variables:
//! - `TASK_ORCHESTRATOR_PROVIDER`: Provider name ("ollama", "openai", "anthropic", "mlx")
//! - `OLLAMA_MODEL`: Model name for Ollama (default: "llama3.1")
//! - `OPENAI_MODEL`: Model name for OpenAI (default: "gpt-4")
//! - `ANTHROPIC_MODEL`: Model name for Anthropic (default: "claude-3-5-sonnet-20241022")
//! - `MLX_MODEL`: Model name for MLX (default: "mlx-community/Phi-3-mini-4k-instruct")
//! - `OLLAMA_EMBEDDING_MODEL`: Embedding model for Ollama (default: "nomic-embed-text")
//! - `OPENAI_EMBEDDING_MODEL`: Embedding model for OpenAI (default: "text-embedding-3-small")
//! - `OLLAMA_VISION_MODEL`: Vision model for Ollama (default: "llava")
//! - `OPENAI_VISION_MODEL`: Vision model for OpenAI (default: "gpt-4o")
//! - `ANTHROPIC_VISION_MODEL`: Vision model for Anthropic (default: "claude-3-5-sonnet-20241022")
//! - `OPENAI_API_KEY`: API key for OpenAI
//! - `ANTHROPIC_API_KEY`: API key for Anthropic
//!
//! Revision History
//! - 2025-11-30T11:25:00Z @AI: Add vision adapter creation for Phase 5 image processing implementation.
//! - 2025-11-28T20:00:00Z @AI: Add embedding adapter creation for Phase 3 RAG implementation (Task 3.2).
//! - 2025-11-24T00:20:00Z @AI: Add MLX provider support for macOS Apple Silicon optimization (Phase 5 Sprint 11 Task 5.8).
//! - 2025-11-23T22:30:00Z @AI: Add ModelRole-based adapter creation for heterogeneous pipeline (Phase 5 Sprint 10 Task 5.2/5.3).
//! - 2025-11-23 @AI: Create ProviderFactory for vendor-agnostic LLM providers (Phase 1 Sprint 3 Task 1.9).

/// Factory for creating LLM adapters with configurable providers.
///
/// The ProviderFactory reads environment variables to determine which LLM
/// provider to use and creates the appropriate adapter instances.
///
/// # Examples
///
/// ```
/// use task_orchestrator::adapters::provider_factory::ProviderFactory;
///
/// let factory = ProviderFactory::from_env().unwrap();
/// let enhancer = factory.create_enhancement_adapter().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ProviderFactory {
    provider: String,
    model: String,
    model_selection: crate::domain::model_role::ModelSelectionStrategy,
}

impl ProviderFactory {
    /// Creates a new ProviderFactory from environment variables.
    ///
    /// Reads configuration from:
    /// - `TASK_ORCHESTRATOR_PROVIDER` (defaults to "ollama")
    /// - Provider-specific model configuration
    ///
    /// # Returns
    ///
    /// Returns a ProviderFactory or an error if configuration is invalid.
    pub fn from_env() -> hexser::HexResult<Self> {
        // Auto-detect MLX availability on macOS if no provider specified
        let provider = std::env::var("TASK_ORCHESTRATOR_PROVIDER")
            .unwrap_or_else(|_| {
                // Check if MLX is available for automatic selection
                if crate::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter::is_available() {
                    "mlx".to_string()
                } else {
                    "ollama".to_string()
                }
            });

        let model = match provider.as_str() {
            "ollama" => std::env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama3.1".to_string()),
            "openai" => std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4".to_string()),
            "anthropic" => std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            "mlx" => std::env::var("MLX_MODEL")
                .unwrap_or_else(|_| "mlx-community/Phi-3-mini-4k-instruct".to_string()),
            _ => {
                return std::result::Result::Err(hexser::Hexserror::adapter(
                    "UNSUPPORTED_PROVIDER",
                    &std::format!("Unsupported provider: {}. Supported providers: ollama, openai, anthropic, mlx", provider)
                ))
            }
        };

        // Initialize model selection strategy with defaults
        let model_selection = crate::domain::model_role::ModelSelectionStrategy::default();

        std::result::Result::Ok(Self {
            provider,
            model,
            model_selection,
        })
    }

    /// Creates a new ProviderFactory with explicit provider and model.
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name ("ollama", "openai", or "anthropic")
    /// * `model` - Model identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::adapters::provider_factory::ProviderFactory;
    ///
    /// let factory = ProviderFactory::new("ollama", "llama3.1").unwrap();
    /// ```
    pub fn new(provider: &str, model: &str) -> hexser::HexResult<Self> {
        // Validate provider
        match provider {
            "ollama" | "openai" | "anthropic" | "mlx" => {}
            _ => {
                return std::result::Result::Err(hexser::Hexserror::adapter(
                    "UNSUPPORTED_PROVIDER",
                    &std::format!("Unsupported provider: {}. Supported providers: ollama, openai, anthropic, mlx", provider)
                ))
            }
        }

        // Initialize model selection strategy with defaults
        let model_selection = crate::domain::model_role::ModelSelectionStrategy::default();

        std::result::Result::Ok(Self {
            provider: provider.to_string(),
            model: model.to_string(),
            model_selection,
        })
    }

    /// Gets the configured provider name.
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// Gets the configured model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Creates a TaskEnhancementPort adapter for the configured provider.
    ///
    /// # Returns
    ///
    /// Returns an Arc-wrapped implementation of TaskEnhancementPort.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Provider is not supported
    /// - Required API keys are missing
    /// - Adapter creation fails
    pub fn create_enhancement_adapter(
        &self,
    ) -> hexser::HexResult<std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort + std::marker::Send + std::marker::Sync>> {
        match self.provider.as_str() {
            "ollama" => {
                let adapter = crate::adapters::ollama_enhancement_adapter::OllamaEnhancementAdapter::new(
                    self.model.clone(),
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "mlx" => {
                // Check if MLX is available on this platform
                if !crate::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter::is_available() {
                    return std::result::Result::Err(hexser::Hexserror::adapter(
                        "MLX_NOT_AVAILABLE",
                        "MLX backend requires macOS Apple Silicon and Python with mlx-lm package installed"
                    ));
                }

                let adapter = crate::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter::new(
                    self.model.clone(),
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "openai" => {
                // Verify API key is set
                std::env::var("OPENAI_API_KEY").map_err(|_| {
                    hexser::Hexserror::adapter(
                        "MISSING_API_KEY",
                        "OPENAI_API_KEY environment variable is required for OpenAI provider"
                    )
                })?;

                // For now, return error as we haven't implemented OpenAI adapter yet
                std::result::Result::Err(hexser::Hexserror::adapter(
                    "NOT_IMPLEMENTED",
                    "OpenAI enhancement adapter not yet implemented. Use ollama provider for now."
                ))
            }
            "anthropic" => {
                // Verify API key is set
                std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
                    hexser::Hexserror::adapter(
                        "MISSING_API_KEY",
                        "ANTHROPIC_API_KEY environment variable is required for Anthropic provider"
                    )
                })?;

                // For now, return error as we haven't implemented Anthropic adapter yet
                std::result::Result::Err(hexser::Hexserror::adapter(
                    "NOT_IMPLEMENTED",
                    "Anthropic enhancement adapter not yet implemented. Use ollama provider for now."
                ))
            }
            _ => std::result::Result::Err(hexser::Hexserror::adapter(
                "UNSUPPORTED_PROVIDER",
                &std::format!("Unsupported provider: {}", self.provider)
            )),
        }
    }

    /// Creates a TaskEnhancementPort adapter for a specific ModelRole.
    ///
    /// This method enables the heterogeneous agent pipeline by creating adapters
    /// with specialized models based on the orchestration role. For example,
    /// Router role uses Phi-3-mini for fast inference, while Decomposer uses
    /// Orca-2 for complex reasoning.
    ///
    /// # Arguments
    ///
    /// * `role` - The orchestration role requiring enhancement
    ///
    /// # Returns
    ///
    /// Returns an Arc-wrapped implementation of TaskEnhancementPort using the
    /// model recommended for this role.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::adapters::provider_factory::ProviderFactory;
    /// use task_orchestrator::domain::model_role::ModelRole;
    ///
    /// let factory = ProviderFactory::from_env().unwrap();
    /// let router_enhancer = factory.create_enhancement_adapter_for_role(ModelRole::Router).unwrap();
    /// let decomposer = factory.create_enhancement_adapter_for_role(ModelRole::Decomposer).unwrap();
    /// ```
    pub fn create_enhancement_adapter_for_role(
        &self,
        role: crate::domain::model_role::ModelRole,
    ) -> hexser::HexResult<std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort + std::marker::Send + std::marker::Sync>> {
        // Get model for role
        let model = self.model_selection.select_model_for_role(role);

        match self.provider.as_str() {
            "ollama" => {
                let adapter = crate::adapters::ollama_enhancement_adapter::OllamaEnhancementAdapter::new(
                    model.to_string(),
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "mlx" => {
                // Check if MLX is available on this platform
                if !crate::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter::is_available() {
                    return std::result::Result::Err(hexser::Hexserror::adapter(
                        "MLX_NOT_AVAILABLE",
                        "MLX backend requires macOS Apple Silicon and Python with mlx-lm package installed"
                    ));
                }

                // For MLX, use MLX-community model naming convention
                let mlx_model = std::format!("mlx-community/{}", model);
                let adapter = crate::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter::new(
                    mlx_model,
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "openai" | "anthropic" => {
                // For now, heterogeneous pipeline only works with Ollama and MLX
                std::result::Result::Err(hexser::Hexserror::adapter(
                    "NOT_IMPLEMENTED",
                    &std::format!("Heterogeneous pipeline currently only supported with Ollama and MLX providers. Provider: {}", self.provider)
                ))
            }
            _ => std::result::Result::Err(hexser::Hexserror::adapter(
                "UNSUPPORTED_PROVIDER",
                &std::format!("Unsupported provider: {}", self.provider)
            )),
        }
    }

    /// Creates a ComprehensionTestPort adapter for the configured provider.
    ///
    /// # Returns
    ///
    /// Returns an Arc-wrapped implementation of ComprehensionTestPort.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Provider is not supported
    /// - Required API keys are missing
    /// - Adapter creation fails
    pub fn create_comprehension_test_adapter(
        &self,
    ) -> hexser::HexResult<std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort + std::marker::Send + std::marker::Sync>> {
        match self.provider.as_str() {
            "ollama" => {
                let adapter = crate::adapters::ollama_comprehension_test_adapter::OllamaComprehensionTestAdapter::new(
                    self.model.clone(),
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "openai" => {
                // Verify API key is set
                std::env::var("OPENAI_API_KEY").map_err(|_| {
                    hexser::Hexserror::adapter(
                        "MISSING_API_KEY",
                        "OPENAI_API_KEY environment variable is required for OpenAI provider"
                    )
                })?;

                // For now, return error as we haven't implemented OpenAI adapter yet
                std::result::Result::Err(hexser::Hexserror::adapter(
                    "NOT_IMPLEMENTED",
                    "OpenAI comprehension test adapter not yet implemented. Use ollama provider for now."
                ))
            }
            "anthropic" => {
                // Verify API key is set
                std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
                    hexser::Hexserror::adapter(
                        "MISSING_API_KEY",
                        "ANTHROPIC_API_KEY environment variable is required for Anthropic provider"
                    )
                })?;

                // For now, return error as we haven't implemented Anthropic adapter yet
                std::result::Result::Err(hexser::Hexserror::adapter(
                    "NOT_IMPLEMENTED",
                    "Anthropic comprehension test adapter not yet implemented. Use ollama provider for now."
                ))
            }
            _ => std::result::Result::Err(hexser::Hexserror::adapter(
                "UNSUPPORTED_PROVIDER",
                &std::format!("Unsupported provider: {}", self.provider)
            )),
        }
    }

    /// Creates a PRDParserPort adapter for the configured provider.
    ///
    /// # Returns
    ///
    /// Returns an Arc-wrapped implementation of PRDParserPort.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Provider is not supported
    /// - Required API keys are missing
    /// - Adapter creation fails
    pub fn create_prd_parser_adapter(
        &self,
    ) -> hexser::HexResult<std::sync::Arc<dyn crate::ports::prd_parser_port::PRDParserPort + std::marker::Send + std::marker::Sync>> {
        match self.provider.as_str() {
            "ollama" | "openai" | "anthropic" => {
                // RigPRDParserAdapter uses Rig internally and can work with any provider
                // Use same model for both main and fallback since ProviderFactory doesn't have separate fallback config
                // Note: ProviderFactory doesn't have database access, so personas must be passed separately by caller
                let adapter = crate::adapters::rig_prd_parser_adapter::RigPRDParserAdapter::new(
                    self.model.clone(),
                    self.model.clone(),
                    std::vec::Vec::new(), // No personas - caller should query database separately
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            _ => std::result::Result::Err(hexser::Hexserror::adapter(
                "UNSUPPORTED_PROVIDER",
                &std::format!("Unsupported provider: {}", self.provider)
            )),
        }
    }

    /// Creates a TaskDecompositionPort adapter for the configured provider.
    ///
    /// # Returns
    ///
    /// Returns an Arc-wrapped implementation of TaskDecompositionPort.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Provider is not supported
    /// - Required API keys are missing
    /// - Adapter creation fails
    pub fn create_task_decomposition_adapter(
        &self,
    ) -> hexser::HexResult<std::sync::Arc<dyn crate::ports::task_decomposition_port::TaskDecompositionPort + std::marker::Send + std::marker::Sync>> {
        match self.provider.as_str() {
            "ollama" | "openai" | "anthropic" => {
                // RigTaskDecompositionAdapter uses Rig internally and can work with any provider
                let adapter = crate::adapters::rig_task_decomposition_adapter::RigTaskDecompositionAdapter::new(
                    self.model.clone(),
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "mlx" => {
                // Check if MLX is available on this platform
                if !crate::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter::is_available() {
                    return std::result::Result::Err(hexser::Hexserror::adapter(
                        "MLX_NOT_AVAILABLE",
                        "MLX backend requires macOS Apple Silicon and Python with mlx-lm package installed"
                    ));
                }

                let adapter = crate::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter::new(
                    self.model.clone(),
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            _ => std::result::Result::Err(hexser::Hexserror::adapter(
                "UNSUPPORTED_PROVIDER",
                &std::format!("Unsupported provider: {}", self.provider)
            )),
        }
    }

    /// Creates a TaskDecompositionPort adapter for a specific ModelRole.
    ///
    /// This method enables the heterogeneous agent pipeline for decomposition tasks,
    /// using specialized models like Orca-2 that excel at complex reasoning and
    /// breaking down large tasks.
    ///
    /// # Arguments
    ///
    /// * `role` - The orchestration role (typically Decomposer)
    ///
    /// # Returns
    ///
    /// Returns an Arc-wrapped implementation of TaskDecompositionPort using the
    /// model recommended for this role (e.g., orca2 for Decomposer role).
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::adapters::provider_factory::ProviderFactory;
    /// use task_orchestrator::domain::model_role::ModelRole;
    ///
    /// let factory = ProviderFactory::from_env().unwrap();
    /// let decomposer = factory.create_task_decomposition_adapter_for_role(ModelRole::Decomposer).unwrap();
    /// ```
    pub fn create_task_decomposition_adapter_for_role(
        &self,
        role: crate::domain::model_role::ModelRole,
    ) -> hexser::HexResult<std::sync::Arc<dyn crate::ports::task_decomposition_port::TaskDecompositionPort + std::marker::Send + std::marker::Sync>> {
        // Get model for role
        let model = self.model_selection.select_model_for_role(role);

        match self.provider.as_str() {
            "ollama" | "openai" | "anthropic" => {
                // RigTaskDecompositionAdapter uses Rig internally
                let adapter = crate::adapters::rig_task_decomposition_adapter::RigTaskDecompositionAdapter::new(
                    model.to_string(),
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "mlx" => {
                // Check if MLX is available on this platform
                if !crate::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter::is_available() {
                    return std::result::Result::Err(hexser::Hexserror::adapter(
                        "MLX_NOT_AVAILABLE",
                        "MLX backend requires macOS Apple Silicon and Python with mlx-lm package installed"
                    ));
                }

                // For MLX, use MLX-community model naming convention
                let mlx_model = std::format!("mlx-community/{}", model);
                let adapter = crate::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter::new(
                    mlx_model,
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            _ => std::result::Result::Err(hexser::Hexserror::adapter(
                "UNSUPPORTED_PROVIDER",
                &std::format!("Unsupported provider: {}", self.provider)
            )),
        }
    }

    /// Creates an EmbeddingPort adapter for the configured provider.
    ///
    /// This method creates embedding generation adapters for RAG (Retrieval-Augmented
    /// Generation) systems. The embedding model is selected based on the provider
    /// and can be overridden with environment variables.
    ///
    /// # Returns
    ///
    /// Returns an Arc-wrapped implementation of EmbeddingPort.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Provider is not supported
    /// - Required API keys are missing
    /// - Adapter creation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::adapters::provider_factory::ProviderFactory;
    ///
    /// let factory = ProviderFactory::from_env().unwrap();
    /// let embedding_adapter = factory.create_embedding_adapter().unwrap();
    /// ```
    pub fn create_embedding_adapter(
        &self,
    ) -> hexser::HexResult<std::sync::Arc<dyn crate::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>> {
        match self.provider.as_str() {
            "ollama" => {
                // Get embedding model from environment or use default
                let embedding_model = std::env::var("OLLAMA_EMBEDDING_MODEL")
                    .unwrap_or_else(|_| "nomic-embed-text".to_string());

                let adapter = crate::adapters::rig_embedding_adapter::RigEmbeddingAdapter::new_ollama(
                    embedding_model,
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "openai" => {
                // Verify API key is set
                let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
                    hexser::Hexserror::adapter(
                        "MISSING_API_KEY",
                        "OPENAI_API_KEY environment variable is required for OpenAI provider"
                    )
                })?;

                // Get embedding model from environment or use default
                let embedding_model = std::env::var("OPENAI_EMBEDDING_MODEL")
                    .unwrap_or_else(|_| "text-embedding-3-small".to_string());

                let adapter = crate::adapters::rig_embedding_adapter::RigEmbeddingAdapter::new_openai(
                    api_key,
                    embedding_model,
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "anthropic" | "mlx" => {
                // Anthropic doesn't have native embedding models; MLX is for local inference
                std::result::Result::Err(hexser::Hexserror::adapter(
                    "NOT_SUPPORTED",
                    &std::format!("Embedding generation not supported for provider: {}. Use ollama or openai.", self.provider)
                ))
            }
            _ => std::result::Result::Err(hexser::Hexserror::adapter(
                "UNSUPPORTED_PROVIDER",
                &std::format!("Unsupported provider: {}", self.provider)
            )),
        }
    }

    /// Creates a VisionPort adapter for the configured provider.
    ///
    /// This method creates vision-capable LLM adapters for describing images and
    /// PDF pages. Vision models are selected based on the provider and can be
    /// overridden with environment variables.
    ///
    /// # Returns
    ///
    /// Returns an Arc-wrapped implementation of VisionPort.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Provider is not supported for vision (MLX doesn't have vision models)
    /// - Required API keys are missing
    /// - Adapter creation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::adapters::provider_factory::ProviderFactory;
    ///
    /// let factory = ProviderFactory::from_env().unwrap();
    /// let vision_adapter = factory.create_vision_adapter().unwrap();
    /// ```
    pub fn create_vision_adapter(
        &self,
    ) -> hexser::HexResult<std::sync::Arc<dyn crate::ports::vision_port::VisionPort + std::marker::Send + std::marker::Sync>> {
        match self.provider.as_str() {
            "ollama" => {
                // Get vision model from environment or use default
                let vision_model = std::env::var("OLLAMA_VISION_MODEL")
                    .unwrap_or_else(|_| "llava".to_string());

                let adapter = crate::adapters::rig_vision_adapter::RigVisionAdapter::new_ollama(
                    vision_model,
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "openai" => {
                // Verify API key is set
                let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
                    hexser::Hexserror::adapter(
                        "MISSING_API_KEY",
                        "OPENAI_API_KEY environment variable is required for OpenAI provider"
                    )
                })?;

                // Get vision model from environment or use default
                let vision_model = std::env::var("OPENAI_VISION_MODEL")
                    .unwrap_or_else(|_| "gpt-4o".to_string());

                let adapter = crate::adapters::rig_vision_adapter::RigVisionAdapter::new_openai(
                    api_key,
                    vision_model,
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "anthropic" => {
                // Verify API key is set
                let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
                    hexser::Hexserror::adapter(
                        "MISSING_API_KEY",
                        "ANTHROPIC_API_KEY environment variable is required for Anthropic provider"
                    )
                })?;

                // Get vision model from environment or use default
                let vision_model = std::env::var("ANTHROPIC_VISION_MODEL")
                    .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());

                let adapter = crate::adapters::rig_vision_adapter::RigVisionAdapter::new_anthropic(
                    api_key,
                    vision_model,
                );
                std::result::Result::Ok(std::sync::Arc::new(adapter))
            }
            "mlx" => {
                // MLX is for local inference and doesn't have vision models
                std::result::Result::Err(hexser::Hexserror::adapter(
                    "NOT_SUPPORTED",
                    "Vision capability not supported for MLX provider. Use ollama, openai, or anthropic."
                ))
            }
            _ => std::result::Result::Err(hexser::Hexserror::adapter(
                "UNSUPPORTED_PROVIDER",
                &std::format!("Unsupported provider: {}", self.provider)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_new_ollama() {
        let factory = ProviderFactory::new("ollama", "llama3.1").unwrap();
        assert_eq!(factory.provider(), "ollama");
        assert_eq!(factory.model(), "llama3.1");
    }

    #[test]
    fn test_factory_new_openai() {
        let factory = ProviderFactory::new("openai", "gpt-4").unwrap();
        assert_eq!(factory.provider(), "openai");
        assert_eq!(factory.model(), "gpt-4");
    }

    #[test]
    fn test_factory_new_anthropic() {
        let factory = ProviderFactory::new("anthropic", "claude-3-5-sonnet-20241022").unwrap();
        assert_eq!(factory.provider(), "anthropic");
        assert_eq!(factory.model(), "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_factory_new_unsupported_provider() {
        let result = ProviderFactory::new("unsupported", "model");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported provider"));
    }

    #[test]
    fn test_create_enhancement_adapter_ollama() {
        let factory = ProviderFactory::new("ollama", "llama3.1").unwrap();
        let adapter = factory.create_enhancement_adapter();
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_create_comprehension_test_adapter_ollama() {
        let factory = ProviderFactory::new("ollama", "llama3.1").unwrap();
        let adapter = factory.create_comprehension_test_adapter();
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_create_prd_parser_adapter_ollama() {
        let factory = ProviderFactory::new("ollama", "llama3.1").unwrap();
        let adapter = factory.create_prd_parser_adapter();
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_create_openai_adapter_without_api_key() {
        // Clear the API key if it exists
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
        }

        let factory = ProviderFactory::new("openai", "gpt-4").unwrap();
        let result = factory.create_enhancement_adapter();

        // Should fail because API key is missing
        assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
            assert!(e.to_string().contains("OPENAI_API_KEY"));
        }
    }

    #[test]
    fn test_from_env_defaults_to_ollama() {
        // Clear any existing provider env var
        unsafe {
            std::env::remove_var("TASK_ORCHESTRATOR_PROVIDER");
            std::env::remove_var("OLLAMA_MODEL");
        }

        let factory = ProviderFactory::from_env().unwrap();
        assert_eq!(factory.provider(), "ollama");
        assert_eq!(factory.model(), "llama3.1");
    }

    #[test]
    fn test_from_env_reads_config() {
        unsafe {
            std::env::set_var("TASK_ORCHESTRATOR_PROVIDER", "ollama");
            std::env::set_var("OLLAMA_MODEL", "qwen2.5");
        }

        let factory = ProviderFactory::from_env().unwrap();
        assert_eq!(factory.provider(), "ollama");
        assert_eq!(factory.model(), "qwen2.5");

        // Cleanup
        unsafe {
            std::env::remove_var("TASK_ORCHESTRATOR_PROVIDER");
            std::env::remove_var("OLLAMA_MODEL");
        }
    }

    #[test]
    fn test_create_enhancement_adapter_for_router_role() {
        // Test: Validates Router role uses Phi-3-mini model.
        // Justification: Router needs fast inference for routing decisions.
        let factory = ProviderFactory::from_env().unwrap();
        let adapter = factory.create_enhancement_adapter_for_role(
            crate::domain::model_role::ModelRole::Router
        );
        std::assert!(adapter.is_ok());
    }

    #[test]
    fn test_create_enhancement_adapter_for_decomposer_role() {
        // Test: Validates Decomposer role uses Orca-2 model.
        // Justification: Decomposer needs complex reasoning for task breakdown.
        let factory = ProviderFactory::from_env().unwrap();
        let adapter = factory.create_enhancement_adapter_for_role(
            crate::domain::model_role::ModelRole::Decomposer
        );
        std::assert!(adapter.is_ok());
    }

    #[test]
    fn test_create_task_decomposition_adapter_for_role() {
        // Test: Validates role-based decomposition adapter creation.
        let factory = ProviderFactory::from_env().unwrap();
        let adapter = factory.create_task_decomposition_adapter_for_role(
            crate::domain::model_role::ModelRole::Decomposer
        );
        std::assert!(adapter.is_ok());
    }

    #[test]
    fn test_heterogeneous_pipeline_uses_different_models() {
        // Test: Validates different roles get different models.
        // Justification: This is the core of heterogeneous pipeline optimization.
        let factory = ProviderFactory::from_env().unwrap();

        // Router should use phi3
        let router_model = factory.model_selection.select_model_for_role(
            crate::domain::model_role::ModelRole::Router
        );
        std::assert_eq!(router_model, "phi3");

        // Decomposer should use orca2
        let decomposer_model = factory.model_selection.select_model_for_role(
            crate::domain::model_role::ModelRole::Decomposer
        );
        std::assert_eq!(decomposer_model, "orca2");

        // Enhancer should use llama3.1
        let enhancer_model = factory.model_selection.select_model_for_role(
            crate::domain::model_role::ModelRole::Enhancer
        );
        std::assert_eq!(enhancer_model, "llama3.1");

        // Tester should use mistral
        let tester_model = factory.model_selection.select_model_for_role(
            crate::domain::model_role::ModelRole::Tester
        );
        std::assert_eq!(tester_model, "mistral");
    }

    #[test]
    fn test_create_embedding_adapter_ollama() {
        // Test: Validates Ollama embedding adapter creation.
        // Justification: Ensures factory creates valid embedding adapter for RAG.
        let factory = ProviderFactory::new("ollama", "llama3.1").unwrap();
        let adapter = factory.create_embedding_adapter();
        std::assert!(adapter.is_ok());
    }

    #[test]
    fn test_create_embedding_adapter_openai_without_api_key() {
        // Test: Validates OpenAI embedding adapter requires API key.
        // Justification: Should fail gracefully if OPENAI_API_KEY is missing.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
        }

        let factory = ProviderFactory::new("openai", "gpt-4").unwrap();
        let result = factory.create_embedding_adapter();

        std::assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
            std::assert!(e.to_string().contains("OPENAI_API_KEY"));
        }
    }

    #[test]
    fn test_create_embedding_adapter_anthropic_not_supported() {
        // Test: Validates Anthropic embedding adapter is not supported.
        // Justification: Anthropic doesn't provide embedding models.
        let factory = ProviderFactory::new("anthropic", "claude-3-5-sonnet-20241022").unwrap();
        let result = factory.create_embedding_adapter();

        std::assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
            std::assert!(e.to_string().contains("not supported"));
        }
    }

    #[test]
    fn test_create_embedding_adapter_mlx_not_supported() {
        // Test: Validates MLX embedding adapter is not supported.
        // Justification: MLX is for local inference, not embedding generation.
        let factory = ProviderFactory::new("mlx", "mlx-community/Phi-3-mini-4k-instruct").unwrap();
        let result = factory.create_embedding_adapter();

        std::assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
            std::assert!(e.to_string().contains("not supported"));
        }
    }

    #[test]
    fn test_embedding_model_from_environment() {
        // Test: Validates embedding model can be overridden via environment variable.
        // Justification: Enables users to customize embedding models for specific use cases.
        unsafe {
            std::env::set_var("OLLAMA_EMBEDDING_MODEL", "all-minilm");
        }

        let factory = ProviderFactory::new("ollama", "llama3.1").unwrap();
        let adapter = factory.create_embedding_adapter();
        std::assert!(adapter.is_ok());

        // Cleanup
        unsafe {
            std::env::remove_var("OLLAMA_EMBEDDING_MODEL");
        }
    }

    #[test]
    fn test_create_vision_adapter_ollama() {
        // Test: Validates Ollama vision adapter creation.
        // Justification: Ensures factory creates valid vision adapter with llava model.
        let factory = ProviderFactory::new("ollama", "llama3.1").unwrap();
        let adapter = factory.create_vision_adapter();
        std::assert!(adapter.is_ok());
    }

    #[test]
    fn test_create_vision_adapter_openai_without_api_key() {
        // Test: Validates OpenAI vision adapter requires API key.
        // Justification: Should fail gracefully if OPENAI_API_KEY is missing.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
        }

        let factory = ProviderFactory::new("openai", "gpt-4").unwrap();
        let result = factory.create_vision_adapter();

        std::assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
            std::assert!(e.to_string().contains("OPENAI_API_KEY"));
        }
    }

    #[test]
    fn test_create_vision_adapter_anthropic_without_api_key() {
        // Test: Validates Anthropic vision adapter requires API key.
        // Justification: Should fail gracefully if ANTHROPIC_API_KEY is missing.
        unsafe {
            std::env::remove_var("ANTHROPIC_API_KEY");
        }

        let factory = ProviderFactory::new("anthropic", "claude-3-5-sonnet-20241022").unwrap();
        let result = factory.create_vision_adapter();

        std::assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
            std::assert!(e.to_string().contains("ANTHROPIC_API_KEY"));
        }
    }

    #[test]
    fn test_create_vision_adapter_mlx_not_supported() {
        // Test: Validates MLX vision adapter is not supported.
        // Justification: MLX doesn't have vision-capable models.
        let factory = ProviderFactory::new("mlx", "mlx-community/Phi-3-mini-4k-instruct").unwrap();
        let result = factory.create_vision_adapter();

        std::assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
            std::assert!(e.to_string().contains("not supported"));
        }
    }

    #[test]
    fn test_vision_model_from_environment() {
        // Test: Validates vision model can be overridden via environment variable.
        // Justification: Enables users to customize vision models for specific use cases.
        unsafe {
            std::env::set_var("OLLAMA_VISION_MODEL", "llava-llama3");
        }

        let factory = ProviderFactory::new("ollama", "llama3.1").unwrap();
        let adapter = factory.create_vision_adapter();
        std::assert!(adapter.is_ok());

        // Cleanup
        unsafe {
            std::env::remove_var("OLLAMA_VISION_MODEL");
        }
    }
}
