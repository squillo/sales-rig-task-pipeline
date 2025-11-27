//! Terminal EndTask for graph-flow that immediately ends execution.
//!
//! This shim implements `graph_flow::Task` and simply returns `NextAction::End`
//! when executed. It is used as the terminal node in the orchestrator state
//! graph to represent a successful completion branch.
//!
//! Revision History
//! - 2025-11-15T10:31:00Z @AI: Add EndTask shim to provide terminal node for graph assembly.

/// Task that completes the workflow when reached.
pub struct EndTask;

#[async_trait::async_trait]
impl graph_flow::Task for EndTask {
    async fn run(&self, _context: graph_flow::Context) -> graph_flow::Result<graph_flow::TaskResult> {
        std::result::Result::Ok(graph_flow::TaskResult::new(std::option::Option::None, graph_flow::NextAction::End))
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_end_task_returns_end() {
        let t = super::EndTask;
        let ctx = graph_flow::Context::new();
        let res = <super::EndTask as graph_flow::Task>::run(&t, ctx).await.unwrap();
        std::assert!(matches!(res.next_action, graph_flow::NextAction::End));
    }
}
