//! Convenience function to run a task through the orchestrator using Ollama-based adapters.
//!
//! This helper constructs the minimal adapter set (enhancement and comprehension test)
//! using the provided Ollama model name, wires them into the TaskGraphRunner, and executes
//! the sequential orchestration flow over the supplied Task. It keeps the system verifiable
//! without a graph runtime while Phase 6 integration proceeds behind feature gates.
//!
//! Revision History
//! - 2025-11-12T23:26:00Z @AI: Add run_task_with_ollama helper with unit test; no new dependencies.

/// Runs the orchestration flow for a single task using Ollama-backed adapters.
///
/// This function builds the enhancement and comprehension test adapters with the
/// provided `model`, constructs a `TaskGraphRunner` with the specified `test_type`,
/// and executes the flow, returning the updated task on success.
///
/// # Arguments
///
/// * `model` - The Ollama model name (e.g., "llama3.1").
/// * `test_type` - The comprehension test type to request (e.g., "short_answer").
/// * `task` - The Task to orchestrate.
///
/// # Returns
///
/// * `Ok(Task)` - The updated task after orchestration.
/// * `Err(String)` - An error message if any node fails during execution.
///
/// # Errors
///
/// Propagates errors from underlying adapters or nodes as `Err(String)`.
pub async fn run_task_with_ollama(
    model: &str,
    test_type: &str,
    task: task_manager::domain::task::Task,
) -> std::result::Result<task_manager::domain::task::Task, std::string::String> {
    let enh = std::sync::Arc::new(
        crate::adapters::ollama_enhancement_adapter::OllamaEnhancementAdapter::new(
            std::string::String::from(model),
        ),
    );
    let ct = std::sync::Arc::new(
        crate::adapters::ollama_comprehension_test_adapter::OllamaComprehensionTestAdapter::new(
            std::string::String::from(model),
        ),
    );

    let runner = crate::use_cases::task_graph_runner::TaskGraphRunner::new(
        enh,
        ct,
        std::string::String::from(test_type),
    );

    crate::use_cases::task_graph_runner::TaskGraphRunner::run_task(&runner, task).await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_run_task_with_ollama_completes() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Write README for project"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let result = super::run_task_with_ollama("llama3.1", "short_answer", task).await;
        std::assert!(result.is_ok());
        let updated = result.unwrap();
        // Should have at least one enhancement and one comprehension test
        std::assert!(updated.enhancements.is_some());
        std::assert!(updated.comprehension_tests.is_some());
    }
}
