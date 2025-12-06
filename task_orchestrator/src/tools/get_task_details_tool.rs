//! Tool for retrieving detailed information about a specific task.
//!
//! GetTaskDetailsTool allows Rig agents to fetch complete task information
//! including description, complexity, dependencies, reasoning, and more. This
//! enables deep inspection of task details for answering detailed questions.
//!
//! Revision History
//! - 2025-12-03T00:00:00Z @AI: Create GetTaskDetailsTool for LLM agent task inspection.

/// Error type for task details operations.
#[derive(Debug, Clone)]
pub enum GetTaskDetailsError {
    /// Repository query failed
    RepositoryError(std::string::String),
    /// Task not found
    NotFound(std::string::String),
    /// Invalid parameters
    InvalidParameters(std::string::String),
}

impl std::fmt::Display for GetTaskDetailsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTaskDetailsError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            GetTaskDetailsError::NotFound(msg) => write!(f, "Not found: {}", msg),
            GetTaskDetailsError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
        }
    }
}

impl std::error::Error for GetTaskDetailsError {}

/// Arguments for get_task_details tool.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct GetTaskDetailsArgs {
    /// Task ID to retrieve (can be partial ID matching first 8 characters)
    pub task_id: std::string::String,
}

/// Tool for retrieving detailed task information by ID.
///
/// This tool enables Rig agents to fetch complete information about a specific
/// task, including all metadata, dependencies, reasoning, and context files.
///
/// # Examples
///
/// ```ignore
/// let tool = GetTaskDetailsTool::new(task_repo);
/// let details = tool.call(GetTaskDetailsArgs {
///     task_id: "550e8400".to_string(), // Partial ID match
/// }).await?;
/// ```
#[derive(Clone)]
pub struct GetTaskDetailsTool {
    task_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::task_repository_port::TaskRepositoryPort + std::marker::Send>>,
}

impl GetTaskDetailsTool {
    /// Creates a new GetTaskDetailsTool.
    ///
    /// # Arguments
    ///
    /// * `task_repository` - Repository for task storage and queries
    ///
    /// # Returns
    ///
    /// A new GetTaskDetailsTool instance.
    pub fn new(
        task_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::task_repository_port::TaskRepositoryPort + std::marker::Send>>,
    ) -> Self {
        Self { task_repository }
    }

    /// Retrieves detailed information about a task.
    ///
    /// # Arguments
    ///
    /// * `task_id` - Task ID (full or partial matching first 8 chars)
    ///
    /// # Returns
    ///
    /// Formatted string containing complete task details.
    pub async fn get_details(
        &self,
        task_id: &str,
    ) -> std::result::Result<std::string::String, GetTaskDetailsError> {
        // Validate parameters
        if task_id.is_empty() {
            return std::result::Result::Err(GetTaskDetailsError::InvalidParameters(
                std::string::String::from("Task ID cannot be empty")
            ));
        }

        // Query repository
        let repo = self.task_repository.lock()
            .map_err(|e| GetTaskDetailsError::RepositoryError(std::format!("Lock error: {}", e)))?;

        // Try exact match first
        let filter = task_manager::ports::task_repository_port::TaskFilter::ById(task_id.to_string());
        let options = hexser::ports::repository::FindOptions {
            sort: std::option::Option::None,
            limit: std::option::Option::Some(1),
            offset: std::option::Option::None,
        };

        let mut tasks = repo.find(&filter, options)
            .map_err(|e| GetTaskDetailsError::RepositoryError(std::format!("{:?}", e)))?;

        // If no exact match, try partial match on all tasks
        if tasks.is_empty() {
            let all_filter = task_manager::ports::task_repository_port::TaskFilter::All;
            let all_options = hexser::ports::repository::FindOptions {
                sort: std::option::Option::None,
                limit: std::option::Option::Some(100),
                offset: std::option::Option::None,
            };

            let all_tasks = repo.find(&all_filter, all_options)
                .map_err(|e| GetTaskDetailsError::RepositoryError(std::format!("{:?}", e)))?;

            tasks = all_tasks.into_iter()
                .filter(|t| t.id.starts_with(task_id))
                .collect();
        }

        if tasks.is_empty() {
            return std::result::Result::Err(GetTaskDetailsError::NotFound(
                std::format!("No task found with ID: {}", task_id)
            ));
        }

        let task = &tasks[0];

        // Format detailed output
        let mut result = std::format!("# Task Details: {}\n\n", task.title);
        result.push_str(&std::format!("**ID:** `{}`\n", task.id));
        result.push_str(&std::format!("**Status:** {}\n", self.format_status(&task.status)));

        if let std::option::Option::Some(ref persona) = task.agent_persona {
            result.push_str(&std::format!("**Assignee:** {}\n", persona));
        }

        if let std::option::Option::Some(complexity) = task.complexity {
            result.push_str(&std::format!("**Complexity:** {}/10\n", complexity));
        }

        if let std::option::Option::Some(ref due_date) = task.due_date {
            result.push_str(&std::format!("**Due Date:** {}\n", due_date));
        }

        // Description
        if !task.description.is_empty() {
            result.push_str(&std::format!("\n## Description\n\n{}\n", task.description));
        }

        // Reasoning
        if let std::option::Option::Some(ref reasoning) = task.reasoning {
            result.push_str(&std::format!("\n## Reasoning\n\n{}\n", reasoning));
        }

        // Dependencies
        if !task.dependencies.is_empty() {
            result.push_str("\n## Dependencies\n\n");
            for dep in &task.dependencies {
                result.push_str(&std::format!("- {}\n", dep));
            }
        }

        // Context files
        if !task.context_files.is_empty() {
            result.push_str("\n## Context Files\n\n");
            for file in &task.context_files {
                result.push_str(&std::format!("- `{}`\n", file));
            }
        }

        // Parent/subtask relationships
        if let std::option::Option::Some(ref parent_id) = task.parent_task_id {
            result.push_str(&std::format!("\n**Parent Task:** {}\n", parent_id));
        }

        if !task.subtask_ids.is_empty() {
            result.push_str(&std::format!("\n**Subtasks:** {} subtask(s)\n", task.subtask_ids.len()));
            for subtask_id in &task.subtask_ids {
                result.push_str(&std::format!("- {}\n", subtask_id));
            }
        }

        // Completion summary if completed
        if let std::option::Option::Some(ref summary) = task.completion_summary {
            result.push_str(&std::format!("\n## Completion Summary\n\n{}\n", summary));
        }

        // Timestamps
        result.push_str(&std::format!(
            "\n---\n*Created:* {} | *Updated:* {}\n",
            task.created_at.format("%Y-%m-%d %H:%M"),
            task.updated_at.format("%Y-%m-%d %H:%M")
        ));

        std::result::Result::Ok(result)
    }

