//! Port trait for generating comprehension tests for tasks.
//!
//! This trait defines the interface used by orchestrator nodes to request a
//! comprehension test for a given task. Implementations typically call an LLM
//! via an adapter and return structured results suitable for routing decisions.
//!
//! Revision History
//! - 2025-11-12T17:22:00Z @AI: Revert HexPort attribute; current hexser release does not support trait annotation.
//! - 2025-11-12T21:38:00Z @AI: Refactor to use async_trait with async fn; keep tests and NO `use` compliance.
//! - 2025-11-12T21:11:00Z @AI: Add ComprehensionTestPort trait with async API and unit test.

#[async_trait::async_trait]
/// Port for generating a comprehension test for a task.
///
/// Adapters implementing this port must return a structured test instance.
/// Errors are reported via `Err(String)` with actionable messages.
pub trait ComprehensionTestPort: std::marker::Send + std::marker::Sync {
    /// Generate a comprehension test of the specified `test_type` for `task`.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if generation fails.
    async fn generate_comprehension_test(
        &self,
        task: &task_manager::domain::task::Task,
        test_type: &str,
    ) -> std::result::Result<task_manager::domain::comprehension_test::ComprehensionTest, std::string::String>;
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_mock_comprehension_test_port() {
        struct MockPort;
        #[async_trait::async_trait]
        impl super::ComprehensionTestPort for MockPort {
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
                    question: std::format!("What is the core goal of '{}'?", task.title),
                    options: std::option::Option::None,
                    correct_answer: std::string::String::from("To be determined"),
                };
                std::result::Result::Ok(ct)
            }
        }

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        let port = MockPort;
        let res = <MockPort as super::ComprehensionTestPort>::generate_comprehension_test(&port, &task, "short_answer").await;
        std::assert!(res.is_ok());
        let ct = res.unwrap();
        std::assert_eq!(ct.task_id, task.id);
        std::assert_eq!(ct.test_type, std::string::String::from("short_answer"));
        std::assert!(ct.question.contains("core goal"));
    }
}
