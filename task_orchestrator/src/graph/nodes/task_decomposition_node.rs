//! TaskDecompositionNode for breaking complex tasks into subtasks.
//!
//! This node uses TaskDecompositionPort to analyze complex tasks and generate
//! 3-5 manageable subtasks via LLM. Decomposed subtasks are stored in GraphState
//! and the parent task status is updated to Decomposed.
//!
//! Revision History
//! - 2025-11-23T23:20:00Z @AI: Document Orca-2 usage for heterogeneous pipeline (Phase 5 Sprint 10 Task 5.5).
//! - 2025-11-23T17:30:00Z @AI: Create TaskDecompositionNode for Phase 3 Sprint 7.
//!
//! # Heterogeneous Pipeline Notes
//!
//! This node performs COMPLEX REASONING for task decomposition and should use
//! specialized models optimized for this purpose.
//!
//! **Recommended Model**: Orca-2 (7B)
//! - Trained with process imitation (recall-reason-generate)
//! - Excels at breaking down complex tasks into logical subtasks
//! - Superior reasoning quality compared to general-purpose models
//!
//! Use `ProviderFactory::create_task_decomposition_adapter_for_role(ModelRole::Decomposer)`
//! to automatically select Orca-2 for this node.

/// Node that decomposes complex tasks into subtasks using AI.
///
/// TaskDecompositionNode uses TaskDecompositionPort to break down high-complexity
/// tasks (typically score >= 7) into 3-5 actionable subtasks. The node stores
/// the generated subtasks in GraphState and updates the parent task's status.
///
/// # Decomposition Flow
///
/// 1. Receives GraphState with complex task (routed by SemanticRouterNode)
/// 2. Calls decomposition_port.decompose_task() to generate subtasks
/// 3. Stores subtasks in GraphState.subtasks field
/// 4. Updates parent task status to TaskStatus::Decomposed
/// 5. Sets parent task.subtask_ids with generated subtask IDs
/// 6. Returns updated GraphState
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::graph::nodes::task_decomposition_node::TaskDecompositionNode;
/// # use task_orchestrator::graph::state::GraphState;
/// # use task_orchestrator::ports::task_decomposition_port::TaskDecompositionPort;
/// # use task_manager::domain::task::Task;
/// # use transcript_extractor::domain::action_item::ActionItem;
/// # async fn example(decomposition_port: std::sync::Arc<dyn TaskDecompositionPort>) {
/// let node = TaskDecompositionNode::new(decomposition_port);
///
/// let action = ActionItem {
///     title: std::string::String::from("Refactor authentication system"),
///     assignee: std::option::Option::None,
///     due_date: std::option::Option::None,
/// };
/// let task = Task::from_action_item(&action, std::option::Option::None);
/// let state = GraphState::new(task);
///
/// let result = node.execute(state).await;
/// std::assert!(result.is_ok());
///
/// let output = result.unwrap();
/// std::assert!(output.subtasks.is_some(), "Should have generated subtasks");
/// # }
/// ```
pub struct TaskDecompositionNode {
    decomposition_port: std::sync::Arc<dyn crate::ports::task_decomposition_port::TaskDecompositionPort>,
}

impl TaskDecompositionNode {
    /// Creates a new TaskDecompositionNode with the provided decomposition port.
    ///
    /// # Arguments
    ///
    /// * `decomposition_port` - The port implementation for task decomposition
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::graph::nodes::task_decomposition_node::TaskDecompositionNode;
    /// # use task_orchestrator::adapters::rig_task_decomposition_adapter::RigTaskDecompositionAdapter;
    /// # use task_orchestrator::ports::task_decomposition_port::TaskDecompositionPort;
    /// let decomposer = std::sync::Arc::new(
    ///     RigTaskDecompositionAdapter::new(std::string::String::from("llama3.1"))
    /// );
    /// let node = TaskDecompositionNode::new(decomposer);
    /// ```
    pub fn new(
        decomposition_port: std::sync::Arc<dyn crate::ports::task_decomposition_port::TaskDecompositionPort>,
    ) -> Self {
        TaskDecompositionNode { decomposition_port }
    }

