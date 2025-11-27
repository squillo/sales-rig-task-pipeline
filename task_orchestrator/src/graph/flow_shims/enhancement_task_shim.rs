//! Shim for EnhancementNode that generates an enhancement via the port.
//!
//! This shim delegates to `EnhancementNode::execute` to append an enhancement
//! to the task in `GraphState` and advance status to PendingComprehensionTest.
//! It is a stable call surface for a graph runtime to invoke without changing
//! node internals.
//!
//! Revision History
//! - 2025-11-15T09:45:30Z @AI: Implement graph_flow::Task; persist updated Task in Context; add Task-impl unit test.
//! - 2025-11-13T09:32:00Z @AI: Add EnhancementTaskShim with run() delegating to node; add unit test.

/// Shim that mirrors how a graph runtime would invoke the enhancement node.
pub struct EnhancementTaskShim {
    port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>,
}

impl EnhancementTaskShim {
    /// Constructs a new EnhancementTaskShim with the given port.
    pub fn new(
        port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>,
    ) -> Self { EnhancementTaskShim { port } }

    /// Runs enhancement generation by delegating to EnhancementNode::execute.
    pub async fn run(
        &self,
        state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        let node = crate::graph::nodes::enhancement_node::EnhancementNode::new(self.port.clone());
        crate::graph::nodes::graph_node::GraphNode::execute(&node, state).await
    }
}

#[async_trait::async_trait]
impl graph_flow::Task for EnhancementTaskShim {
    async fn run(&self, context: graph_flow::Context) -> graph_flow::Result<graph_flow::TaskResult> {
        let maybe_task: std::option::Option<task_manager::domain::task::Task> = context.get("task").await;
        let task = match maybe_task {
            std::option::Option::Some(t) => t,
            std::option::Option::None => {
                let title: std::string::String = context.get("task_title").await.unwrap_or_else(|| std::string::String::from(""));
                let ai = transcript_extractor::domain::action_item::ActionItem { title, assignee: std::option::Option::None, due_date: std::option::Option::None };
                task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None)
            }
        };
        let state_in = crate::graph::state::GraphState::new(task);
        let state_out = match EnhancementTaskShim::run(self, state_in).await {
            std::result::Result::Ok(s) => s,
            std::result::Result::Err(e) => return std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(e)),
        };
        // Persist updated task for downstream tasks
        context.set("task", state_out.task.clone()).await;
        std::result::Result::Ok(graph_flow::TaskResult::new(std::option::Option::None, graph_flow::NextAction::Continue))
    }
}

#[cfg(test)]
mod tests {
    struct MockEnh;
    #[async_trait::async_trait]
    impl crate::ports::task_enhancement_port::TaskEnhancementPort for MockEnh {
        async fn generate_enhancement(
            &self,
            task: &task_manager::domain::task::Task,
        ) -> std::result::Result<task_manager::domain::enhancement::Enhancement, std::string::String> {
            let enh = task_manager::domain::enhancement::Enhancement {
                enhancement_id: std::string::String::from("e-1"),
                task_id: task.id.clone(),
                timestamp: chrono::Utc::now(),
                enhancement_type: std::string::String::from("rewrite"),
                content: std::format!("E:{}", task.title),
            };
            std::result::Result::Ok(enh)
        }
    }

    #[tokio::test]
    async fn test_enhancement_shim_appends_enhancement() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);
        let shim = super::EnhancementTaskShim::new(std::sync::Arc::new(MockEnh));
        let out = super::EnhancementTaskShim::run(&shim, state).await.unwrap();
        std::assert!(out.task.enhancements.is_some());
        std::assert_eq!(out.task.status, task_manager::domain::task_status::TaskStatus::PendingComprehensionTest);
    }

    #[tokio::test]
    async fn test_task_impl_persists_task_in_context() {
        let shim = super::EnhancementTaskShim::new(std::sync::Arc::new(MockEnh));
        let ctx = graph_flow::Context::new();
        // Only provide a title; shim synthesizes Task
        ctx.set("task_title", std::string::String::from("Title")).await;
        let result = <super::EnhancementTaskShim as graph_flow::Task>::run(&shim, ctx.clone()).await.unwrap();
        std::assert!(matches!(result.next_action, graph_flow::NextAction::Continue));
        let task_after: std::option::Option<task_manager::domain::task::Task> = ctx.get("task").await;
        let t = task_after.expect("task should be present in context");
        std::assert!(t.enhancements.is_some());
        std::assert_eq!(t.status, task_manager::domain::task_status::TaskStatus::PendingComprehensionTest);
    }
}
