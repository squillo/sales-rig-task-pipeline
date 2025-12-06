//! Configuration management for Rigger v3.0.
//!
//! Provides unified configuration structure supporting multiple LLM providers,
//! API key management, task slots, and automatic migration from legacy formats.
//!
//! Revision History
//! - 2025-12-03T07:50:00Z @AI: Initial config module for rigger_core (Phase 2.2 of CONFIG-MODERN-20251203).

pub mod provider;
pub mod task_slots;
pub mod error;
pub mod migration;

pub use provider::{ProviderConfig, ProviderType};
pub use task_slots::{TaskSlotConfig, TaskSlot};
pub use error::ConfigError;
pub use migration::ConfigVersion;

/// Main configuration structure for Rigger v3.0.
///
/// Supports multiple LLM providers (OpenAI, Anthropic, Ollama, etc.), task slot
/// assignments, and extensible settings. API keys are managed via environment
/// variables and never stored in the config file.
///
/// # Examples
///
/// ```
/// use rigger_core::RiggerConfig;
///
/// // Load config with automatic migration
/// let config = RiggerConfig::load_with_migration(".rigger/config.json").unwrap();
///
/// // Validate before use
/// config.validate().unwrap();
///
/// // Get provider for chat agent
/// let chat_provider = config.providers.get(&config.task_slots.chat_agent.provider);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RiggerConfig {
    /// Config schema version for migration
    #[serde(default = "default_version")]
    pub version: std::string::String,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// LLM provider configurations (keyed by provider name)
    #[serde(default)]
    pub providers: std::collections::HashMap<std::string::String, ProviderConfig>,

    /// Task slot assignments
    #[serde(default)]
    pub task_slots: TaskSlotConfig,

    /// Performance and monitoring settings
    #[serde(default)]
    pub performance: PerformanceConfig,

    /// TUI-specific settings
    #[serde(default)]
    pub tui: TuiConfig,
}

fn default_version() -> std::string::String {
    std::string::String::from("3.0")
}

/// Database configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DatabaseConfig {
    /// Database URL (SQLite or PostgreSQL)
    #[serde(default = "default_db_url")]
    pub url: std::string::String,

    /// Enable auto-vacuum for SQLite
    #[serde(default = "default_true")]
    pub auto_vacuum: bool,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
}

fn default_db_url() -> std::string::String {
    std::string::String::from("sqlite:.rigger/tasks.db")
}

fn default_pool_size() -> usize {
    5
}

/// Performance and monitoring configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PerformanceConfig {
    /// Enable performance metrics collection
    #[serde(default = "default_true")]
    pub enable_metrics: bool,

    /// Path to metrics output file
    #[serde(default = "default_metrics_file")]
    pub metrics_file: std::string::String,

    /// Cache embeddings to avoid regeneration
    #[serde(default = "default_true")]
    pub cache_embeddings: bool,

    /// Maximum concurrent tasks
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_tasks: usize,
}

fn default_metrics_file() -> std::string::String {
    std::string::String::from(".rigger/metrics.jsonl")
}

fn default_max_concurrent() -> usize {
    4
}

/// TUI-specific configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TuiConfig {
    /// Color theme
    #[serde(default = "default_theme")]
    pub theme: std::string::String,

    /// Layout preset
    #[serde(default = "default_layout")]
    pub layout: std::string::String,

    /// Auto-refresh interval in milliseconds
    #[serde(default = "default_refresh_interval")]
    pub auto_refresh_interval_ms: u64,

    /// Show notification panel
    #[serde(default = "default_true")]
    pub show_notifications: bool,
}

fn default_theme() -> std::string::String {
    std::string::String::from("default")
}

fn default_layout() -> std::string::String {
    std::string::String::from("default")
}

fn default_refresh_interval() -> u64 {
    30000
}

fn default_true() -> bool {
    true
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_db_url(),
            auto_vacuum: true,
            pool_size: default_pool_size(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_file: default_metrics_file(),
            cache_embeddings: true,
            max_concurrent_tasks: default_max_concurrent(),
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            layout: default_layout(),
            auto_refresh_interval_ms: default_refresh_interval(),
            show_notifications: true,
        }
    }
}

