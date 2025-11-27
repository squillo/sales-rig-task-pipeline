//! CheckTestResultNode inspects comprehension tests and emits a pass/fail decision.
//!
//! This node examines the latest comprehension test attached to the task and
//! determines whether the task should proceed to completion ("pass") or loop
//! back for further enhancement ("fail"). The current heuristic is simple and
//! deterministic for unit testing.
//!
//! Revision History
//! - 2025-11-12T21:44:00Z @AI: Add CheckTestResultNode with deterministic pass/fail heuristic and tests.

/// Node that decides routing based on comprehension test content.
pub struct CheckTestResultNode;

impl CheckTestResultNode {
    /// Creates a new CheckTestResultNode.
    pub fn new() -> Self { CheckTestResultNode }

    /// Executes the check and writes a routing_decision of "pass" or "fail".
    pub async fn execute(
        &self,
        mut state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        let decision = match &state.task.comprehension_tests {
            std::option::Option::Some(list) if !list.is_empty() => {
                let last = list.last().unwrap();
                // Heuristic: if question length <= 80, treat as pass; else fail.
                if last.question.len() <= 80 { "pass" } else { "fail" }
            }
            _ => {
                // No tests yet: fail safe to request more enhancement.
                "fail"
            }
        };
        state.routing_decision = std::option::Option::Some(std::string::String::from(decision));
        if decision == "pass" {
            state.task.status = task_manager::domain::task_status::TaskStatus::OrchestrationComplete;
        }
        std::result::Result::Ok(state)
    }
}

#[async_trait::async_trait]
impl crate::graph::nodes::graph_node::GraphNode for CheckTestResultNode {
    async fn execute(
        &self,
        state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        CheckTestResultNode::execute(self, state).await
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_no_tests_results_in_fail() {
        let ai = transcript_extractor::domain::action_item::ActionItem { title: std::string::String::from("Title"), assignee: std::option::Option::None, due_date: std::option::Option::None };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);
        let node = super::CheckTestResultNode::new();
        let out = crate::graph::nodes::graph_node::GraphNode::execute(&node, state).await.unwrap();
        std::assert_eq!(out.routing_decision, std::option::Option::Some(std::string::String::from("fail")));
    }

    #[tokio::test]
    async fn test_short_question_passes() {
        let ai = transcript_extractor::domain::action_item::ActionItem { title: std::string::String::from("Title"), assignee: std::option::Option::None, due_date: std::option::Option::None };
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
        let node = super::CheckTestResultNode::new();
        let out = crate::graph::nodes::graph_node::GraphNode::execute(&node, state).await.unwrap();
        std::assert_eq!(out.routing_decision, std::option::Option::Some(std::string::String::from("pass")));
        std::assert_eq!(out.task.status, task_manager::domain::task_status::TaskStatus::OrchestrationComplete);
    }

    #[tokio::test]
    async fn test_long_question_fails() {
        let ai = transcript_extractor::domain::action_item::ActionItem { title: std::string::String::from("Title"), assignee: std::option::Option::None, due_date: std::option::Option::None };
        let mut task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let ct = task_manager::domain::comprehension_test::ComprehensionTest {
            test_id: std::string::String::from("ct-1"),
            task_id: task.id.clone(),
            timestamp: chrono::Utc::now(),
            test_type: std::string::String::from("long"),
            question: std::string::String::from("This is a very long, verbose question intended to exceed the threshold of eighty characters to force a fail routing decision in tests."),
            options: std::option::Option::None,
            correct_answer: std::string::String::from("A"),
        };
        task.comprehension_tests = std::option::Option::Some(vec![ct]);
        let state = crate::graph::state::GraphState::new(task);
        let node = super::CheckTestResultNode::new();
        let out = crate::graph::nodes::graph_node::GraphNode::execute(&node, state).await.unwrap();
        std::assert_eq!(out.routing_decision, std::option::Option::Some(std::string::String::from("fail")));
    }
}
