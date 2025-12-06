//! LLM provider configuration.
//!
//! Defines provider types (OpenAI, Anthropic, Ollama, etc.) and their
//! configuration including API keys, base URLs, timeouts, and retry policies.
//!
//! Revision History
//! - 2025-12-03T07:55:00Z @AI: Create ProviderConfig for rigger_core (Phase 2.2 of CONFIG-MODERN-20251203).

/// Configuration for a single LLM provider.
///
/// Includes connection details, authentication, and operational settings.
/// API keys are managed via environment variables and never stored in config files.
///
/// # Examples
///
/// ```
/// use rigger_core::config::provider::{ProviderConfig, ProviderType};
///
/// // Ollama provider (no API key required)
/// let ollama = ProviderConfig {
///     provider_type: ProviderType::Ollama,
///     base_url: "http://localhost:11434".to_string(),
///     api_key_env: None,
///     timeout_seconds: 120,
///     max_retries: 2,
///     default_model: "llama3.2".to_string(),
/// };
///
/// // No API key needed for Ollama
/// assert!(ollama.has_api_key());
/// assert_eq!(ollama.get_masked_api_key(), "(not required)");
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProviderConfig {
    /// Provider type (OpenAI, Anthropic, Ollama, etc.)
    #[serde(rename = "type")]
    pub provider_type: ProviderType,

    /// Base URL for API endpoint
    pub base_url: std::string::String,

    /// Environment variable name containing API key
    /// If None, provider doesn't require authentication (e.g., Ollama)
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub api_key_env: std::option::Option<std::string::String>,

    /// Timeout in seconds for requests
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    /// Default model to use if not specified in task slot
    pub default_model: std::string::String,
}

fn default_timeout() -> u64 {
    60
}

fn default_max_retries() -> usize {
    3
}

/// Supported LLM provider types.
///
/// Extensible enum supporting all major LLM providers plus custom providers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum ProviderType {
    /// OpenAI (GPT-4, GPT-4o, etc.)
    OpenAI,
    /// Anthropic (Claude Opus, Sonnet, Haiku)
    Anthropic,
    /// Ollama (local models)
    Ollama,
    /// Mistral AI
    Mistral,
    /// Groq (fast inference)
    Groq,
    /// Cohere (embeddings and generation)
    Cohere,
    /// Custom provider (user-defined)
    #[serde(untagged)]
    Custom(std::string::String),
}

impl ProviderConfig {
    /// Get the API key from environment variable.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(key))` if key is available
    /// - `Ok(None)` if provider doesn't need a key
    /// - `Err` if key is required but not set
    pub fn get_api_key(&self) -> std::result::Result<std::option::Option<std::string::String>, crate::config::error::ConfigError> {
        match &self.api_key_env {
            std::option::Option::Some(env_var) => {
                match std::env::var(env_var) {
                    std::result::Result::Ok(key) => std::result::Result::Ok(std::option::Option::Some(key)),
                    std::result::Result::Err(_) => std::result::Result::Err(crate::config::error::ConfigError::MissingApiKey {
                        provider: self.provider_type.clone(),
                        env_var: env_var.clone(),
                    }),
                }
            }
            std::option::Option::None => std::result::Result::Ok(std::option::Option::None), // Provider doesn't need API key
        }
    }

    /// Check if API key is available without retrieving it.
    ///
    /// # Returns
    ///
    /// `true` if key is set or not required, `false` if required but missing.
    pub fn has_api_key(&self) -> bool {
        match &self.api_key_env {
            std::option::Option::Some(env_var) => std::env::var(env_var).is_ok(),
            std::option::Option::None => true, // No key required
        }
    }

    /// Get masked API key for display (e.g., "sk-...abc123").
    ///
    /// Safe to display in UI without exposing full key.
    ///
    /// # Returns
    ///
    /// Masked key string like "sk-...abc123" or status message.
    pub fn get_masked_api_key(&self) -> std::string::String {
        match self.get_api_key() {
            std::result::Result::Ok(std::option::Option::Some(key)) => {
                if key.len() > 10 {
                    std::format!("{}...{}", &key[..std::cmp::min(3, key.len())], &key[key.len().saturating_sub(6)..])
                } else {
                    std::string::String::from("***")
                }
            }
            std::result::Result::Ok(std::option::Option::None) => std::string::String::from("(not required)"),
            std::result::Result::Err(_) => std::string::String::from("(not set)"),
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::OpenAI => write!(f, "OpenAI"),
            ProviderType::Anthropic => write!(f, "Anthropic"),
            ProviderType::Ollama => write!(f, "Ollama"),
            ProviderType::Mistral => write!(f, "Mistral"),
            ProviderType::Groq => write!(f, "Groq"),
            ProviderType::Cohere => write!(f, "Cohere"),
            ProviderType::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_display() {
        // Test: Validates ProviderType Display implementation.
        // Justification: UI needs human-readable provider names.
        std::assert_eq!(ProviderType::OpenAI.to_string(), "OpenAI");
        std::assert_eq!(ProviderType::Anthropic.to_string(), "Anthropic");
        std::assert_eq!(ProviderType::Custom(std::string::String::from("MyProvider")).to_string(), "MyProvider");
    }

    #[test]
    fn test_masked_api_key() {
        // Test: Validates API key masking for security.
        // Justification: Must not expose full keys in UI.
        let provider = ProviderConfig {
            provider_type: ProviderType::OpenAI,
            base_url: std::string::String::from("https://api.openai.com/v1"),
            api_key_env: std::option::Option::None,
            timeout_seconds: 60,
            max_retries: 3,
            default_model: std::string::String::from("gpt-4o-mini"),
        };

        let masked = provider.get_masked_api_key();
        std::assert_eq!(masked, "(not required)");
    }

    #[test]
    fn test_has_api_key_no_requirement() {
        // Test: Validates has_api_key for providers without auth.
        // Justification: Ollama and local providers don't need keys.
        let provider = ProviderConfig {
            provider_type: ProviderType::Ollama,
            base_url: std::string::String::from("http://localhost:11434"),
            api_key_env: std::option::Option::None,
            timeout_seconds: 120,
            max_retries: 2,
            default_model: std::string::String::from("llama3.2"),
        };

        std::assert!(provider.has_api_key());
    }
}
