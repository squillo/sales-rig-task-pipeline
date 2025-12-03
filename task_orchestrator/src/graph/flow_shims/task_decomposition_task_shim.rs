//! Shim for TaskDecompositionNode to integrate with graph_flow runtime.
//!
//! This shim wraps TaskDecompositionNode and implements the graph_flow::Task trait,
//! enabling the decomposition node to be used within the StateGraph orchestration
//! framework. It handles state marshalling between graph_flow::Context and GraphState.
//!
//! Revision History
//! - 2025-11-23T17:45:00Z @AI: Create TaskDecompositionTaskShim for Phase 3 Sprint 7.

/// Shim that wraps TaskDecompositionNode for graph-flow integration.
///
/// TaskDecompositionTaskShim implements the graph_flow::Task trait to enable
/// TaskDecompositionNode to participate in the StateGraph orchestration pipeline.
/// The shim handles:
/// - Converting graph_flow::Context to/from GraphState
/// - Storing the decomposition port for node construction
/// - Persisting subtasks back to Context for downstream nodes
pub struct TaskDecompositionTaskShim {
    decomposition_port: std::sync::Arc<dyn crate::ports::task_decomposition_port::TaskDecompositionPort>,
}

impl TaskDecompositionTaskShim {
    /// Creates a new TaskDecompositionTaskShim with the provided decomposition port.
    ///
    /// # Arguments
    ///
    /// * `decomposition_port` - The port implementation for task decomposition
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::graph::flow_shims::task_decomposition_task_shim::TaskDecompositionTaskShim;
    /// # use task_orchestrator::adapters::rig_task_decomposition_adapter::RigTaskDecompositionAdapter;
    /// let decomposer = std::sync::Arc::new(
    ///     RigTaskDecompositionAdapter::new(std::string::String::from("llama3.1"))
    /// );
    /// let shim = TaskDecompositionTaskShim::new(decomposer);
    /// ```
    pub fn new(
        decomposition_port: std::sync::Arc<dyn crate::ports::task_decomposition_port::TaskDecompositionPort>,
    ) -> Self {
        TaskDecompositionTaskShim { decomposition_port }
    }

    /// Runs the decomposition logic by delegating to TaskDecompositionNode::execute.
    ///
    /// Internal method used by the graph_flow::Task trait implementation.
    pub async fn run(
        &self,
        state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        let node = crate::graph::nodes::task_decomposition_node::TaskDecompositionNode::new(
            self.decomposition_port.clone(),
        );
        crate::graph::nodes::task_decomposition_node::TaskDecompositionNode::execute(&node, state).await
    }
}

#[async_trait::async_trait]
impl graph_flow::Task for TaskDecompositionTaskShim {
    async fn run(&self, context: graph_flow::Context) -> graph_flow::Result<graph_flow::TaskResult> {
        // Retrieve task from context
        let maybe_task: std::option::Option<task_manager::domain::task::Task> = context.get("task").await;
        let task = match maybe_task {
            std::option::Option::Some(t) => t,
            std::option::Option::None => {
                return std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(
                    std::string::String::from("No task found in context for decomposition"),
                ));
            }
        };

        // Run decomposition node logic
        let state_in = crate::graph::state::GraphState::new(task);
        let state_out = match TaskDecompositionTaskShim::run(self, state_in).await {
            std::result::Result::Ok(s) => s,
            std::result::Result::Err(e) => {
                return std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(e));
            }
        };

        // Persist updated task back to context
        context.set("task", state_out.task.clone()).await;

        // Persist subtasks to context for potential downstream nodes
        if let std::option::Option::Some(subtasks) = state_out.subtasks.clone() {
            context.set("subtasks", subtasks).await;
        }

        std::result::Result::Ok(graph_flow::TaskResult::new(
            std::option::Option::None,
            graph_flow::NextAction::Continue,
        ))
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
            let subtasks: std::vec::Vec<task_manager::domain::task::Task> = (1..=3)
                .map(|i| {
                    let action = transcript_extractor::domain::action_item::ActionItem {
                        title: std::format!("Subtask {}: {}", i, task.title),
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
    async fn test_decomposition_shim_runs_successfully() {
        // Test: Validates shim executes decomposition and returns success.
        // Justification: Shim must correctly delegate to node and handle results.
        let mock_port = std::sync::Arc::new(MockDecompositionPort);
        let shim = TaskDecompositionTaskShim::new(mock_port);

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Complex task"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);

        let result = TaskDecompositionTaskShim::run(&shim, state).await;
        std::assert!(result.is_ok(), "Shim should execute successfully");

        let output = result.unwrap();
        std::assert!(output.subtasks.is_some(), "Should have generated subtasks");
        std::assert_eq!(
            output.task.status,
            task_manager::domain::task_status::TaskStatus::Decomposed,
            "Task should be marked Decomposed"
        );
    }

    #[tokio::test]
    async fn test_task_impl_persists_subtasks_in_context() {
        // Test: Validates graph_flow::Task implementation persists subtasks to Context.
        // Justification: Downstream nodes may need access to generated subtasks.
        let mock_port = std::sync::Arc::new(MockDecompositionPort);
        let shim = TaskDecompositionTaskShim::new(mock_port);

        let ctx = graph_flow::Context::new();

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor system"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        let task_id = task.id.clone();

        ctx.set("task", task).await;

        let result = <TaskDecompositionTaskShim as graph_flow::Task>::run(&shim, ctx.clone())
            .await
            .unwrap();

        std::assert!(matches!(result.next_action, graph_flow::NextAction::Continue));

        // Verify task updated in context
        let updated_task: std::option::Option<task_manager::domain::task::Task> = ctx.get("task").await;
        std::assert!(updated_task.is_some(), "Task should be in context");
        let task = updated_task.unwrap();
        std::assert_eq!(
            task.status,
            task_manager::domain::task_status::TaskStatus::Decomposed,
            "Task should be Decomposed"
        );
        std::assert_eq!(task.id, task_id, "Task ID should be preserved");

        // Verify subtasks persisted to context
        let subtasks: std::option::Option<std::vec::Vec<task_manager::domain::task::Task>> = ctx.get("subtasks").await;
        std::assert!(subtasks.is_some(), "Subtasks should be in context");
        let subtasks = subtasks.unwrap();
        std::assert_eq!(subtasks.len(), 3, "Should have 3 subtasks");

        for subtask in &subtasks {
            std::assert_eq!(
                subtask.parent_task_id,
                std::option::Option::Some(task_id.clone()),
                "Each subtask should link to parent"
            );
        }
    }
}
