//! Convenience function to run a task with injected ports (adapters).
//!
//! This helper enables callers to provide their own implementations of the
//! TaskEnhancementPort and ComprehensionTestPort. It wires them into the
//! existing sequential TaskGraphRunner to execute an end-to-end flow. This
//! keeps the system verifiable while the graph runtime wiring proceeds.
//!
//! Revision History
//! - 2025-11-13T08:31:00Z @AI: Add run_task_with_ports helper with unit test; no new dependencies.

/// Runs orchestration using injected ports and a specified test type.
///
/// This function constructs a `TaskGraphRunner` with the provided ports and
/// `test_type`, then executes the flow, returning the updated task on success.
///
/// # Arguments
///
/// * `enhancement_port` - Adapter implementing TaskEnhancementPort.
/// * `test_port` - Adapter implementing ComprehensionTestPort.
/// * `test_type` - The comprehension test type to request (e.g., "short_answer").
/// * `task` - The Task to orchestrate.
///
/// # Returns
///
/// * `Ok(Task)` - The updated task after orchestration.
/// * `Err(String)` - An error message if any node fails during execution.
pub async fn run_task_with_ports(
    enhancement_port: std::sync::Arc<dyn crate::ports::task_enhancement_port::TaskEnhancementPort>,
    test_port: std::sync::Arc<dyn crate::ports::comprehension_test_port::ComprehensionTestPort>,
    test_type: &str,
    task: task_manager::domain::task::Task,
) -> std::result::Result<task_manager::domain::task::Task, std::string::String> {
    let runner = crate::use_cases::task_graph_runner::TaskGraphRunner::new(
        enhancement_port,
        test_port,
        std::string::String::from(test_type),
    );

    crate::use_cases::task_graph_runner::TaskGraphRunner::run_task(&runner, task).await
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
                content: std::format!("Enhanced: {}", task.title),
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
    async fn test_run_task_with_ports_completes() {
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Compose release notes"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let result = super::run_task_with_ports(
            std::sync::Arc::new(MockEnh),
            std::sync::Arc::new(MockCT),
            "short_answer",
            task,
        ).await;
        std::assert!(result.is_ok());
        let updated = result.unwrap();
        std::assert!(updated.enhancements.is_some());
        std::assert!(updated.comprehension_tests.is_some());
    }
}
