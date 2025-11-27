//! Port for task decomposition services.
//!
//! TaskDecompositionPort defines the interface for breaking complex tasks
//! into manageable subtasks. Implementations use LLMs to analyze task
//! complexity and generate 3-5 actionable subtasks with parent linkage.
//!
//! Revision History
//! - 2025-11-23T17:00:00Z @AI: Create TaskDecompositionPort for Phase 3 Sprint 7.

/// Port for generating task decomposition using AI services.
///
/// TaskDecompositionPort abstracts the complexity of breaking down
/// high-complexity tasks into smaller, manageable subtasks. The port
/// enables different decomposition strategies (LLM-based, heuristic, etc.)
/// while maintaining a consistent interface for orchestration nodes.
///
/// # Decomposition Requirements
///
/// Implementations must:
/// - Generate 3-5 subtasks per parent task
/// - Set `parent_task_id` on each subtask linking to original task
/// - Reduce complexity score (typically parent_complexity - 2)
/// - Preserve context from parent task (assignee, due_date if applicable)
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::ports::task_decomposition_port::TaskDecompositionPort;
/// # use task_manager::domain::task::Task;
/// # use transcript_extractor::domain::action_item::ActionItem;
/// # async fn example(decomposer: std::sync::Arc<dyn TaskDecompositionPort>) {
/// let parent_action = ActionItem {
///     title: std::string::String::from("Refactor authentication system"),
///     assignee: std::option::Option::None,
///     due_date: std::option::Option::None,
/// };
/// let parent_task = Task::from_action_item(&parent_action, std::option::Option::None);
///
/// let subtasks = decomposer.decompose_task(&parent_task).await.unwrap();
/// std::assert!(subtasks.len() >= 3 && subtasks.len() <= 5, "Should generate 3-5 subtasks");
///
/// for subtask in &subtasks {
///     std::assert_eq!(subtask.parent_task_id, std::option::Option::Some(parent_task.id.clone()));
/// }
/// # }
/// ```
#[async_trait::async_trait]
pub trait TaskDecompositionPort: std::marker::Send + std::marker::Sync {
    /// Decomposes a complex task into 3-5 manageable subtasks.
    ///
    /// Analyzes the parent task and generates subtasks that collectively
    /// achieve the parent task's objective. Each subtask is linked to the
    /// parent via `parent_task_id` field.
    ///
    /// # Arguments
    ///
    /// * `task` - The parent task to decompose (typically complexity >= 7)
    ///
    /// # Returns
    ///
    /// A Result containing:
    /// - `Ok(Vec<Task>)`: 3-5 subtasks with parent linkage
    /// - `Err(String)`: Error message if decomposition fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - LLM service is unavailable
    /// - Task complexity is too low for decomposition
    /// - Generated subtasks fail validation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::ports::task_decomposition_port::TaskDecompositionPort;
    /// # use task_manager::domain::task::Task;
    /// # use transcript_extractor::domain::action_item::ActionItem;
    /// # async fn example(decomposer: std::sync::Arc<dyn TaskDecompositionPort>) {
    /// let action = ActionItem {
    ///     title: std::string::String::from("Implement OAuth2 authentication"),
    ///     assignee: std::option::Option::None,
    ///     due_date: std::option::Option::Some(std::string::String::from("2025-12-31")),
    /// };
    /// let task = Task::from_action_item(&action, std::option::Option::None);
    ///
    /// match decomposer.decompose_task(&task).await {
    ///     std::result::Result::Ok(subtasks) => {
    ///         std::println!("Decomposed into {} subtasks", subtasks.len());
    ///         for (i, st) in subtasks.iter().enumerate() {
    ///             std::println!("  Subtask {}: {}", i + 1, st.title);
    ///         }
    ///     }
    ///     std::result::Result::Err(e) => std::eprintln!("Decomposition failed: {}", e),
    /// }
    /// # }
    /// ```
    async fn decompose_task(
        &self,
        task: &task_manager::domain::task::Task,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String>;
}
