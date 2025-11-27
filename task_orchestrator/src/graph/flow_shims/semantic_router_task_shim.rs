//! Shim for SemanticRouterNode routing behavior.
//!
//! This shim delegates to `SemanticRouterNode::execute` to update the
//! `routing_decision` in the `GraphState`. It exists to provide a stable API
//! surface that a graph runtime can call without changing node internals.
//!
//! Revision History
//! - 2025-11-23T16:30:00Z @AI: Update shim to inject TriageService for intelligent routing (Phase 3 Sprint 6).
//! - 2025-11-15T09:21:00Z @AI: Implement graph_flow::Task for shim; add Context round-trip unit test.
//! - 2025-11-13T09:32:00Z @AI: Add SemanticRouterTaskShim with run() delegating to node; add unit test.

/// Shim that mirrors how a graph runtime would invoke the router node.
pub struct SemanticRouterTaskShim {
    triage_service: task_manager::domain::services::triage_service::TriageService,
}

impl SemanticRouterTaskShim {
    /// Constructs a new SemanticRouterTaskShim with TriageService for intelligent routing.
    pub fn new() -> Self {
        let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
        let triage_service = task_manager::domain::services::triage_service::TriageService::new(scorer);
        SemanticRouterTaskShim { triage_service }
    }

    /// Runs the router logic by delegating to SemanticRouterNode::execute.
    pub async fn run(
        &self,
        state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        let node = crate::graph::nodes::semantic_router_node::SemanticRouterNode::new(self.triage_service.clone());
        crate::graph::nodes::semantic_router_node::SemanticRouterNode::execute(&node, state).await
    }
}

#[async_trait::async_trait]
impl graph_flow::Task for SemanticRouterTaskShim {
    async fn run(&self, context: graph_flow::Context) -> graph_flow::Result<graph_flow::TaskResult> {
        // Retrieve a Task from context, or synthesize from title if only a title is provided.
        let maybe_task: std::option::Option<task_manager::domain::task::Task> = context.get("task").await;
        let task = match maybe_task {
            std::option::Option::Some(t) => t,
            std::option::Option::None => {
                let title: std::string::String = context.get("task_title").await.unwrap_or_else(|| std::string::String::from(""));
                let ai = transcript_extractor::domain::action_item::ActionItem { title, assignee: std::option::Option::None, due_date: std::option::Option::None };
                task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None)
            }
        };
        // Run node logic
        let state_in = crate::graph::state::GraphState::new(task);
        let state_out = match SemanticRouterTaskShim::run(self, state_in).await {
                    std::result::Result::Ok(s) => s,
                    std::result::Result::Err(e) => return std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(e)),
                };
        // Persist routing decision back to context for downstream tasks
        if let std::option::Option::Some(decision) = state_out.routing_decision.clone() {
            context.set("routing_decision", decision.clone()).await;
            return std::result::Result::Ok(graph_flow::TaskResult::new(std::option::Option::Some(decision), graph_flow::NextAction::Continue));
        }
        std::result::Result::Ok(graph_flow::TaskResult::new(std::option::Option::None, graph_flow::NextAction::Continue))
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_router_shim_updates_routing() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Short title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);
        let shim = super::SemanticRouterTaskShim::new();
        let out = super::SemanticRouterTaskShim::run(&shim, state).await.unwrap();
        std::assert!(out.routing_decision.is_some());
    }

    #[tokio::test]
    async fn test_task_impl_writes_decision_to_context() {
        let shim = super::SemanticRouterTaskShim::new();
        let ctx = graph_flow::Context::new();
        // Provide only a title; shim will synthesize a Task
        ctx.set("task_title", std::string::String::from("Short title")).await;
        let result = <super::SemanticRouterTaskShim as graph_flow::Task>::run(&shim, ctx.clone()).await.unwrap();
        std::assert!(matches!(result.next_action, graph_flow::NextAction::Continue));
        let decision: std::option::Option<std::string::String> = ctx.get("routing_decision").await;
        std::assert_eq!(decision, std::option::Option::Some(std::string::String::from("enhance")));
    }
}
