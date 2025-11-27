//! Port trait for generating task enhancements.
//!
//! This trait defines the interface used by orchestrator nodes to request a
//! model-generated enhancement for a given task. Implementations should be pure
//! adapters that call external systems (e.g., LLMs) and return structured results.
//!
//! Revision History
//! - 2025-11-12T17:22:00Z @AI: Revert HexPort attribute; not supported on traits in current hexser; retain async trait.
//! - 2025-11-12T21:20:00Z @AI: Refactor to use async_trait with async fn; keep tests and NO `use` compliance.
//! - 2025-11-12T21:07:00Z @AI: Add TaskEnhancementPort trait with async API and unit test.

#[async_trait::async_trait]
/// Port for generating a single enhancement for a task.
///
/// Adapters implementing this port are responsible for producing a structured
/// enhancement for the provided task. Errors must be reported via `Err(String)`
/// with a clear message (C-GOOD-ERR).
pub trait TaskEnhancementPort: std::marker::Send + std::marker::Sync {
    /// Generate an enhancement for the specified task.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if generation fails.
    async fn generate_enhancement(
        &self,
        task: &task_manager::domain::task::Task,
    ) -> std::result::Result<task_manager::domain::enhancement::Enhancement, std::string::String>;
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_mock_enhancement_port() {
        struct MockPort;
        #[async_trait::async_trait]
        impl super::TaskEnhancementPort for MockPort {
            async fn generate_enhancement(
                &self,
                task: &task_manager::domain::task::Task,
            ) -> std::result::Result<task_manager::domain::enhancement::Enhancement, std::string::String> {
                let enhancement = task_manager::domain::enhancement::Enhancement {
                    enhancement_id: std::string::String::from("e-1"),
                    task_id: task.id.clone(),
                    timestamp: chrono::Utc::now(),
                    enhancement_type: std::string::String::from("rewrite"),
                    content: std::format!("Enhanced: {}", task.title),
                };
                std::result::Result::Ok(enhancement)
            }
        }

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Title"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        let port = MockPort;
        let res = <MockPort as super::TaskEnhancementPort>::generate_enhancement(&port, &task).await;
        std::assert!(res.is_ok());
        let e = res.unwrap();
        std::assert_eq!(e.task_id, task.id);
        std::assert_eq!(e.enhancement_type, std::string::String::from("rewrite"));
    }
}