    /// Formats TaskStatus enum as human-readable string.
    fn format_status(&self, status: &task_manager::domain::task_status::TaskStatus) -> &'static str {
        match status {
            task_manager::domain::task_status::TaskStatus::Todo => "Todo",
            task_manager::domain::task_status::TaskStatus::InProgress => "In Progress",
            task_manager::domain::task_status::TaskStatus::Completed => "Completed",
            task_manager::domain::task_status::TaskStatus::Archived => "Archived",
            task_manager::domain::task_status::TaskStatus::Errored => "Errored",
            _ => "Other",
        }
    }
}

#[allow(refining_impl_trait)]
impl rig::tool::Tool for GetTaskDetailsTool {
    const NAME: &'static str = "get_task_details";

    type Error = GetTaskDetailsError;
    type Args = GetTaskDetailsArgs;
    type Output = std::string::String;

    fn definition(&self, _prompt: std::string::String) -> impl std::future::Future<Output = rig::completion::ToolDefinition> + Send + Sync {
        async {
            rig::completion::ToolDefinition {
                name: Self::NAME.to_string(),
                description: "Retrieves detailed information about a specific task including description, complexity, dependencies, reasoning, and more. Use this when you need complete information about a task.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "The task ID to retrieve. Can be a full ID or partial ID (first 8 characters)."
                        }
                    },
                    "required": ["task_id"]
                }),
            }
        }
    }

    fn call(&self, args: Self::Args) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send + Sync>> {
        let tool = self.clone();
        std::boxed::Box::pin(async move {
            let handle = tokio::spawn(async move {
                tool.get_details(&args.task_id).await
            });
            handle.await
                .map_err(|e| GetTaskDetailsError::RepositoryError(std::format!("Task join error: {}", e)))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock task repository for testing.
    struct MockTaskRepository {
        tasks: std::vec::Vec<task_manager::domain::task::Task>,
    }

    impl hexser::ports::Repository<task_manager::domain::task::Task> for MockTaskRepository {
        fn save(&mut self, entity: task_manager::domain::task::Task) -> hexser::HexResult<()> {
            self.tasks.push(entity);
            std::result::Result::Ok(())
        }
    }

    impl hexser::ports::repository::QueryRepository<task_manager::domain::task::Task> for MockTaskRepository {
        type Filter = task_manager::ports::task_repository_port::TaskFilter;
        type SortKey = task_manager::ports::task_repository_port::TaskSortKey;

        fn find_one(&self, _filter: &Self::Filter) -> hexser::HexResult<std::option::Option<task_manager::domain::task::Task>> {
            std::result::Result::Ok(std::option::Option::None)
        }

        fn find(&self, filter: &Self::Filter, _options: hexser::ports::repository::FindOptions<Self::SortKey>) -> hexser::HexResult<std::vec::Vec<task_manager::domain::task::Task>> {
            let filtered: std::vec::Vec<_> = match filter {
                task_manager::ports::task_repository_port::TaskFilter::ById(id) => {
                    self.tasks.iter().filter(|t| &t.id == id).cloned().collect()
                }
                task_manager::ports::task_repository_port::TaskFilter::All => {
                    self.tasks.clone()
                }
                _ => std::vec::Vec::new(),
            };
            std::result::Result::Ok(filtered)
        }
    }

    impl task_manager::ports::task_repository_port::TaskRepositoryPort for MockTaskRepository {}

    #[tokio::test]
    async fn test_get_task_details_exact_match() {
        // Test: Validates exact ID match retrieves task.
        // Justification: Core functionality.
        let mut repo = MockTaskRepository { tasks: std::vec::Vec::new() };
        let task_id = std::string::String::from("550e8400-e29b-41d4-a716-446655440000");

        hexser::ports::Repository::save(&mut repo, task_manager::domain::task::Task {
            id: task_id.clone(),
            title: std::string::String::from("Test Task"),
            description: std::string::String::from("This is a test task description."),
            agent_persona: std::option::Option::Some(std::string::String::from("Backend Developer")),
            due_date: std::option::Option::None,
            status: task_manager::domain::task_status::TaskStatus::InProgress,
            source_transcript_id: std::option::Option::None,
            source_prd_id: std::option::Option::None,
            parent_task_id: std::option::Option::None,
            subtask_ids: std::vec::Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            enhancements: std::option::Option::None,
            comprehension_tests: std::option::Option::None,
            complexity: std::option::Option::Some(5),
            reasoning: std::option::Option::Some(std::string::String::from("Reasoning here")),
            completion_summary: std::option::Option::None,
            context_files: std::vec::Vec::new(),
            dependencies: std::vec![std::string::String::from("task-123")],
            sort_order: std::option::Option::None,
        }).unwrap();

        let tool = GetTaskDetailsTool::new(
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
        );

        let result = tool.get_details(&task_id).await;
        std::assert!(result.is_ok());

        let output = result.unwrap();
        std::assert!(output.contains("Test Task"));
        std::assert!(output.contains("Backend Developer"));
        std::assert!(output.contains("Complexity: 5/10"));
        std::assert!(output.contains("task-123"));
    }

    #[tokio::test]
    async fn test_get_task_details_partial_match() {
        // Test: Validates partial ID match works.
        // Justification: Users often use short IDs.
        let mut repo = MockTaskRepository { tasks: std::vec::Vec::new() };
        let full_id = std::string::String::from("550e8400-e29b-41d4-a716-446655440000");

        hexser::ports::Repository::save(&mut repo, task_manager::domain::task::Task {
            id: full_id.clone(),
            title: std::string::String::from("Partial Match Test"),
            description: std::string::String::new(),
            agent_persona: std::option::Option::None,
            due_date: std::option::Option::None,
            status: task_manager::domain::task_status::TaskStatus::Todo,
            source_transcript_id: std::option::Option::None,
            source_prd_id: std::option::Option::None,
            parent_task_id: std::option::Option::None,
            subtask_ids: std::vec::Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            enhancements: std::option::Option::None,
            comprehension_tests: std::option::Option::None,
            complexity: std::option::Option::None,
            reasoning: std::option::Option::None,
            completion_summary: std::option::Option::None,
            context_files: std::vec::Vec::new(),
            dependencies: std::vec::Vec::new(),
            sort_order: std::option::Option::None,
        }).unwrap();

        let tool = GetTaskDetailsTool::new(
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
        );

        let result = tool.get_details("550e8400").await;
        std::assert!(result.is_ok());

        let output = result.unwrap();
        std::assert!(output.contains("Partial Match Test"));
    }

    #[tokio::test]
    async fn test_get_task_details_not_found() {
        // Test: Validates error when task doesn't exist.
        // Justification: Must handle missing tasks gracefully.
        let repo = MockTaskRepository { tasks: std::vec::Vec::new() };
        let tool = GetTaskDetailsTool::new(
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
        );

        let result = tool.get_details("nonexistent-id").await;
        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().to_string().contains("No task found"));
    }
}
