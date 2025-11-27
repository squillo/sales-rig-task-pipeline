//! Shim for CheckTestResultNode pass/fail decision.
//!
//! This shim delegates to `CheckTestResultNode::execute` to write a
//! `routing_decision` of "pass" or "fail" into the `GraphState` and update
//! task status on pass. It provides a stable surface a graph runtime could
//! call without changing node internals.
//!
//! Revision History
//! - 2025-11-15T10:25:00Z @AI: Implement graph_flow::Task for shim; persist decision and task in Context; add Task-impl unit test.
//! - 2025-11-14T09:27:30Z @AI: Add CheckTestResultTaskShim with run() delegating to node; add unit tests.

/// Shim that mirrors how a graph runtime would invoke the check-test node.
pub struct CheckTestResultTaskShim;

impl CheckTestResultTaskShim {
    /// Constructs a new CheckTestResultTaskShim.
    pub fn new() -> Self { CheckTestResultTaskShim }

    /// Runs the check by delegating to CheckTestResultNode::execute.
    pub async fn run(
        &self,
        state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        let node = crate::graph::nodes::check_test_result_node::CheckTestResultNode::new();
        crate::graph::nodes::graph_node::GraphNode::execute(&node, state).await
    }
}

#[async_trait::async_trait]
impl graph_flow::Task for CheckTestResultTaskShim {
    async fn run(&self, context: graph_flow::Context) -> graph_flow::Result<graph_flow::TaskResult> {
        // Retrieve Task from context or synthesize from title
        let maybe_task: std::option::Option<task_manager::domain::task::Task> = context.get("task").await;
        let task = match maybe_task {
            std::option::Option::Some(t) => t,
            std::option::Option::None => {
                let title: std::string::String = context
                    .get("task_title")
                    .await
                    .unwrap_or_else(|| std::string::String::from(""));
                let ai = transcript_extractor::domain::action_item::ActionItem {
                    title,
                    assignee: std::option::Option::None,
                    due_date: std::option::Option::None,
                };
                task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None)
            }
        };
        let state_in = crate::graph::state::GraphState::new(task);
        let state_out = match CheckTestResultTaskShim::run(self, state_in).await {
            std::result::Result::Ok(s) => s,
            std::result::Result::Err(e) => {
                return std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(e))
            }
        };
        // Persist decision and updated task into context for downstream usage
        if let std::option::Option::Some(decision) = state_out.routing_decision.clone() {
            context.set("routing_decision", decision.clone()).await;
            context.set("task", state_out.task.clone()).await;
            return std::result::Result::Ok(graph_flow::TaskResult::new(
                std::option::Option::Some(decision),
                graph_flow::NextAction::Continue,
            ));
        }
        context.set("task", state_out.task.clone()).await;
        std::result::Result::Ok(graph_flow::TaskResult::new(std::option::Option::None, graph_flow::NextAction::Continue))
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_check_shim_without_tests_routes_fail() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);
        let shim = super::CheckTestResultTaskShim::new();
        let out = super::CheckTestResultTaskShim::run(&shim, state).await.unwrap();
        std::assert_eq!(out.routing_decision, std::option::Option::Some(std::string::String::from("fail")));
    }

    #[tokio::test]
    async fn test_check_shim_with_short_question_passes() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let ct = task_manager::domain::comprehension_test::ComprehensionTest {
            test_id: std::string::String::from("ct-1"),
            task_id: task.id.clone(),
            timestamp: chrono::Utc::now(),
            test_type: std::string::String::from("short_answer"),
            question: std::string::String::from("Short?"),
            options: std::option::Option::None,
            correct_answer: std::string::String::from("Yes"),
        };
        task.comprehension_tests = std::option::Option::Some(vec![ct]);
        let state = crate::graph::state::GraphState::new(task);
        let shim = super::CheckTestResultTaskShim::new();
        let out = super::CheckTestResultTaskShim::run(&shim, state).await.unwrap();
        std::assert_eq!(out.routing_decision, std::option::Option::Some(std::string::String::from("pass")));
        std::assert_eq!(out.task.status, task_manager::domain::task_status::TaskStatus::OrchestrationComplete);
    }

    #[tokio::test]
    async fn test_task_impl_persists_decision_and_task_in_context() {
        let shim = super::CheckTestResultTaskShim::new();
        let ctx = graph_flow::Context::new();
        // Provide a title with no tests; decision should be fail
        ctx.set("task_title", std::string::String::from("Title")).await;
        let result = <super::CheckTestResultTaskShim as graph_flow::Task>::run(&shim, ctx.clone()).await.unwrap();
        std::assert!(matches!(result.next_action, graph_flow::NextAction::Continue));
        let decision: std::option::Option<std::string::String> = ctx.get("routing_decision").await;
        std::assert_eq!(decision, std::option::Option::Some(std::string::String::from("fail")));
        let task_after: std::option::Option<task_manager::domain::task::Task> = ctx.get("task").await;
        std::assert!(task_after.is_some());
    }
}
