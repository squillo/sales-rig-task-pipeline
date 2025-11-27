//! Defines roles for specialized models in the heterogeneous agent pipeline.
//!
//! Based on CONTEXT7_RESEARCH.md insights, different orchestration stages benefit
//! from different specialized models. This module provides the ModelRole enum
//! for categorizing agent tasks and a ModelSelectionStrategy for choosing
//! the optimal model for each role.
//!
//! Revision History
//! - 2025-11-23T22:00:00Z @AI: Initial ModelRole implementation for heterogeneous pipeline (Phase 5 Sprint 10 Task 5.1).

/// Represents the role a model plays in the orchestration pipeline.
///
/// Each role has different performance requirements:
/// - Router: Fast classification, low latency (Phi-3-mini recommended)
/// - Decomposer: Complex reasoning, process imitation (Orca-2 recommended)
/// - Enhancer: General improvement, balanced capabilities (Mistral 7B / llama3.1)
/// - Tester: Comprehension test generation (Mistral 7B)
///
/// # Examples
///
/// ```
/// use task_orchestrator::domain::model_role::ModelRole;
///
/// let role = ModelRole::Router;
/// std::assert_eq!(role.recommended_model(), "phi3");
/// std::assert!(role.requires_fast_inference());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ModelRole {
    /// Fast routing and classification (complexity scoring, triage)
    Router,

    /// Complex reasoning for task decomposition
    Decomposer,

    /// General task enhancement and improvement
    Enhancer,

    /// Comprehension test generation
    Tester,
}

impl ModelRole {
    /// Returns the recommended model for this role based on CONTEXT7_RESEARCH.md.
    ///
    /// # Returns
    ///
    /// The default model name for this role.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::model_role::ModelRole;
    ///
    /// std::assert_eq!(ModelRole::Router.recommended_model(), "phi3");
    /// std::assert_eq!(ModelRole::Decomposer.recommended_model(), "orca2");
    /// ```
    pub fn recommended_model(&self) -> &'static str {
        match self {
            ModelRole::Router => "phi3",
            ModelRole::Decomposer => "orca2",
            ModelRole::Enhancer => "llama3.1",
            ModelRole::Tester => "mistral",
        }
    }

    /// Returns whether this role requires fast inference (low latency).
    ///
    /// # Returns
    ///
    /// `true` if this role is latency-sensitive.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::model_role::ModelRole;
    ///
    /// std::assert!(ModelRole::Router.requires_fast_inference());
    /// std::assert!(!ModelRole::Decomposer.requires_fast_inference());
    /// ```
    pub fn requires_fast_inference(&self) -> bool {
        match self {
            ModelRole::Router => true,
            ModelRole::Decomposer => false,
            ModelRole::Enhancer => false,
            ModelRole::Tester => false,
        }
    }

    /// Returns the priority level for this role (higher = more important).
    ///
    /// Used for resource allocation when running multiple models concurrently.
    ///
    /// # Returns
    ///
    /// Priority value from 1 (lowest) to 10 (highest).
    pub fn priority(&self) -> u8 {
        match self {
            ModelRole::Router => 10, // Highest priority - gates entire flow
            ModelRole::Decomposer => 8,
            ModelRole::Enhancer => 5,
            ModelRole::Tester => 3,
        }
    }

    /// Returns a human-readable description of this role.
    pub fn description(&self) -> &'static str {
        match self {
            ModelRole::Router => "Fast routing and complexity classification",
            ModelRole::Decomposer => "Complex reasoning for task decomposition",
            ModelRole::Enhancer => "General task enhancement and clarification",
            ModelRole::Tester => "Comprehension test generation and validation",
        }
    }
}

impl std::fmt::Display for ModelRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelRole::Router => write!(f, "Router"),
            ModelRole::Decomposer => write!(f, "Decomposer"),
            ModelRole::Enhancer => write!(f, "Enhancer"),
            ModelRole::Tester => write!(f, "Tester"),
        }
    }
}

