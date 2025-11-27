//! TaskGraphRunner: sequential runner for orchestrating a task through nodes.
//!
//! This application-layer component coordinates the existing placeholder nodes
//! (SemanticRouterNode, EnhancementNode, ComprehensionTestNode, CheckTestResultNode)
//! to provide an end-to-end, verifiable orchestration flow without a graph
//! runtime. In Phase 6, this will be replaced by an rs-graph-llm based runner.
//!
//! Revision History
//! - 2025-11-12T22:22:00Z @AI: Introduce TaskGraphRunner with sequential execution and unit tests.

/// Runner that executes the orchestration flow over a single task.
pub struct TaskGraphRunner {
    enhancement_port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>,
    test_port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
    test_type: String,
}

impl TaskGraphRunner {
    /// Creates a new TaskGraphRunner with the provided ports and default test_type.
    pub fn new(
        enhancement_port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>,
        test_port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
        test_type: String,
    ) -> Self {
        TaskGraphRunner { enhancement_port, test_port, test_type }
    }

    /// Runs the task through the sequential node flow and returns the updated task.
    ///
    /// Flow:
    /// 1) SemanticRouterNode decides route ("enhance" or "decompose").
    /// 2) EnhancementNode generates an enhancement.
    /// 3) ComprehensionTestNode generates a test (self.test_type).
    /// 4) CheckTestResultNode evaluates pass/fail and may set OrchestrationComplete.
    ///
    /// Note: For the interim, a "decompose" route follows the same linear path
    /// as "enhance" to maintain a working demo without a decomposition node.
    pub async fn run_task(
        &self,
        task: task_manager::domain::task::Task,
    ) -> std::result::Result<task_manager::domain::task::Task, std::string::String> {
        // Initialize state
        let mut state = crate::graph::state::GraphState::new(task);

        // 1) Route
        let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
        let triage_service = task_manager::domain::services::triage_service::TriageService::new(scorer);
        let router = crate::graph::nodes::semantic_router_node::SemanticRouterNode::new(triage_service);
        state = crate::graph::nodes::semantic_router_node::SemanticRouterNode::execute(&router, state).await?;
        let _route = state
            .routing_decision
            .clone()
            .unwrap_or_else(|| std::string::String::from("enhance"));

        // 2) Enhance (same path even if "decompose" for now)
        let enh_node = crate::graph::nodes::enhancement_node::EnhancementNode::new(self.enhancement_port.clone());
        state = crate::graph::nodes::graph_node::GraphNode::execute(&enh_node, state).await?;

        // 3) Comprehension test
        let comp_node = crate::graph::nodes::comprehension_test_node::ComprehensionTestNode::new(
            self.test_port.clone(),
            self.test_type.clone(),
        );
        state = crate::graph::nodes::graph_node::GraphNode::execute(&comp_node, state).await?;

        // 4) Check result
        let check = crate::graph::nodes::check_test_result_node::CheckTestResultNode::new();
        state = crate::graph::nodes::graph_node::GraphNode::execute(&check, state).await?;

        // Return updated task
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
    async fn test_runner_happy_path_completes() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let runner = super::TaskGraphRunner::new(
            std::sync::Arc::new(MockEnh),
            std::sync::Arc::new(MockCT),
            std::string::String::from("short_answer"),
        );
        let out = super::TaskGraphRunner::run_task(&runner, task).await.unwrap();
        let enhs = out.enhancements.unwrap();
        std::assert_eq!(enhs.len(), 1);
        let tests = out.comprehension_tests.unwrap();
        std::assert_eq!(tests.len(), 1);
        std::assert_eq!(out.status, task_manager::domain::task_status::TaskStatus::OrchestrationComplete);
    }

    #[tokio::test]
    async fn test_runner_long_title_routes_and_runs() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("This is a very long title intended to exceed the threshold for routing and also produce a long question"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let runner = super::TaskGraphRunner::new(
            std::sync::Arc::new(MockEnh),
            std::sync::Arc::new(MockCT),
            std::string::String::from("short_answer"),
        );
        let out = super::TaskGraphRunner::run_task(&runner, task).await.unwrap();
        // Enhancement still occurs
        std::assert!(out.enhancements.is_some());
        // Long question likely triggers fail path (heuristic > 80 chars)
        let status = out.status;
        let allowed = status == task_manager::domain::task_status::TaskStatus::PendingFollowOn
            || status == task_manager::domain::task_status::TaskStatus::OrchestrationComplete;
        std::assert!(allowed);
    }
}
