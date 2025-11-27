//! Feature-gated convenience function (disabled variant) to run a task through the orchestrator using a graph runtime.
//!
//! This file provides the default (no-feature) implementation for
//! `run_task_with_flow`. When the `graph_flow` Cargo feature is not enabled,
//! the function is available but returns an explanatory error string. This
//! keeps the crate API stable while ensuring the default workspace build
//! remains green without optional dependencies.
//!
//! Revision History
//! - 2025-11-13T00:27:00Z @AI: Add disabled variant of run_task_with_flow returning an explanatory error.

/// Attempts to run the orchestration flow for a single task using a feature-gated
/// graph runtime. When the `graph_flow` feature is not enabled, this returns
/// an error explaining how to enable the feature.
///
/// # Arguments
///
/// * `model` - The Ollama model name (e.g., "llama3.1"). Currently unused in this variant.
/// * `test_type` - The comprehension test type to request (e.g., "short_answer"). Unused here.
/// * `task` - The Task to orchestrate.
///
/// # Returns
///
/// * `Err(String)` - Always returns an error in the disabled variant.
pub async fn run_task_with_flow(
    _model: &str,
    _test_type: &str,
    _task: task_manager::domain::task::Task,
) -> std::result::Result<task_manager::domain::task::Task, std::string::String> {
    std::result::Result::Err(std::string::String::from(
        "graph_flow feature not enabled; rebuild with --features graph_flow",
    ))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_run_task_with_flow_returns_err_when_disabled() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let result = super::run_task_with_flow("llama3.1", "short_answer", task).await;
        std::assert!(result.is_err());
        let msg = result.unwrap_err();
        std::assert!(msg.contains("graph_flow feature not enabled"));
    }
}
