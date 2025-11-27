//! Configuration management for Rigger orchestrator.
//!
//! Loads configuration from `.rigger/config.json` to customize model roles,
//! quantization settings, and other orchestrator behavior. Falls back to
//! sensible defaults if the file is missing or malformed.
//!
//! Revision History
//! - 2025-11-23T23:30:00Z @AI: Create config module for heterogeneous pipeline (Phase 5 Sprint 10 Task 5.6).

/// Orchestrator configuration loaded from `.rigger/config.json`.
///
/// This structure defines all configurable aspects of the Rigger orchestrator,
/// including model role assignments for the heterogeneous agent pipeline.
///
/// # Examples
///
/// ```
/// use task_orchestrator::infrastructure::config::OrchestratorConfig;
///
/// let config = OrchestratorConfig::load_from_rigger_dir(".rigger").unwrap();
/// let router_model = config.model_roles.get("router").unwrap();
/// std::assert_eq!(router_model, "phi3");
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct OrchestratorConfig {
    /// Model assignments for each orchestration role
    #[serde(default = "default_model_roles")]
    pub model_roles: std::collections::HashMap<String, String>,

    /// Quantization settings per model
    #[serde(default = "default_quantization")]
    pub quantization: std::collections::HashMap<String, String>,

    /// Provider settings (default provider, URLs, etc.)
    #[serde(default = "default_providers")]
    pub providers: ProviderConfig,

    /// Performance monitoring settings
    #[serde(default = "default_performance")]
    pub performance: PerformanceConfig,

    /// TUI settings
    #[serde(default = "default_tui")]
    pub tui: TuiConfig,
}

/// Provider configuration section.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProviderConfig {
    #[serde(default = "default_provider")]
    pub default: String,

    #[serde(default = "default_ollama_url")]
    pub ollama_base_url: String,
}

/// Performance monitoring configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PerformanceConfig {
    #[serde(default = "default_enable_metrics")]
    pub enable_metrics: bool,

    #[serde(default = "default_metrics_file")]
    pub metrics_file: String,
}

/// TUI configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TuiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_layout")]
    pub layout: String,

    #[serde(default = "default_refresh_interval")]
    pub auto_refresh_interval_ms: u64,
}

// Default value functions for serde
fn default_model_roles() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert(String::from("router"), String::from("phi3"));
    map.insert(String::from("decomposer"), String::from("orca2"));
    map.insert(String::from("enhancer"), String::from("llama3.1"));
    map.insert(String::from("tester"), String::from("mistral"));
    map
}

fn default_quantization() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert(String::from("phi3"), String::from("Q4_K_M"));
    map.insert(String::from("orca2"), String::from("Q5_K_M"));
    map.insert(String::from("llama3.1"), String::from("Q4_K_M"));
    map.insert(String::from("mistral"), String::from("Q4_K_M"));
    map
}

fn default_providers() -> ProviderConfig {
    ProviderConfig {
        default: String::from("ollama"),
        ollama_base_url: String::from("http://localhost:11434"),
    }
}

fn default_performance() -> PerformanceConfig {
    PerformanceConfig {
        enable_metrics: true,
        metrics_file: String::from(".rigger/metrics.jsonl"),
    }
}

fn default_tui() -> TuiConfig {
    TuiConfig {
        theme: String::from("default"),
        layout: String::from("default"),
        auto_refresh_interval_ms: 30000,
    }
}

fn default_provider() -> String {
    String::from("ollama")
}

fn default_ollama_url() -> String {
    String::from("http://localhost:11434")
}

fn default_enable_metrics() -> bool {
    true
}

