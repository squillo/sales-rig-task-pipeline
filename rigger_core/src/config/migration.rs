//! Configuration migration from legacy formats to v3.0.
//!
//! Handles automatic migration from three legacy config formats:
//! - V0: Simple format with provider and model fields
//! - V1: OrchestratorConfig format with model_roles
//! - V2: Setup wizard format with task_tools
//!
//! Revision History
//! - 2025-12-03T08:15:00Z @AI: Create migration module for rigger_core (Phase 2.3 of CONFIG-MODERN-20251203).

use super::{RiggerConfig, ProviderConfig, ProviderType, TaskSlot, TaskSlotConfig, DatabaseConfig, PerformanceConfig, TuiConfig};
use super::error::ConfigError;

/// Configuration version for migration detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigVersion {
    /// V0: Legacy simple format { "provider": "ollama", "model": {...} }
    V0,
    /// V1: OrchestratorConfig { "model_roles": {...}, "providers": {...} }
    V1,
    /// V2: Setup wizard { "task_tools": {...} }
    V2,
    /// V3: Current format { "version": "3.0", "providers": {...}, "task_slots": {...} }
    V3,
    /// Unknown version
    Unknown,
}

impl RiggerConfig {
    /// Detect configuration version from raw JSON.
    ///
    /// # Arguments
    ///
    /// * `raw` - Parsed JSON value
    ///
    /// # Returns
    ///
    /// Detected config version.
    pub fn detect_version(raw: &serde_json::Value) -> ConfigVersion {
        if let std::option::Option::Some(version) = raw.get("version").and_then(|v| v.as_str()) {
            match version {
                "3.0" => ConfigVersion::V3,
                _ => ConfigVersion::Unknown,
            }
        } else if raw.get("task_tools").is_some() {
            ConfigVersion::V2 // Setup wizard format
        } else if raw.get("model_roles").is_some() {
            ConfigVersion::V1 // OrchestratorConfig format
        } else if raw.get("provider").is_some() || raw.get("model").is_some() {
            ConfigVersion::V0 // Legacy simple format
        } else {
            ConfigVersion::Unknown
        }
    }

    /// Migrate from V0 (legacy simple format).
    ///
    /// Format: `{ "provider": "ollama", "model": { "main": "llama3.2", ... }, "database_url": "..." }`
    pub fn migrate_from_v0(raw: &serde_json::Value) -> std::result::Result<Self, ConfigError> {
        let provider_name = raw.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("ollama");

        let mut providers = std::collections::HashMap::new();

        // Create provider config
        let provider_type = match provider_name {
            "openai" => ProviderType::OpenAI,
            "anthropic" => ProviderType::Anthropic,
            "ollama" => ProviderType::Ollama,
            "mistral" => ProviderType::Mistral,
            "groq" => ProviderType::Groq,
            other => ProviderType::Custom(other.to_string()),
        };

        let (base_url, api_key_env, timeout) = match provider_type {
            ProviderType::OpenAI => (
                "https://api.openai.com/v1".to_string(),
                std::option::Option::Some("OPENAI_API_KEY".to_string()),
                60,
            ),
            ProviderType::Anthropic => (
                "https://api.anthropic.com/v1".to_string(),
                std::option::Option::Some("ANTHROPIC_API_KEY".to_string()),
                60,
            ),
            ProviderType::Ollama => (
                "http://localhost:11434".to_string(),
                std::option::Option::None,
                120,
            ),
            ProviderType::Groq => (
                "https://api.groq.com/openai/v1".to_string(),
                std::option::Option::Some("GROQ_API_KEY".to_string()),
                30,
            ),
            _ => (
                "http://localhost:11434".to_string(),
                std::option::Option::None,
                120,
            ),
        };

        providers.insert(provider_name.to_string(), ProviderConfig {
            provider_type,
            base_url,
            api_key_env,
            timeout_seconds: timeout,
            max_retries: 3,
            default_model: raw.get("model")
                .and_then(|m| m.get("main"))
                .and_then(|v| v.as_str())
                .unwrap_or("llama3.2")
                .to_string(),
        });

        // Create task slots from model field
        let models = raw.get("model");
        let main_model = models.and_then(|m| m.get("main")).and_then(|v| v.as_str()).unwrap_or("llama3.2");
        let research_model = models.and_then(|m| m.get("research")).and_then(|v| v.as_str()).unwrap_or(main_model);
        let fallback_model = models.and_then(|m| m.get("fallback")).and_then(|v| v.as_str()).unwrap_or(main_model);

        let task_slots = TaskSlotConfig {
            main: TaskSlot {
                provider: provider_name.to_string(),
                model: main_model.to_string(),
                enabled: true,
                description: "Primary task decomposition and generation".to_string(),
                streaming: std::option::Option::None,
            },
            research: TaskSlot {
                provider: provider_name.to_string(),
                model: research_model.to_string(),
                enabled: true,
                description: "Web research and artifact search".to_string(),
                streaming: std::option::Option::None,
            },
            fallback: TaskSlot {
                provider: provider_name.to_string(),
                model: fallback_model.to_string(),
                enabled: true,
                description: "Fallback when main provider fails".to_string(),
                streaming: std::option::Option::None,
            },
            embedding: TaskSlot {
                provider: provider_name.to_string(),
                model: "nomic-embed-text".to_string(),
                enabled: true,
                description: "Generate embeddings for semantic search".to_string(),
                streaming: std::option::Option::None,
            },
            vision: TaskSlot {
                provider: provider_name.to_string(),
                model: "llava:latest".to_string(),
                enabled: false,
                description: "Image analysis and description".to_string(),
                streaming: std::option::Option::None,
            },
            chat_agent: TaskSlot {
                provider: provider_name.to_string(),
                model: main_model.to_string(),
                enabled: true,
                description: "Interactive chat agent with tool calling".to_string(),
                streaming: std::option::Option::Some(true),
            },
        };

        // Database config
        let database = DatabaseConfig {
            url: raw.get("database_url")
                .and_then(|v| v.as_str())
                .unwrap_or("sqlite:.rigger/tasks.db")
                .to_string(),
            auto_vacuum: true,
            pool_size: 5,
        };

        std::result::Result::Ok(Self {
            version: "3.0".to_string(),
            database,
            providers,
            task_slots,
            performance: PerformanceConfig::default(),
            tui: TuiConfig::default(),
        })
    }