    /// Executes decomposition logic to break task into subtasks.
    ///
    /// Calls the decomposition port to generate 3-5 subtasks, stores them in
    /// GraphState, and updates the parent task to Decomposed status.
    ///
    /// # Arguments
    ///
    /// * `state` - The current GraphState containing the task to decompose
    ///
    /// # Returns
    ///
    /// Updated GraphState with:
    /// - `subtasks` field populated with generated subtasks
    /// - `task.status` set to TaskStatus::Decomposed
    /// - `task.subtask_ids` containing IDs of generated subtasks
    ///
    /// # Errors
    ///
    /// Returns error if decomposition port fails to generate subtasks.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::graph::nodes::task_decomposition_node::TaskDecompositionNode;
    /// # use task_orchestrator::graph::state::GraphState;
    /// # use task_orchestrator::ports::task_decomposition_port::TaskDecompositionPort;
    /// # use task_manager::domain::task::Task;
    /// # use transcript_extractor::domain::action_item::ActionItem;
    /// # async fn example(node: TaskDecompositionNode) {
    /// let action = ActionItem {
    ///     title: std::string::String::from("Implement OAuth2 with SAML"),
    ///     assignee: std::option::Option::None,
    ///     due_date: std::option::Option::None,
    /// };
    /// let task = Task::from_action_item(&action, std::option::Option::None);
    /// let state = GraphState::new(task);
    ///
    /// let output = node.execute(state).await.unwrap();
    /// std::println!("Generated {} subtasks", output.subtasks.as_ref().unwrap().len());
    /// # }
    /// ```
    pub async fn execute(
        &self,
        mut state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        // Decompose task into subtasks
        let subtasks = self.decomposition_port.decompose_task(&state.task).await?;

        // Extract subtask IDs for parent task linkage
        let subtask_ids: std::vec::Vec<String> = subtasks.iter().map(|st| st.id.clone()).collect();

        // Update parent task
        state.task.status = task_manager::domain::task_status::TaskStatus::Decomposed;
        state.task.subtask_ids = subtask_ids;

        // Store subtasks in GraphState
        state.subtasks = std::option::Option::Some(subtasks);

        std::result::Result::Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock decomposition port for testing.
    struct MockDecompositionPort;

    #[async_trait::async_trait]
    impl crate::ports::task_decomposition_port::TaskDecompositionPort for MockDecompositionPort {
        async fn decompose_task(
            &self,
            task: &task_manager::domain::task::Task,
        ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String> {
            // Generate 3 mock subtasks
            let subtask_titles = std::vec![
                std::format!("Design: {}", task.title),
                std::format!("Implement: {}", task.title),
                std::format!("Test: {}", task.title),
            ];

            let subtasks: std::vec::Vec<task_manager::domain::task::Task> = subtask_titles
                .into_iter()
                .map(|title| {
                    let action = transcript_extractor::domain::action_item::ActionItem {
                        title,
                        assignee: task.agent_persona.clone(),
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

    #[tokio::test]
    async fn test_decomposition_node_generates_subtasks() {
        // Test: Validates node generates subtasks and updates parent task.
        // Justification: Core decomposition functionality must work correctly.
        let mock_port = std::sync::Arc::new(MockDecompositionPort);
        let node = TaskDecompositionNode::new(mock_port);

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor authentication"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        let parent_id = task.id.clone();
        let state = crate::graph::state::GraphState::new(task);

        let result = node.execute(state).await;
        std::assert!(result.is_ok(), "Decomposition should succeed");

        let output = result.unwrap();

        // Verify subtasks generated
        std::assert!(output.subtasks.is_some(), "Subtasks should be present");
        let subtasks = output.subtasks.unwrap();
        std::assert_eq!(subtasks.len(), 3, "Should generate 3 subtasks");

        // Verify parent task updated
        std::assert_eq!(
            output.task.status,
            task_manager::domain::task_status::TaskStatus::Decomposed,
            "Parent task should be marked Decomposed"
        );
        std::assert_eq!(
            output.task.subtask_ids.len(),
            3,
            "Parent should reference 3 subtasks"
        );

        // Verify subtask linkage
        for subtask in &subtasks {
            std::assert_eq!(
                subtask.parent_task_id,
                std::option::Option::Some(parent_id.clone()),
                "Each subtask should link to parent"
            );
            std::assert!(
                output.task.subtask_ids.contains(&subtask.id),
                "Parent should reference subtask ID"
            );
        }
    }

    #[tokio::test]
    async fn test_decomposition_node_preserves_parent_context() {
        // Test: Validates parent task context is available to decomposition.
        // Justification: Subtasks should inherit relevant context from parent.
        let mock_port = std::sync::Arc::new(MockDecompositionPort);
        let node = TaskDecompositionNode::new(mock_port);

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Complex task"),
            assignee: std::option::Option::Some(std::string::String::from("QA Engineer")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-31")),
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);

        let result = node.execute(state).await.unwrap();

        let subtasks = result.subtasks.unwrap();
        for subtask in &subtasks {
            std::assert_eq!(
                subtask.agent_persona,
                std::option::Option::Some(std::string::String::from("QA Engineer")),
                "Subtasks should inherit parent agent_persona"
            );
            std::assert_eq!(
                subtask.due_date,
                std::option::Option::Some(std::string::String::from("2025-12-31")),
                "Subtasks should inherit parent due_date"
            );
        }
    }
}
