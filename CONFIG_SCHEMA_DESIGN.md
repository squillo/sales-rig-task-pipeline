# Rigger Configuration Schema Design v3.0

**Date**: 2025-12-03
**Status**: Draft
**Authors**: @AI with user approval

---

## Overview

This document defines the canonical configuration structure for Rigger v3.0, supporting multiple LLM providers, API key management, and extensible task slots.

---

## JSON Schema Example

```json
{
  "version": "3.0",
  "database": {
    "url": "sqlite:.rigger/tasks.db",
    "auto_vacuum": true,
    "pool_size": 5
  },
  "providers": {
    "openai": {
      "type": "OpenAI",
      "base_url": "https://api.openai.com/v1",
      "api_key_env": "OPENAI_API_KEY",
      "timeout_seconds": 60,
      "max_retries": 3,
      "default_model": "gpt-4o-mini"
    },
    "anthropic": {
      "type": "Anthropic",
      "base_url": "https://api.anthropic.com/v1",
      "api_key_env": "ANTHROPIC_API_KEY",
      "timeout_seconds": 60,
      "max_retries": 3,
      "default_model": "claude-sonnet-4-5"
    },
    "ollama": {
      "type": "Ollama",
      "base_url": "http://localhost:11434",
      "timeout_seconds": 120,
      "max_retries": 2,
      "default_model": "llama3.2"
    },
    "groq": {
      "type": "Groq",
      "base_url": "https://api.groq.com/openai/v1",
      "api_key_env": "GROQ_API_KEY",
      "timeout_seconds": 30,
      "max_retries": 3,
      "default_model": "llama-3.3-70b-versatile"
    }
  },
  "task_slots": {
    "main": {
      "provider": "ollama",
      "model": "llama3.2",
      "enabled": true,
      "description": "Primary task decomposition and generation"
    },
    "research": {
      "provider": "groq",
      "model": "llama-3.3-70b-versatile",
      "enabled": true,
      "description": "Web research and artifact search"
    },
    "fallback": {
      "provider": "ollama",
      "model": "llama3.2",
      "enabled": true,
      "description": "Fallback when main provider fails"
    },
    "embedding": {
      "provider": "ollama",
      "model": "nomic-embed-text",
      "enabled": true,
      "description": "Generate embeddings for semantic search"
    },
    "vision": {
      "provider": "ollama",
      "model": "llava:latest",
      "enabled": false,
      "description": "Image analysis and description"
    },
    "chat_agent": {
      "provider": "anthropic",
      "model": "claude-sonnet-4-5",
      "enabled": true,
      "description": "Interactive chat agent with tool calling",
      "streaming": true
    }
  },
  "performance": {
    "enable_metrics": true,
    "metrics_file": ".rigger/metrics.jsonl",
    "cache_embeddings": true,
    "max_concurrent_tasks": 4
  },
  "tui": {
    "theme": "default",
    "layout": "default",
    "auto_refresh_interval_ms": 30000,
    "show_notifications": true
  }
}
```

---

## Rust Type Definitions

### Core Config Structure

```rust
/// Main configuration structure for Rigger.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RiggerConfig {
    /// Config schema version for migration
    #[serde(default = "default_version")]
    pub version: String,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// LLM provider configurations (keyed by provider name)
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,

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

fn default_version() -> String {
    String::from("3.0")
}
```

### Provider Configuration

```rust
/// Configuration for a single LLM provider.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProviderConfig {
    /// Provider type (OpenAI, Anthropic, Ollama, etc.)
    #[serde(rename = "type")]
    pub provider_type: ProviderType,

    /// Base URL for API endpoint
    pub base_url: String,

    /// Environment variable name containing API key
    /// If None, provider doesn't require authentication (e.g., Ollama)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,

    /// Timeout in seconds for requests
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    /// Default model to use if not specified in task slot
    pub default_model: String,
}

fn default_timeout() -> u64 {
    60
}

fn default_max_retries() -> usize {
    3
}

/// Supported LLM provider types.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Ollama,
    Mistral,
    Groq,
    Cohere,
    #[serde(untagged)]
    Custom(String),
}
```

### Task Slot Configuration