/// Strategy for selecting models based on role and configuration.
///
/// Reads from .rigger/config.json `model_roles` section to determine
/// which model to use for each role, falling back to recommended defaults.
///
/// # Examples
///
/// ```
/// use task_orchestrator::domain::model_role::{ModelRole, ModelSelectionStrategy};
///
/// let strategy = ModelSelectionStrategy::default();
/// let model = strategy.select_model_for_role(ModelRole::Router);
/// std::assert_eq!(model, "phi3");
/// ```
#[derive(Debug, Clone)]
pub struct ModelSelectionStrategy {
    /// Model assignments for each role (loaded from config)
    role_assignments: std::collections::HashMap<ModelRole, String>,
}

impl ModelSelectionStrategy {
    /// Creates a new strategy with recommended defaults.
    ///
    /// # Returns
    ///
    /// A strategy using the recommended models for each role.
    pub fn new() -> Self {
        let mut role_assignments = std::collections::HashMap::new();
        role_assignments.insert(ModelRole::Router, String::from("phi3"));
        role_assignments.insert(ModelRole::Decomposer, String::from("orca2"));
        role_assignments.insert(ModelRole::Enhancer, String::from("llama3.1"));
        role_assignments.insert(ModelRole::Tester, String::from("mistral"));

        Self { role_assignments }
    }

    /// Loads model assignments from a configuration map.
    ///
    /// # Arguments
    ///
    /// * `config` - Map of role names to model names
    ///
    /// # Returns
    ///
    /// A strategy with custom model assignments, falling back to defaults for missing roles.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::model_role::{ModelSelectionStrategy};
    /// use std::collections::HashMap;
    ///
    /// let mut config = HashMap::new();
    /// config.insert(String::from("router"), String::from("phi3"));
    /// config.insert(String::from("decomposer"), String::from("orca2"));
    ///
    /// let strategy = ModelSelectionStrategy::from_config(config);
    /// ```
    pub fn from_config(config: std::collections::HashMap<String, String>) -> Self {
        let mut role_assignments = std::collections::HashMap::new();

        // Parse config keys to ModelRole
        for (role_str, model) in config {
            let role = match role_str.to_lowercase().as_str() {
                "router" => ModelRole::Router,
                "decomposer" => ModelRole::Decomposer,
                "enhancer" => ModelRole::Enhancer,
                "tester" => ModelRole::Tester,
                _ => continue, // Skip unknown roles
            };
            role_assignments.insert(role, model);
        }

        // Fill in missing roles with defaults
        let default = Self::new();
        for role in [ModelRole::Router, ModelRole::Decomposer, ModelRole::Enhancer, ModelRole::Tester] {
            if !role_assignments.contains_key(&role) {
                role_assignments.insert(role, default.role_assignments[&role].clone());
            }
        }

        Self { role_assignments }
    }

    /// Selects the appropriate model for a given role.
    ///
    /// # Arguments
    ///
    /// * `role` - The orchestration role requiring a model
    ///
    /// # Returns
    ///
    /// The model name configured for this role.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::model_role::{ModelRole, ModelSelectionStrategy};
    ///
    /// let strategy = ModelSelectionStrategy::default();
    /// std::assert_eq!(strategy.select_model_for_role(ModelRole::Router), "phi3");
    /// std::assert_eq!(strategy.select_model_for_role(ModelRole::Decomposer), "orca2");
    /// ```
    pub fn select_model_for_role(&self, role: ModelRole) -> &str {
        self.role_assignments.get(&role)
            .map(|s| s.as_str())
            .unwrap_or(role.recommended_model())
    }

    /// Returns all configured model assignments.
    pub fn get_all_assignments(&self) -> &std::collections::HashMap<ModelRole, String> {
        &self.role_assignments
    }
}

