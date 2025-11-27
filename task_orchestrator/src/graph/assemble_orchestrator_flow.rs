//! Assembles the orchestrator flow using graph_flow::GraphBuilder.
//!
//! This function wires the Phase 6 task shims into a concrete workflow graph:
//! router → (cond) enhance → comprehend → check → (cond) { end | enhance }.
//! The conditional edges consult `routing_decision` persisted in the
//! graph_flow::Context by the router and check tasks.
//!
//! Ports for enhancement and comprehension test generation are provided by
//! callers to keep this assembly decoupled from adapter choices.
//!
//! Revision History
//! - 2025-11-23T18:00:00Z @AI: Add decomposition path for Phase 3 Sprint 7.
//! - 2025-11-15T10:34:00Z @AI: Add assemble_orchestrator_flow with conditional edges and minimal build test.

/// Builds and returns a GraphBuilder with orchestrator tasks and edges wired.
///
/// The returned builder contains the following tasks:
/// - SemanticRouterTaskShim (routes based on complexity)
/// - TaskDecompositionTaskShim (requires TaskDecompositionPort)
/// - EnhancementTaskShim (requires TaskEnhancementPort)
/// - ComprehensionTestTaskShim (requires ComprehensionTestPort)
/// - CheckTestResultTaskShim
/// - EndTask (terminal node)
///
/// Edges:
/// - router --[routing_decision == "decompose"]--> decompose --> end
/// - router --[routing_decision == "enhance"]--> enhance -> comprehend -> check
/// - check --[routing_decision == "pass"]--> end; else -> enhance (loop)
pub fn assemble_orchestrator_flow(
    enhancement_port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>,
    comprehension_port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
    decomposition_port: std::sync::Arc<dyn crate::ports::task_decomposition_port::TaskDecompositionPort>,
    comprehension_test_type: std::string::String,
) -> graph_flow::GraphBuilder {
    let router = std::sync::Arc::new(crate::graph::flow_shims::semantic_router_task_shim::SemanticRouterTaskShim::new());
    let decompose = std::sync::Arc::new(crate::graph::flow_shims::task_decomposition_task_shim::TaskDecompositionTaskShim::new(decomposition_port));
    let enhance = std::sync::Arc::new(crate::graph::flow_shims::enhancement_task_shim::EnhancementTaskShim::new(enhancement_port));
    let comprehend = std::sync::Arc::new(crate::graph::flow_shims::comprehension_test_task_shim::ComprehensionTestTaskShim::new(
        comprehension_port,
        comprehension_test_type,
    ));
    let check = std::sync::Arc::new(crate::graph::flow_shims::check_test_result_task_shim::CheckTestResultTaskShim::new());
    let end = std::sync::Arc::new(crate::graph::flow_shims::end_task::EndTask);

    let builder = graph_flow::GraphBuilder::new("task_orchestrator")
        .add_task(router.clone())
        .add_task(decompose.clone())
        .add_task(enhance.clone())
        .add_task(comprehend.clone())
        .add_task(check.clone())
        .add_task(end.clone())
        .add_conditional_edge(
            <crate::graph::flow_shims::semantic_router_task_shim::SemanticRouterTaskShim as graph_flow::Task>::id(router.as_ref()),
            |ctx| ctx.get_sync::<std::string::String>("routing_decision").unwrap_or_else(|| std::string::String::new()) == "decompose",
            <crate::graph::flow_shims::task_decomposition_task_shim::TaskDecompositionTaskShim as graph_flow::Task>::id(decompose.as_ref()),
            <crate::graph::flow_shims::enhancement_task_shim::EnhancementTaskShim as graph_flow::Task>::id(enhance.as_ref()),
        )
        .add_edge(
            <crate::graph::flow_shims::task_decomposition_task_shim::TaskDecompositionTaskShim as graph_flow::Task>::id(decompose.as_ref()),
            <crate::graph::flow_shims::end_task::EndTask as graph_flow::Task>::id(end.as_ref()),
        )
        .add_edge(
            <crate::graph::flow_shims::enhancement_task_shim::EnhancementTaskShim as graph_flow::Task>::id(enhance.as_ref()),
            <crate::graph::flow_shims::comprehension_test_task_shim::ComprehensionTestTaskShim as graph_flow::Task>::id(comprehend.as_ref()),
        )
        .add_edge(
            <crate::graph::flow_shims::comprehension_test_task_shim::ComprehensionTestTaskShim as graph_flow::Task>::id(comprehend.as_ref()),
            <crate::graph::flow_shims::check_test_result_task_shim::CheckTestResultTaskShim as graph_flow::Task>::id(check.as_ref()),
        )
        .add_conditional_edge(
            <crate::graph::flow_shims::check_test_result_task_shim::CheckTestResultTaskShim as graph_flow::Task>::id(check.as_ref()),
            |ctx| ctx.get_sync::<std::string::String>("routing_decision").unwrap_or_else(|| std::string::String::new()) == "pass",
            <crate::graph::flow_shims::end_task::EndTask as graph_flow::Task>::id(end.as_ref()),
            <crate::graph::flow_shims::enhancement_task_shim::EnhancementTaskShim as graph_flow::Task>::id(enhance.as_ref()),
        );

    builder
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

    struct MockDecomp;
    #[async_trait::async_trait]
    impl crate::ports::task_decomposition_port::TaskDecompositionPort for MockDecomp {
        async fn decompose_task(
            &self,
            task: &task_manager::domain::task::Task,
        ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String> {
            let subtasks: std::vec::Vec<task_manager::domain::task::Task> = (1..=3)
                .map(|i| {
                    let action = transcript_extractor::domain::action_item::ActionItem {
                        title: std::format!("Subtask {}: {}", i, task.title),
                        assignee: task.assignee.clone(),
                        due_date: task.due_date.clone(),
                    };
                    let mut subtask = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
                    subtask.parent_task_id = std::option::Option::Some(task.id.clone());
                    subtask
                })
                .collect();
            std::result::Result::Ok(subtasks)
        }
    }

    #[test]
    fn test_assemble_orchestrator_flow_builds() {
        let b = super::assemble_orchestrator_flow(
            std::sync::Arc::new(MockEnh),
            std::sync::Arc::new(MockCT),
            std::sync::Arc::new(MockDecomp),
            std::string::String::from("short_answer"),
        );
        // Ensure builder can build without panic
        let _graph = b.build();
    }
}
