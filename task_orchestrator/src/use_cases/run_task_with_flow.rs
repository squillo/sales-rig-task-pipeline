//! Convenience function to run a task through the orchestrator using a graph runtime.
//!
//! This unified helper is always available and now delegates to the
//! graph_flow::FlowRunner backed by the assembled orchestrator graph.
//! This maintains a verifiable end-to-end flow using the StateGraph.
//!
//! Revision History
//! - 2025-11-23T23:15:00Z @AI: Use role-based adapter creation for heterogeneous pipeline (Phase 5 Sprint 10 Task 5.5).
//! - 2025-11-23 @AI: Update to use ProviderFactory for vendor-agnostic LLM providers (Phase 1 Sprint 3 Task 1.11).
//! - 2025-11-18T11:23:10Z @AI: Add optional SQLite-backed session storage behind `sqlite_persistence` feature; default remains in-memory.
//! - 2025-11-15T10:41:00Z @AI: Switch to graph_flow::FlowRunner using assembled orchestrator flow.
//! - 2025-11-14T15:44:00Z @AI: Switch implementation to FlowRunner using shim-based execution.
//! - 2025-11-13T21:06:00Z @AI: Unify: make helper always available; delegate to TaskGraphRunner.
//! - 2025-11-13T00:31:00Z @AI: Add default (no-feature) run_task_with_flow that returns an explanatory error.

/// Runs the orchestration flow using the unified runtime entrypoint.
///
/// This function uses a ProviderFactory to construct vendor-agnostic LLM adapters,
/// then executes the flow via the shim-based FlowRunner. This keeps the API stable
/// and deterministic while supporting multiple LLM providers.
///
/// # Arguments
///
/// * `factory` - The ProviderFactory for creating LLM adapters.
/// * `test_type` - The comprehension test type to request (e.g., "short_answer").
/// * `task` - The Task to orchestrate.
///
/// # Returns
///
/// * `Ok(Task)` - The updated task after orchestration.
/// * `Err(String)` - An error message if any node fails during execution.
pub async fn run_task_with_flow(
    factory: &crate::adapters::provider_factory::ProviderFactory,
    test_type: &str,
    task: task_manager::domain::task::Task,
) -> std::result::Result<task_manager::domain::task::Task, std::string::String> {
    // Build adapters (ports) using the factory
    // Use role-based adapter creation for heterogeneous pipeline optimization
    let enh_port = factory.create_enhancement_adapter_for_role(
        crate::domain::model_role::ModelRole::Enhancer
    ).map_err(|e| e.to_string())?;

    let ct_port = factory.create_comprehension_test_adapter()
        .map_err(|e| e.to_string())?;

    // Use Orca-2 for decomposition (excels at complex reasoning)
    let decomp_port = factory.create_task_decomposition_adapter_for_role(
        crate::domain::model_role::ModelRole::Decomposer
    ).map_err(|e| e.to_string())?;

    // Assemble graph
    let builder = crate::graph::assemble_orchestrator_flow::assemble_orchestrator_flow(
        enh_port.clone(),
        ct_port.clone(),
        decomp_port.clone(),
        std::string::String::from(test_type),
    );
    let graph = std::sync::Arc::new(builder.build());

    // Create storage and runner
    // If the `sqlite_persistence` feature is enabled, prefer SQLite; otherwise, use in-memory.
    #[allow(clippy::let_and_return)]
    let storage: std::sync::Arc<dyn graph_flow::SessionStorage> = {
        #[cfg(feature = "sqlite_persistence")]
        {
            let db_url = std::env::var("TASK_ORCHESTRATOR_SQLITE_URL")
                .unwrap_or_else(|_| std::string::String::from("sqlite::memory:"));
            let sqlite = match crate::infrastructure::sqlite_session_storage::SQLiteSessionStorage::connect(&db_url).await {
                std::result::Result::Ok(s) => s,
                std::result::Result::Err(e) => {
                    return std::result::Result::Err(std::format!("sqlite connect error: {}", e));
                }
            };
            let arc: std::sync::Arc<dyn graph_flow::SessionStorage> = std::sync::Arc::new(sqlite);
            arc
        }
        #[cfg(not(feature = "sqlite_persistence"))]
        {
            let arc: std::sync::Arc<dyn graph_flow::SessionStorage> = std::sync::Arc::new(graph_flow::InMemorySessionStorage::new());
            arc
        }
    };
    let runner = graph_flow::FlowRunner::new(graph, storage.clone());

    // Create a session and seed context with the task
    let session_id = uuid::Uuid::new_v4().to_string();
    // Compute start task id (router)
    let router = std::sync::Arc::new(crate::graph::flow_shims::semantic_router_task_shim::SemanticRouterTaskShim::new());
    let start_id = <crate::graph::flow_shims::semantic_router_task_shim::SemanticRouterTaskShim as graph_flow::Task>::id(router.as_ref());
    let session = graph_flow::Session::new_from_task(session_id.clone(), start_id);
    graph_flow::Context::set(&session.context, "task", task.clone()).await;
    match graph_flow::SessionStorage::save(storage.as_ref(), session).await {
        std::result::Result::Ok(_) => {}
        std::result::Result::Err(e) => return std::result::Result::Err(std::format!("session save error: {:?}", e)),
    }

    // Execute until completion or waiting for input
    loop {
        let step = match graph_flow::FlowRunner::run(&runner, &session_id).await {
            std::result::Result::Ok(s) => s,
            std::result::Result::Err(e) => return std::result::Result::Err(std::format!("runner error: {:?}", e)),
        };
        match step.status {
            graph_flow::ExecutionStatus::Completed => break,
            graph_flow::ExecutionStatus::Paused { next_task_id: _, reason: _ } => continue,
            graph_flow::ExecutionStatus::WaitingForInput => {
                return std::result::Result::Err(std::string::String::from("waiting for input"))
            }
            graph_flow::ExecutionStatus::Error(err) => {
                return std::result::Result::Err(err)
            }
        }
    }

    // Retrieve final session and extract task from context
    let final_session = match graph_flow::SessionStorage::get(storage.as_ref(), &session_id).await {
        std::result::Result::Ok(s) => s,
        std::result::Result::Err(e) => return std::result::Result::Err(std::format!("session get error: {:?}", e)),
    };
    if let std::option::Option::Some(sess) = final_session {
        let maybe_task: std::option::Option<task_manager::domain::task::Task> = graph_flow::Context::get(&sess.context, "task").await;
        if let std::option::Option::Some(t) = maybe_task { return std::result::Result::Ok(t); }
    }

    std::result::Result::Err(std::string::String::from("task not found in final context"))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_run_task_with_flow_completes() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let factory = crate::adapters::provider_factory::ProviderFactory::new("ollama", "llama3.1").unwrap();
        let result = super::run_task_with_flow(&factory, "short_answer", task).await;
        std::assert!(result.is_ok());
    }

    #[cfg(feature = "sqlite_persistence")]
    #[tokio::test]
    async fn test_run_task_with_flow_sqlite_persistence_smoke() {
        // With the `sqlite_persistence` feature enabled, the helper uses SQLite by default (sqlite::memory:)
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("SQLite Path"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let factory = crate::adapters::provider_factory::ProviderFactory::new("ollama", "llama3.1").unwrap();
        let result = super::run_task_with_flow(&factory, "short_answer", task).await;
        std::assert!(result.is_ok());
    }
}
