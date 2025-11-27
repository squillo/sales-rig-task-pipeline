//! Defines the GraphNode trait for node execution in the orchestrator graph.
//!
//! The GraphNode trait provides a minimal, framework-agnostic async interface
//! for executing a node over the GraphState. This placeholder will be replaced
//! by rs-graph-llm Task implementations in Phase 6.
//!
//! Revision History
//! - 2025-11-12T21:40:00Z @AI: Introduce GraphNode trait as temporary abstraction pending graph runtime.

/// Minimal async trait for graph node execution.
#[async_trait::async_trait]
pub trait GraphNode: std::marker::Send + std::marker::Sync {
    /// Executes this node with the provided state and returns the new state.
    ///
    /// # Errors
    ///
    /// Returns an error string if node execution fails in a recoverable way.
    async fn execute(
        &self,
        state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String>;
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_graph_node_trait_object_compiles() {
        struct Noop;
        #[async_trait::async_trait]
        impl super::GraphNode for Noop {
            async fn execute(
                &self,
                state: crate::graph::state::GraphState,
            ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
                std::result::Result::Ok(state)
            }
        }
        let ai = transcript_extractor::domain::action_item::ActionItem { title: std::string::String::from("X"), assignee: std::option::Option::None, due_date: std::option::Option::None };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);
        let node: std::boxed::Box<dyn super::GraphNode> = std::boxed::Box::new(Noop);
        let res = super::GraphNode::execute(node.as_ref(), state).await;
        std::assert!(res.is_ok());
    }
}
