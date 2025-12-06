//! Error types for Rigger configuration.
//!
//! Provides detailed, actionable error messages for config validation failures,
//! missing API keys, and migration issues.
//!
//! Revision History
//! - 2025-12-03T07:55:00Z @AI: Create ConfigError for rigger_core (Phase 2.2 of CONFIG-MODERN-20251203).

use super::ProviderType;

/// Configuration errors with helpful context.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Missing API key for provider
    #[error("Missing API key for {provider:?}: Please set environment variable {env_var}")]
    MissingApiKey {
        provider: ProviderType,
        env_var: std::string::String,
    },

    /// Task slot references non-existent provider
    #[error("Invalid provider '{provider}' in task slot '{slot}'. Available providers: {available:?}")]
    UnknownProvider {
        slot: std::string::String,
        provider: std::string::String,
        available: std::vec::Vec<std::string::String>,
    },

    /// Invalid base URL format
    #[error("Invalid base URL for provider {provider}: {url}")]
    InvalidBaseUrl {
        provider: std::string::String,
        url: std::string::String,
    },

    /// Config file not found
    #[error("Config file not found: {path}\nError: {error}")]
    FileNotFound {
        path: std::string::String,
        error: std::string::String,
    },

    /// Failed to parse config JSON
    #[error("Failed to parse config: {message}")]
    ParseError {
        message: std::string::String,
    },

    /// Migration from legacy format failed
    #[error("Failed to migrate config from version {from_version}: {message}")]
    MigrationError {
        from_version: std::string::String,
        message: std::string::String,
    },
}