    /// Migrate from V2 (setup wizard format).
    ///
    /// Format: `{ "task_tools": { "main": { "provider": "...", "model": "..." }, ... } }`
    pub fn migrate_from_v2(raw: &serde_json::Value) -> std::result::Result<Self, ConfigError> {
        let task_tools = raw.get("task_tools").ok_or_else(|| ConfigError::MigrationError {
            from_version: "v2".to_string(),
            message: "Missing task_tools field".to_string(),
        })?;

        let mut providers = std::collections::HashMap::new();
        let mut provider_configs = std::collections::HashMap::new();

        // Extract unique providers from task_tools
        for tool_name in ["main", "research", "fallback", "embedding", "vision"] {
            if let std::option::Option::Some(tool) = task_tools.get(tool_name) {
                if let std::option::Option::Some(provider_name) = tool.get("provider").and_then(|v| v.as_str()) {
                    if !provider_configs.contains_key(provider_name) {
                        let provider_type = match provider_name {
                            "openai" => ProviderType::OpenAI,
                            "anthropic" => ProviderType::Anthropic,
                            "ollama" => ProviderType::Ollama,
                            "mistral" => ProviderType::Mistral,
                            "groq" => ProviderType::Groq,
                            other => ProviderType::Custom(other.to_string()),
                        };

                        let (base_url, api_key_env, timeout) = match provider_type {
                            ProviderType::OpenAI => (
                                "https://api.openai.com/v1".to_string(),
                                std::option::Option::Some("OPENAI_API_KEY".to_string()),
                                60,
                            ),
                            ProviderType::Anthropic => (
                                "https://api.anthropic.com/v1".to_string(),
                                std::option::Option::Some("ANTHROPIC_API_KEY".to_string()),
                                60,
                            ),
                            ProviderType::Ollama => (
                                "http://localhost:11434".to_string(),
                                std::option::Option::None,
                                120,
                            ),
                            ProviderType::Groq => (
                                "https://api.groq.com/openai/v1".to_string(),
                                std::option::Option::Some("GROQ_API_KEY".to_string()),
                                30,
                            ),
                            _ => (
                                "http://localhost:11434".to_string(),
                                std::option::Option::None,
                                120,
                            ),
                        };

                        provider_configs.insert(provider_name.to_string(), (provider_type, base_url, api_key_env, timeout));
                    }
                }
            }
        }

        // Create provider configs
        for (name, (provider_type, base_url, api_key_env, timeout)) in provider_configs {
            providers.insert(name.clone(), ProviderConfig {
                provider_type,
                base_url,
                api_key_env,
                timeout_seconds: timeout,
                max_retries: 3,
                default_model: task_tools.get("main")
                    .and_then(|t| t.get("model"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("llama3.2")
                    .to_string(),
            });
        }

        // Helper to create task slot from tool
        let create_slot = |tool_name: &str, description: &str| -> TaskSlot {
            if let std::option::Option::Some(tool) = task_tools.get(tool_name) {
                TaskSlot {
                    provider: tool.get("provider")
                        .and_then(|v| v.as_str())
                        .unwrap_or("ollama")
                        .to_string(),
                    model: tool.get("model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("llama3.2")
                        .to_string(),
                    enabled: true,
                    description: description.to_string(),
                    streaming: std::option::Option::None,
                }
            } else {
                TaskSlot {
                    provider: "ollama".to_string(),
                    model: "llama3.2".to_string(),
                    enabled: false,
                    description: description.to_string(),
                    streaming: std::option::Option::None,
                }
            }
        };

        let task_slots = TaskSlotConfig {
            main: create_slot("main", "Primary task decomposition and generation"),
            research: create_slot("research", "Web research and artifact search"),
            fallback: create_slot("fallback", "Fallback when main provider fails"),
            embedding: create_slot("embedding", "Generate embeddings for semantic search"),
            vision: create_slot("vision", "Image analysis and description"),
            chat_agent: TaskSlot {
                provider: task_tools.get("main")
                    .and_then(|t| t.get("provider"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("ollama")
                    .to_string(),
                model: task_tools.get("main")
                    .and_then(|t| t.get("model"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("llama3.2")
                    .to_string(),
                enabled: true,
                description: "Interactive chat agent with tool calling".to_string(),
                streaming: std::option::Option::Some(true),
            },
        };

        // Database config
        let database = DatabaseConfig {
            url: raw.get("database_url")
                .and_then(|v| v.as_str())
                .unwrap_or("sqlite:.rigger/tasks.db")
                .to_string(),
            auto_vacuum: true,
            pool_size: 5,
        };

        std::result::Result::Ok(Self {
            version: "3.0".to_string(),
            database,
            providers,
            task_slots,
            performance: PerformanceConfig::default(),
            tui: TuiConfig::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_v0() {
        // Test: Validates V0 format detection.
        // Justification: Must correctly identify legacy simple format.
        let json = serde_json::json!({
            "provider": "ollama",
            "model": {
                "main": "llama3.2",
                "research": "llama3.2"
            }
        });

        std::assert_eq!(RiggerConfig::detect_version(&json), ConfigVersion::V0);
    }

    #[test]
    fn test_detect_v2() {
        // Test: Validates V2 format detection.
        // Justification: Must correctly identify setup wizard format.
        let json = serde_json::json!({
            "task_tools": {
                "main": {
                    "provider": "ollama",
                    "model": "llama3.2"
                }
            }
        });

        std::assert_eq!(RiggerConfig::detect_version(&json), ConfigVersion::V2);
    }

    #[test]
    fn test_detect_v3() {
        // Test: Validates V3 format detection.
        // Justification: Must correctly identify current format.
        let json = serde_json::json!({
            "version": "3.0",
            "providers": {}
        });

        std::assert_eq!(RiggerConfig::detect_version(&json), ConfigVersion::V3);
    }

    #[test]
    fn test_migrate_from_v0() {
        // Test: Validates migration from V0 format.
        // Justification: Existing users must upgrade seamlessly.
        let json = serde_json::json!({
            "provider": "ollama",
            "model": {
                "main": "llama3.2",
                "research": "llama3.2",
                "fallback": "llama3.2"
            },
            "database_url": "sqlite:.rigger/tasks.db"
        });

        let config = RiggerConfig::migrate_from_v0(&json).unwrap();

        std::assert_eq!(config.version, "3.0");
        std::assert!(config.providers.contains_key("ollama"));
        std::assert_eq!(config.task_slots.main.provider, "ollama");
        std::assert_eq!(config.task_slots.main.model, "llama3.2");
        std::assert_eq!(config.task_slots.chat_agent.streaming, std::option::Option::Some(true));
    }

    #[test]
    fn test_migrate_from_v2() {
        // Test: Validates migration from V2 (setup wizard) format.
        // Justification: New users created via wizard must upgrade.
        let json = serde_json::json!({
            "task_tools": {
                "main": {
                    "provider": "ollama",
                    "model": "llama3.2"
                },
                "embedding": {
                    "provider": "ollama",
                    "model": "nomic-embed-text"
                }
            },
            "database_url": "sqlite:.rigger/tasks.db"
        });

        let config = RiggerConfig::migrate_from_v2(&json).unwrap();

        std::assert_eq!(config.version, "3.0");
        std::assert!(config.providers.contains_key("ollama"));
        std::assert_eq!(config.task_slots.main.provider, "ollama");
        std::assert_eq!(config.task_slots.embedding.model, "nomic-embed-text");
        std::assert_eq!(config.task_slots.chat_agent.streaming, std::option::Option::Some(true));
    }
}
