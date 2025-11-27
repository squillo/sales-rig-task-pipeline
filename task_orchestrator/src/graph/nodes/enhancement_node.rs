//! EnhancementNode generates an enhancement via the TaskEnhancementPort.
//!
//! This node integrates with the port layer to produce an enhancement for the
//! current task and appends it to the GraphState's task.enhancements collection.
//! In Phase 6 this will be implemented as an rs-graph-llm Task; for now we
//! provide a framework-agnostic async execute method and GraphNode impl.
//!
//! Revision History
//! - 2025-11-12T21:42:00Z @AI: Add EnhancementNode with port integration and unit tests.

/// Node responsible for generating a task enhancement through a provided port.
pub struct EnhancementNode {
    port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>,
}

impl EnhancementNode {
    /// Creates a new EnhancementNode with the given port.
    pub fn new(port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>) -> Self {
        EnhancementNode { port }
    }

    /// Executes enhancement generation and updates the task in state.
    pub async fn execute(
        &self,
        mut state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        let enh = crate::ports::task_enhancement_port::TaskEnhancementPort::generate_enhancement(self.port.as_ref(), &state.task).await?;
        let mut list = state.task.enhancements.unwrap_or_else(|| std::vec::Vec::new());
        list.push(enh);
        state.task.enhancements = std::option::Option::Some(list);
        // Suggest next step status for clarity; no strict coupling to UI.
        state.task.status = task_manager::domain::task_status::TaskStatus::PendingComprehensionTest;
        std::result::Result::Ok(state)
    }
}

#[async_trait::async_trait]
impl crate::graph::nodes::graph_node::GraphNode for EnhancementNode {
    async fn execute(
        &self,
        state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        EnhancementNode::execute(self, state).await
    }
}

#[cfg(test)]
mod tests {
    struct MockPort;
    #[async_trait::async_trait]
    impl crate::ports::task_enhancement_port::TaskEnhancementPort for MockPort {
        async fn generate_enhancement(
            &self,
            task: &task_manager::domain::task::Task,
        ) -> std::result::Result<task_manager::domain::enhancement::Enhancement, std::string::String> {
            let enh = task_manager::domain::enhancement::Enhancement {
                enhancement_id: std::string::String::from("e-1"),
                task_id: task.id.clone(),
                timestamp: chrono::Utc::now(),
                enhancement_type: std::string::String::from("rewrite"),
                content: std::format!("Enhanced: {}", task.title),
            };
            std::result::Result::Ok(enh)
        }
    }

    #[tokio::test]
    async fn test_enhancement_node_appends_enhancement() {
        let ai = transcript_extractor::domain::action_item::ActionItem { title: std::string::String::from("Title"), assignee: std::option::Option::None, due_date: std::option::Option::None };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);
        let node = super::EnhancementNode::new(std::sync::Arc::new(MockPort));
        let out = crate::graph::nodes::graph_node::GraphNode::execute(&node, state).await.unwrap();
        let list = out.task.enhancements.unwrap();
        std::assert_eq!(list.len(), 1);
        std::assert_eq!(out.task.status, task_manager::domain::task_status::TaskStatus::PendingComprehensionTest);
    }
}
