//! Shim for ComprehensionTestNode that generates a comprehension test via the port.
//!
//! This shim delegates to `ComprehensionTestNode::execute` to append a test
//! to the task in `GraphState` and advance status to PendingFollowOn. It
//! provides a stable call surface for a graph runtime to invoke without
//! changing node internals.
//!
//! Revision History
//! - 2025-11-15T09:46:10Z @AI: Implement graph_flow::Task; persist updated Task in Context; add Task-impl unit test.
//! - 2025-11-14T09:27:30Z @AI: Add ComprehensionTestTaskShim with run() delegating to node; add unit test.

/// Shim that mirrors how a graph runtime would invoke the comprehension test node.
pub struct ComprehensionTestTaskShim {
    port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
    test_type: String,
}

impl ComprehensionTestTaskShim {
    /// Constructs a new ComprehensionTestTaskShim with the given port and test_type.
    pub fn new(
        port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
        test_type: String,
    ) -> Self {
        ComprehensionTestTaskShim { port, test_type }
    }

    /// Runs test generation by delegating to ComprehensionTestNode::execute.
    pub async fn run(
        &self,
        state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        let node = crate::graph::nodes::comprehension_test_node::ComprehensionTestNode::new(
            self.port.clone(),
            self.test_type.clone(),
        );
        crate::graph::nodes::graph_node::GraphNode::execute(&node, state).await
    }
}

#[async_trait::async_trait]
impl graph_flow::Task for ComprehensionTestTaskShim {
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
        let state_out = match ComprehensionTestTaskShim::run(self, state_in).await {
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
    struct MockCT;
    #[async_trait::async_trait]
    impl crate::ports::comprehension_test_port::ComprehensionTestPort for MockCT {
        async fn generate_comprehension_test(
            &self,
            task: &task_manager::domain::task::Task,
            test_type: &str,
        ) -> std::result::Result<task_manager::domain::comprehension_test::ComprehensionTest, std::string::String> {
            let ct = task_manager::domain::comprehension_test::ComprehensionTest {
                test_id: std::string::String::from("ct-1"),
                task_id: task.id.clone(),
                timestamp: chrono::Utc::now(),
                test_type: std::string::String::from(test_type),
                question: std::format!("Q for {}", task.title),
                options: std::option::Option::None,
                correct_answer: std::string::String::from("A"),
            };
            std::result::Result::Ok(ct)
        }
    }

    #[tokio::test]
    async fn test_comprehension_test_shim_appends_test() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);
        let shim = super::ComprehensionTestTaskShim::new(
            std::sync::Arc::new(MockCT),
            std::string::String::from("short_answer"),
        );
        let out = super::ComprehensionTestTaskShim::run(&shim, state).await.unwrap();
        let list = out.task.comprehension_tests.unwrap();
        std::assert_eq!(list.len(), 1);
        std::assert_eq!(out.task.status, task_manager::domain::task_status::TaskStatus::PendingFollowOn);
    }

    #[tokio::test]
    async fn test_task_impl_persists_task_in_context() {
        let shim = super::ComprehensionTestTaskShim::new(
            std::sync::Arc::new(MockCT),
            std::string::String::from("short_answer"),
        );
        let ctx = graph_flow::Context::new();
        ctx.set("task_title", std::string::String::from("Title")).await;
        let result = <super::ComprehensionTestTaskShim as graph_flow::Task>::run(&shim, ctx.clone()).await.unwrap();
        std::assert!(matches!(result.next_action, graph_flow::NextAction::Continue));
        let task_after: std::option::Option<task_manager::domain::task::Task> = ctx.get("task").await;
        let t = task_after.expect("task should be present in context");
        std::assert!(t.comprehension_tests.is_some());
        std::assert_eq!(t.status, task_manager::domain::task_status::TaskStatus::PendingFollowOn);
    }
}