```rust
/// Configuration for all task slots.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TaskSlotConfig {
    /// Primary task decomposition and generation
    #[serde(default = "default_main_slot")]
    pub main: TaskSlot,

    /// Web research and artifact search
    #[serde(default = "default_research_slot")]
    pub research: TaskSlot,

    /// Fallback when main provider fails
    #[serde(default = "default_fallback_slot")]
    pub fallback: TaskSlot,

    /// Generate embeddings for semantic search
    #[serde(default = "default_embedding_slot")]
    pub embedding: TaskSlot,

    /// Image analysis and description
    #[serde(default = "default_vision_slot")]
    pub vision: TaskSlot,

    /// Interactive chat agent with tool calling
    #[serde(default = "default_chat_agent_slot")]
    pub chat_agent: TaskSlot,
}

/// Configuration for a single task slot.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TaskSlot {
    /// Provider name (must exist in providers HashMap)
    pub provider: String,

    /// Model name for this slot
    pub model: String,

    /// Whether this slot is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Human-readable description
    pub description: String,

    /// Enable streaming responses (for chat_agent)
    #[serde(default = "default_false", skip_serializing_if = "is_false")]
    pub streaming: Option<bool>,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn is_false(b: &Option<bool>) -> bool {
    b.is_none() || !b.unwrap()
}
```

### Database Configuration

```rust
/// Database configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DatabaseConfig {
    /// Database URL (SQLite or PostgreSQL)
    #[serde(default = "default_db_url")]
    pub url: String,

    /// Enable auto-vacuum for SQLite
    #[serde(default = "default_true")]
    pub auto_vacuum: bool,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
}

fn default_db_url() -> String {
    String::from("sqlite:.rigger/tasks.db")
}

fn default_pool_size() -> usize {
    5
}
```

### Performance Configuration

```rust
/// Performance and monitoring configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PerformanceConfig {
    /// Enable performance metrics collection
    #[serde(default = "default_true")]
    pub enable_metrics: bool,

    /// Path to metrics output file
    #[serde(default = "default_metrics_file")]
    pub metrics_file: String,

    /// Cache embeddings to avoid regeneration
    #[serde(default = "default_true")]
    pub cache_embeddings: bool,

    /// Maximum concurrent tasks
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_tasks: usize,
}

fn default_metrics_file() -> String {
    String::from(".rigger/metrics.jsonl")
}

fn default_max_concurrent() -> usize {
    4
}
```

### TUI Configuration

```rust
/// TUI-specific configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TuiConfig {
    /// Color theme
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Layout preset
    #[serde(default = "default_layout")]
    pub layout: String,

    /// Auto-refresh interval in milliseconds
    #[serde(default = "default_refresh_interval")]
    pub auto_refresh_interval_ms: u64,

    /// Show notification panel
    #[serde(default = "default_true")]
    pub show_notifications: bool,
}

fn default_theme() -> String {
    String::from("default")
}

fn default_layout() -> String {
    String::from("default")
}

fn default_refresh_interval() -> u64 {
    30000
}
```

---

## API Key Management

### Security Principles

1. **NEVER store API keys in config.json**
2. **ALWAYS use environment variables**
3. **Mask keys in UI** (show `sk-...abc123`)
4. **Validate env vars exist** before attempting API calls

### Environment Variable Convention

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic/Claude
export ANTHROPIC_API_KEY="sk-ant-..."

# Groq
export GROQ_API_KEY="gsk_..."

# Mistral
export MISTRAL_API_KEY="..."

# Cohere
export COHERE_API_KEY="..."
```

### Runtime API Key Resolution

```rust
impl ProviderConfig {
    /// Get the API key from environment variable.
    pub fn get_api_key(&self) -> Result<Option<String>, ConfigError> {
        match &self.api_key_env {
            Some(env_var) => {
                match std::env::var(env_var) {
                    Ok(key) => Ok(Some(key)),
                    Err(_) => Err(ConfigError::MissingApiKey {
                        provider: self.provider_type.clone(),
                        env_var: env_var.clone(),
                    }),
                }
            }
            None => Ok(None), // Provider doesn't need API key (e.g., Ollama)
        }
    }

    /// Check if API key is available without retrieving it.
    pub fn has_api_key(&self) -> bool {
        match &self.api_key_env {
            Some(env_var) => std::env::var(env_var).is_ok(),
            None => true, // No key required
        }
    }

