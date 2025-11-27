//! ComprehensionTestNode generates a comprehension test via the port.
//!
//! This node requests a comprehension test from the provided port and appends
//! it to the task's comprehension_tests list in the GraphState. It advances the
//! task status toward follow-on actions. This is a runtime-agnostic placeholder
//! pending rs-graph-llm integration in Phase 6.
//!
//! Revision History
//! - 2025-11-12T21:43:00Z @AI: Add ComprehensionTestNode with port integration and unit tests.

/// Node responsible for generating a comprehension test for the task.
pub struct ComprehensionTestNode {
    port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
    test_type: String,
}

impl ComprehensionTestNode {
    /// Creates a new node with the given port and test_type.
    pub fn new(
        port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
        test_type: String,
    ) -> Self {
        ComprehensionTestNode { port, test_type }
    }

    /// Executes test generation and updates the task in state.
    pub async fn execute(
        &self,
        mut state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        let ct = crate::ports::comprehension_test_port::ComprehensionTestPort::generate_comprehension_test(
            self.port.as_ref(),
            &state.task,
            self.test_type.as_str(),
        ).await?;
        let mut list = state.task.comprehension_tests.unwrap_or_else(|| std::vec::Vec::new());
        list.push(ct);
        state.task.comprehension_tests = std::option::Option::Some(list);
        state.task.status = task_manager::domain::task_status::TaskStatus::PendingFollowOn;
        std::result::Result::Ok(state)
    }
}

#[async_trait::async_trait]
impl crate::graph::nodes::graph_node::GraphNode for ComprehensionTestNode {
    async fn execute(
        &self,
        state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        ComprehensionTestNode::execute(self, state).await
    }
}

#[cfg(test)]
mod tests {
    struct MockPort;
    #[async_trait::async_trait]
    impl crate::ports::comprehension_test_port::ComprehensionTestPort for MockPort {
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
    async fn test_comprehension_test_node_appends_test() {
        let ai = transcript_extractor::domain::action_item::ActionItem { title: std::string::String::from("Title"), assignee: std::option::Option::None, due_date: std::option::Option::None };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);
        let node = super::ComprehensionTestNode::new(std::sync::Arc::new(MockPort), std::string::String::from("short_answer"));
        let out = crate::graph::nodes::graph_node::GraphNode::execute(&node, state).await.unwrap();
        let list = out.task.comprehension_tests.unwrap();
        std::assert_eq!(list.len(), 1);
        std::assert_eq!(out.task.status, task_manager::domain::task_status::TaskStatus::PendingFollowOn);
    }
}
