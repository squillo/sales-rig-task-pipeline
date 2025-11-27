//! FlowRunner: thin runtime that executes the flow via shims.
//!
//! This application-layer runner coordinates the existing flow shims that
//! delegate to Phase 5 nodes. It provides a clean, unified execution path
//! for `run_task_with_flow` without introducing external runtime coupling.
//! The runner is deterministic and avoids external side effects.
//!
//! Revision History
//! - 2025-11-14T15:44:00Z @AI: Introduce FlowRunner that executes shims sequentially with a pass/fail check.

/// Executes the task orchestration using shimmed nodes.
#[derive(Clone)]
pub struct FlowRunner {
    enhancement_port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>,
    test_port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
    test_type: String,
}

impl FlowRunner {
    /// Creates a new FlowRunner with the provided ports and test type.
    pub fn new(
        enhancement_port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>,
        test_port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
        test_type: String,
    ) -> Self {
        FlowRunner { enhancement_port, test_port, test_type }
    }

    /// Runs the orchestration flow and returns the updated task.
    ///
    /// Flow:
    /// 1) SemanticRouterTaskShim decides route ("enhance" or "decompose").
    /// 2) EnhancementTaskShim generates enhancement and updates status.
    /// 3) ComprehensionTestTaskShim generates test and updates status.
    /// 4) CheckTestResultTaskShim evaluates pass/fail and may set OrchestrationComplete.
    ///
    /// If the decision is "fail", we return the updated task for a subsequent
    /// iteration by the caller. This avoids complex loops here and keeps the
    /// runner within the single-responsibility boundary.
    pub async fn run(
        &self,
        task: task_manager::domain::task::Task,
    ) -> std::result::Result<task_manager::domain::task::Task, std::string::String> {
        // Initialize state
        let mut state = crate::graph::state::GraphState::new(task);

        // 1) Route
        let router = crate::graph::flow_shims::semantic_router_task_shim::SemanticRouterTaskShim::new();
        state = crate::graph::flow_shims::semantic_router_task_shim::SemanticRouterTaskShim::run(&router, state).await?;

        // 2) Enhance (we enhance regardless of route; decompose fallback not implemented)
        let enh = crate::graph::flow_shims::enhancement_task_shim::EnhancementTaskShim::new(self.enhancement_port.clone());
        state = crate::graph::flow_shims::enhancement_task_shim::EnhancementTaskShim::run(&enh, state).await?;

        // 3) Comprehension test
        let ct = crate::graph::flow_shims::comprehension_test_task_shim::ComprehensionTestTaskShim::new(
            self.test_port.clone(),
            self.test_type.clone(),
        );
        state = crate::graph::flow_shims::comprehension_test_task_shim::ComprehensionTestTaskShim::run(&ct, state).await?;

        // 4) Check result
        let check = crate::graph::flow_shims::check_test_result_task_shim::CheckTestResultTaskShim::new();
        state = crate::graph::flow_shims::check_test_result_task_shim::CheckTestResultTaskShim::run(&check, state).await?;

        std::result::Result::Ok(state.task)
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
    async fn test_flow_runner_completes() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let runner = super::FlowRunner::new(
            std::sync::Arc::new(MockEnh),
            std::sync::Arc::new(MockCT),
            std::string::String::from("short_answer"),
        );
        let out = super::FlowRunner::run(&runner, task).await.unwrap();
        std::assert!(out.enhancements.is_some());
        std::assert!(out.comprehension_tests.is_some());
    }
}