    /// Get masked API key for display (e.g., "sk-...abc123").
    pub fn get_masked_api_key(&self) -> String {
        match self.get_api_key() {
            Ok(Some(key)) => {
                if key.len() > 10 {
                    format!("{}...{}", &key[..3], &key[key.len()-6..])
                } else {
                    String::from("***")
                }
            }
            Ok(None) => String::from("(not required)"),
            Err(_) => String::from("(not set)"),
        }
    }
}
```

---

## Config Validation

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing API key for {provider:?}: Please set environment variable {env_var}")]
    MissingApiKey {
        provider: ProviderType,
        env_var: String,
    },

    #[error("Invalid provider '{provider}' in task slot '{slot}'. Available providers: {available:?}")]
    UnknownProvider {
        slot: String,
        provider: String,
        available: Vec<String>,
    },

    #[error("Invalid base URL for provider {provider}: {url}")]
    InvalidBaseUrl {
        provider: String,
        url: String,
    },

    #[error("Config file not found: {path}")]
    FileNotFound {
        path: String,
    },

    #[error("Failed to parse config: {message}")]
    ParseError {
        message: String,
    },
}

impl RiggerConfig {
    /// Validate configuration and return helpful errors.
    pub fn validate(&self) -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();

        // Validate all task slots reference existing providers
        let provider_names: Vec<String> = self.providers.keys().cloned().collect();

        for (slot_name, slot) in [
            ("main", &self.task_slots.main),
            ("research", &self.task_slots.research),
            ("fallback", &self.task_slots.fallback),
            ("embedding", &self.task_slots.embedding),
            ("vision", &self.task_slots.vision),
            ("chat_agent", &self.task_slots.chat_agent),
        ] {
            if slot.enabled && !self.providers.contains_key(&slot.provider) {
                errors.push(ConfigError::UnknownProvider {
                    slot: slot_name.to_string(),
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

        // Validate API keys for enabled providers
        for (name, provider) in &self.providers {
            if let Some(env_var) = &provider.api_key_env {
                if std::env::var(env_var).is_err() {
                    // Check if any enabled slot uses this provider
                    let is_used = [
                        &self.task_slots.main,
                        &self.task_slots.research,
                        &self.task_slots.fallback,
                        &self.task_slots.embedding,
                        &self.task_slots.vision,
                        &self.task_slots.chat_agent,
                    ].iter().any(|slot| slot.enabled && slot.provider == *name);

                    if is_used {
                        errors.push(ConfigError::MissingApiKey {
                            provider: provider.provider_type.clone(),
                            env_var: env_var.clone(),
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

---

## Migration Strategy

### Version Detection

```rust
impl RiggerConfig {
    pub fn detect_version(raw: &serde_json::Value) -> ConfigVersion {
        if let Some(version) = raw.get("version").and_then(|v| v.as_str()) {
            match version {
                "3.0" => ConfigVersion::V3,
                _ => ConfigVersion::Unknown,
            }
        } else if raw.get("task_tools").is_some() {
            ConfigVersion::V2 // Setup wizard format
        } else if raw.get("model_roles").is_some() {
            ConfigVersion::V1 // OrchestratorConfig format
        } else {
            ConfigVersion::V0 // Legacy simple format
        }
    }
}

pub enum ConfigVersion {
    V0, // Legacy: { "provider": "ollama", "model": {...} }
    V1, // OrchestratorConfig: { "model_roles": {...}, "providers": {...} }
    V2, // Setup wizard: { "task_tools": {...} }
    V3, // Current: { "version": "3.0", "providers": {...}, "task_slots": {...} }
    Unknown,
}
```

### Default Configurations

```rust
impl Default for RiggerConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();

        // Default to Ollama (no API key required)
        providers.insert("ollama".to_string(), ProviderConfig {
            provider_type: ProviderType::Ollama,
            base_url: "http://localhost:11434".to_string(),
            api_key_env: None,
            timeout_seconds: 120,
            max_retries: 2,
            default_model: "llama3.2".to_string(),
        });

        Self {
            version: "3.0".to_string(),
            database: DatabaseConfig::default(),
            providers,
            task_slots: TaskSlotConfig::default(),
            performance: PerformanceConfig::default(),
            tui: TuiConfig::default(),
        }
    }
}
```

---

## Next Steps

1. ✅ Design document complete
2. ⏳ Create `rigger_core` crate
3. ⏳ Implement all config types
4. ⏳ Implement migration logic
5. ⏳ Update config editor UI
6. ⏳ Integration testing

---

## Notes

- **Extensibility**: `ProviderType::Custom(String)` supports user-defined providers
- **Security**: API keys never touch disk, always in env vars
- **Validation**: Catch errors early with helpful messages
- **Migration**: Automatic upgrade from old formats
- **Future-proof**: Easy to add new providers and features