fn default_metrics_file() -> String {
    String::from(".rigger/metrics.jsonl")
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

impl OrchestratorConfig {
    /// Loads configuration from the specified `.rigger` directory.
    ///
    /// If `config.json` exists, it will be parsed. Missing fields use defaults.
    /// If the file doesn't exist or is malformed, returns a default configuration.
    ///
    /// # Arguments
    ///
    /// * `rigger_dir` - Path to the `.rigger` directory
    ///
    /// # Returns
    ///
    /// OrchestratorConfig with loaded or default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::infrastructure::config::OrchestratorConfig;
    ///
    /// let config = OrchestratorConfig::load_from_rigger_dir(".rigger").unwrap();
    /// std::println!("Router model: {}", config.model_roles.get("router").unwrap());
    /// ```
    pub fn load_from_rigger_dir(rigger_dir: &str) -> std::result::Result<Self, String> {
        let config_path = std::path::Path::new(rigger_dir).join("config.json");

        // If file doesn't exist, return default config
        if !config_path.exists() {
            return std::result::Result::Ok(Self::default());
        }

        // Read and parse config file
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| std::format!("Failed to read config.json: {}", e))?;

        let config: OrchestratorConfig = serde_json::from_str(&content)
            .map_err(|e| std::format!("Failed to parse config.json: {}", e))?;

        std::result::Result::Ok(config)
    }

    /// Converts the model_roles HashMap into a ModelSelectionStrategy.
    ///
    /// This allows the configuration to drive the heterogeneous pipeline's
    /// model selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::infrastructure::config::OrchestratorConfig;
    /// use task_orchestrator::domain::model_role::ModelRole;
    ///
    /// let config = OrchestratorConfig::default();
    /// let strategy = config.to_model_selection_strategy();
    /// std::assert_eq!(strategy.select_model_for_role(ModelRole::Router), "phi3");
    /// ```
    pub fn to_model_selection_strategy(&self) -> crate::domain::model_role::ModelSelectionStrategy {
        crate::domain::model_role::ModelSelectionStrategy::from_config(self.model_roles.clone())
    }
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            model_roles: default_model_roles(),
            quantization: default_quantization(),
            providers: default_providers(),
            performance: default_performance(),
            tui: default_tui(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_default_config() {
        // Test: Validates default configuration values.
        // Justification: Ensures sensible defaults when config file is missing.
        let config = super::OrchestratorConfig::default();

        std::assert_eq!(config.model_roles.get("router").unwrap(), "phi3");
        std::assert_eq!(config.model_roles.get("decomposer").unwrap(), "orca2");
        std::assert_eq!(config.model_roles.get("enhancer").unwrap(), "llama3.1");
        std::assert_eq!(config.model_roles.get("tester").unwrap(), "mistral");

        std::assert_eq!(config.providers.default, "ollama");
        std::assert_eq!(config.performance.enable_metrics, true);
        std::assert_eq!(config.tui.theme, "default");
    }

    #[test]
    fn test_to_model_selection_strategy() {
        // Test: Validates conversion to ModelSelectionStrategy.
        // Justification: Configuration must drive heterogeneous pipeline model selection.
        let config = super::OrchestratorConfig::default();
        let strategy = config.to_model_selection_strategy();

        std::assert_eq!(
            strategy.select_model_for_role(crate::domain::model_role::ModelRole::Router),
            "phi3"
        );
        std::assert_eq!(
            strategy.select_model_for_role(crate::domain::model_role::ModelRole::Decomposer),
            "orca2"
        );
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        // Test: Validates loading from non-existent directory returns defaults.
        // Justification: Missing config should not crash, but use sensible defaults.
        let result = super::OrchestratorConfig::load_from_rigger_dir("/nonexistent/path");
        std::assert!(result.is_ok());

        let config = result.unwrap();
        std::assert_eq!(config.model_roles.get("router").unwrap(), "phi3");
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        // Test: Validates JSON serialization/deserialization preserves data.
        // Justification: Config must persist correctly to disk.
        let config = super::OrchestratorConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        let parsed: super::OrchestratorConfig = serde_json::from_str(&json).unwrap();
        std::assert_eq!(parsed.model_roles.get("router").unwrap(), "phi3");
        std::assert_eq!(parsed.quantization.get("orca2").unwrap(), "Q5_K_M");
    }
}
