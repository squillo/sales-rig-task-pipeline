//! Orchestrator facade providing a stable API for running the task flow.
//!
//! The Orchestrator encapsulates model and test configuration and exposes
//! a single async `run` method that executes the end-to-end orchestration
//! using the current runtime implementation. Today, this delegates to the
//! sequential TaskGraphRunner via `run_task_with_flow`, and can be upgraded
//! to a graph runtime transparently without changing call sites.
//!
//! Revision History
//! - 2025-11-23 @AI: Update Orchestrator to use ProviderFactory (Phase 1 Sprint 3 Task 1.10).
//! - 2025-11-18T13:03:00Z @AI: Adjust constructor to take &str, add struct docs with example; no behavior change.
//! - 2025-11-13T21:39:00Z @AI: Introduce Orchestrator facade with async run() and unit test.

/// Facade type that owns configuration and runs orchestration for a Task.
///
/// The facade is a thin, stable layer over the current runtime helper. It
/// uses a ProviderFactory to create vendor-agnostic LLM adapters and provides
/// a single `run` method to process a Task end-to-end.
///
/// # Examples
///
/// ```ignore
/// // Create an action item and Task, then run with the facade.
/// let ai = transcript_extractor::domain::action_item::ActionItem {
///     title: std::string::String::from("Write release notes"),
///     assignee: std::option::Option::None,
///     due_date: std::option::Option::None,
/// };
/// let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
/// let orch = task_orchestrator::use_cases::orchestrator::Orchestrator::from_env().unwrap();
/// // Note: This requires an async runtime and an available model; omitted in this example.
/// // let updated = orch.run(task).await?;
/// ```
#[derive(Debug, Clone)]
pub struct Orchestrator {
    factory: crate::adapters::provider_factory::ProviderFactory,
    test_type: String,
}

impl Orchestrator {
    /// Creates a new Orchestrator from environment variables.
    ///
    /// Reads configuration from TASK_ORCHESTRATOR_PROVIDER and model-specific
    /// environment variables (OLLAMA_MODEL, OPENAI_MODEL, ANTHROPIC_MODEL).
    ///
    /// # Returns
    ///
    /// Returns an Orchestrator or an error if configuration is invalid.
    pub fn from_env() -> hexser::HexResult<Self> {
        let factory = crate::adapters::provider_factory::ProviderFactory::from_env()?;
        let test_type = std::env::var("TEST_TYPE").unwrap_or_else(|_| "short_answer".to_string());

        std::result::Result::Ok(Self {
            factory,
            test_type,
        })
    }

    /// Creates a new Orchestrator with explicit provider and model.
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name ("ollama", "openai", or "anthropic")
    /// * `model` - Model identifier
    /// * `test_type` - Test type for comprehension tests
    ///
    /// # Returns
    ///
    /// Returns an Orchestrator or an error if provider is invalid.
    pub fn new(provider: &str, model: &str, test_type: &str) -> hexser::HexResult<Self> {
        let factory = crate::adapters::provider_factory::ProviderFactory::new(provider, model)?;

        std::result::Result::Ok(Self {
            factory,
            test_type: test_type.to_string(),
        })
    }

    /// Returns the configured provider name.
    pub fn provider(&self) -> &str {
        self.factory.provider()
    }

    /// Returns the configured model name.
    pub fn model(&self) -> &str {
        self.factory.model()
    }

    /// Returns the configured test type.
    pub fn test_type(&self) -> &str {
        self.test_type.as_str()
    }

    /// Returns a reference to the provider factory.
    pub fn factory(&self) -> &crate::adapters::provider_factory::ProviderFactory {
        &self.factory
    }

    /// Runs orchestration for the provided Task and returns the updated Task.
    pub async fn run(
        &self,
        task: task_manager::domain::task::Task,
    ) -> std::result::Result<task_manager::domain::task::Task, std::string::String> {
        crate::use_cases::run_task_with_flow::run_task_with_flow(
            &self.factory,
            self.test_type.as_str(),
            task,
        ).await
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_orchestrator_runs_flow() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Ship Phase 6"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let orch = super::Orchestrator::new(
            "ollama",
            "llama3.1",
            "short_answer",
        ).unwrap();
        let result = super::Orchestrator::run(&orch, task).await;
        std::assert!(result.is_ok());
        let updated = result.unwrap();
        std::assert!(updated.enhancements.is_some());
        std::assert!(updated.comprehension_tests.is_some());
    }

    #[test]
    fn test_orchestrator_from_env() {
        // Test that we can create an orchestrator from environment
        unsafe {
            std::env::set_var("TASK_ORCHESTRATOR_PROVIDER", "ollama");
            std::env::set_var("OLLAMA_MODEL", "llama3.1");
        }

        let orch = super::Orchestrator::from_env();
        assert!(orch.is_ok());

        let orch = orch.unwrap();
        assert_eq!(orch.provider(), "ollama");
        assert_eq!(orch.model(), "llama3.1");
        assert_eq!(orch.test_type(), "short_answer"); // default

        unsafe {
            std::env::remove_var("TASK_ORCHESTRATOR_PROVIDER");
            std::env::remove_var("OLLAMA_MODEL");
        }
    }

    #[test]
    fn test_orchestrator_new() {
        let orch = super::Orchestrator::new("ollama", "qwen2.5", "multiple_choice");
        assert!(orch.is_ok());

        let orch = orch.unwrap();
        assert_eq!(orch.provider(), "ollama");
        assert_eq!(orch.model(), "qwen2.5");
        assert_eq!(orch.test_type(), "multiple_choice");
    }
}