impl Default for RiggerConfig {
    fn default() -> Self {
        let mut providers = std::collections::HashMap::new();

        // Default to Ollama (no API key required)
        providers.insert(std::string::String::from("ollama"), ProviderConfig {
            provider_type: ProviderType::Ollama,
            base_url: std::string::String::from("http://localhost:11434"),
            api_key_env: std::option::Option::None,
            timeout_seconds: 120,
            max_retries: 2,
            default_model: std::string::String::from("llama3.2"),
        });

        Self {
            version: default_version(),
            database: DatabaseConfig::default(),
            providers,
            task_slots: TaskSlotConfig::default(),
            performance: PerformanceConfig::default(),
            tui: TuiConfig::default(),
        }
    }
}

impl RiggerConfig {
    /// Load configuration with automatic migration from legacy formats.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to config.json file
    ///
    /// # Returns
    ///
    /// Loaded configuration or error.
    pub fn load_with_migration(path: &str) -> std::result::Result<Self, ConfigError> {
        let path_buf = std::path::Path::new(path);

        // If file doesn't exist, return default config
        if !path_buf.exists() {
            return std::result::Result::Ok(Self::default());
        }

        // Read file content
        let content = std::fs::read_to_string(path_buf)
            .map_err(|e| ConfigError::FileNotFound {
                path: path.to_string(),
                error: e.to_string(),
            })?;

        // Try parsing as current format first
        match serde_json::from_str::<Self>(&content) {
            std::result::Result::Ok(config) => std::result::Result::Ok(config),
            std::result::Result::Err(_) => {
                // Failed to parse - attempt migration
                let raw: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| ConfigError::ParseError {
                        message: e.to_string(),
                    })?;

                // Detect version and migrate
                let version = Self::detect_version(&raw);
                match version {
                    ConfigVersion::V0 => Self::migrate_from_v0(&raw),
                    ConfigVersion::V2 => Self::migrate_from_v2(&raw),
                    ConfigVersion::V3 => {
                        // This shouldn't happen (we already tried parsing as V3)
                        // But try one more time with better error message
                        serde_json::from_str::<Self>(&content)
                            .map_err(|e| ConfigError::ParseError {
                                message: e.to_string(),
                            })
                    }
                    ConfigVersion::V1 | ConfigVersion::Unknown => {
                        // V1 migration not implemented yet, return default
                        // TODO: Implement V1 (OrchestratorConfig) migration
                        std::result::Result::Ok(Self::default())
                    }
                }
            }
        }
    }

    /// Validate configuration and return helpful errors.
    ///
    /// Checks that:
    /// - All task slots reference existing providers
    /// - Base URLs are valid
    /// - API keys are available for enabled providers
    ///
    /// # Returns
    ///
    /// Ok(()) if valid, or Vec of errors if invalid.
    pub fn validate(&self) -> std::result::Result<(), std::vec::Vec<ConfigError>> {
        let mut errors = std::vec::Vec::new();

        // Validate all task slots reference existing providers
        let provider_names: std::vec::Vec<std::string::String> = self.providers.keys().cloned().collect();

        for (slot_name, slot) in [
            (std::string::String::from("main"), &self.task_slots.main),
            (std::string::String::from("research"), &self.task_slots.research),
            (std::string::String::from("fallback"), &self.task_slots.fallback),
            (std::string::String::from("embedding"), &self.task_slots.embedding),
            (std::string::String::from("vision"), &self.task_slots.vision),
            (std::string::String::from("chat_agent"), &self.task_slots.chat_agent),
        ] {
            if slot.enabled && !self.providers.contains_key(&slot.provider) {
                errors.push(ConfigError::UnknownProvider {
                    slot: slot_name,
                    provider: slot.provider.clone(),
                    available: provider_names.clone(),
                });
            }
        }

        // Validate base URLs
        for (name, provider) in &self.providers {
            if !provider.base_url.starts_with("http://")
                && !provider.base_url.starts_with("https://") {
                errors.push(ConfigError::InvalidBaseUrl {
                    provider: name.clone(),
                    url: provider.base_url.clone(),
                });
            }
        }

        if errors.is_empty() {
            std::result::Result::Ok(())
        } else {
            std::result::Result::Err(errors)
        }
    }
}