impl Default for ModelSelectionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_model_role_recommended_models() {
        // Test: Validates recommended models match CONTEXT7_RESEARCH.md insights.
        // Justification: Ensures heterogeneous pipeline uses optimal models per role.
        std::assert_eq!(super::ModelRole::Router.recommended_model(), "phi3");
        std::assert_eq!(super::ModelRole::Decomposer.recommended_model(), "orca2");
        std::assert_eq!(super::ModelRole::Enhancer.recommended_model(), "llama3.1");
        std::assert_eq!(super::ModelRole::Tester.recommended_model(), "mistral");
    }

    #[test]
    fn test_model_role_fast_inference_requirement() {
        // Test: Validates Router role is marked as requiring fast inference.
        // Justification: Router gates the entire flow, so latency is critical.
        std::assert!(super::ModelRole::Router.requires_fast_inference());
        std::assert!(!super::ModelRole::Decomposer.requires_fast_inference());
        std::assert!(!super::ModelRole::Enhancer.requires_fast_inference());
        std::assert!(!super::ModelRole::Tester.requires_fast_inference());
    }

    #[test]
    fn test_model_role_priority() {
        // Test: Validates Router has highest priority.
        // Justification: Resource allocation should prioritize the routing decision.
        std::assert_eq!(super::ModelRole::Router.priority(), 10);
        std::assert!(super::ModelRole::Router.priority() > super::ModelRole::Decomposer.priority());
        std::assert!(super::ModelRole::Decomposer.priority() > super::ModelRole::Enhancer.priority());
        std::assert!(super::ModelRole::Enhancer.priority() > super::ModelRole::Tester.priority());
    }

    #[test]
    fn test_model_selection_strategy_defaults() {
        // Test: Validates default strategy uses recommended models.
        let strategy = super::ModelSelectionStrategy::default();
        std::assert_eq!(strategy.select_model_for_role(super::ModelRole::Router), "phi3");
        std::assert_eq!(strategy.select_model_for_role(super::ModelRole::Decomposer), "orca2");
        std::assert_eq!(strategy.select_model_for_role(super::ModelRole::Enhancer), "llama3.1");
        std::assert_eq!(strategy.select_model_for_role(super::ModelRole::Tester), "mistral");
    }

    #[test]
    fn test_model_selection_strategy_from_config() {
        // Test: Validates custom configuration overrides defaults.
        let mut config = std::collections::HashMap::new();
        config.insert(String::from("router"), String::from("custom-router-model"));
        config.insert(String::from("decomposer"), String::from("custom-decomposer-model"));

        let strategy = super::ModelSelectionStrategy::from_config(config);

        std::assert_eq!(strategy.select_model_for_role(super::ModelRole::Router), "custom-router-model");
        std::assert_eq!(strategy.select_model_for_role(super::ModelRole::Decomposer), "custom-decomposer-model");
        // Unspecified roles should use defaults
        std::assert_eq!(strategy.select_model_for_role(super::ModelRole::Enhancer), "llama3.1");
        std::assert_eq!(strategy.select_model_for_role(super::ModelRole::Tester), "mistral");
    }

    #[test]
    fn test_model_selection_strategy_all_assignments() {
        // Test: Validates get_all_assignments returns complete map.
        let strategy = super::ModelSelectionStrategy::default();
        let assignments = strategy.get_all_assignments();

        std::assert_eq!(assignments.len(), 4);
        std::assert!(assignments.contains_key(&super::ModelRole::Router));
        std::assert!(assignments.contains_key(&super::ModelRole::Decomposer));
        std::assert!(assignments.contains_key(&super::ModelRole::Enhancer));
        std::assert!(assignments.contains_key(&super::ModelRole::Tester));
    }

    #[test]
    fn test_model_role_display() {
        // Test: Validates Display implementation.
        std::assert_eq!(std::format!("{}", super::ModelRole::Router), "Router");
        std::assert_eq!(std::format!("{}", super::ModelRole::Decomposer), "Decomposer");
        std::assert_eq!(std::format!("{}", super::ModelRole::Enhancer), "Enhancer");
        std::assert_eq!(std::format!("{}", super::ModelRole::Tester), "Tester");
    }
}
