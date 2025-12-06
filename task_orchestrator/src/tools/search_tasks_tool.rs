//! Task search tool for finding tasks by title, status, or assignee.
//!
//! SearchTasksTool allows Rig agents to query the task database using fuzzy
//! search on titles and exact matching on status and assignee fields. This
//! enables agents to answer questions about task status and assignments.
//!
//! Revision History
//! - 2025-12-03T00:00:00Z @AI: Create SearchTasksTool for LLM agent task querying.

/// Error type for task search operations.
#[derive(Debug, Clone)]
pub enum SearchTasksError {
    /// Repository query failed
    RepositoryError(std::string::String),
    /// Invalid search parameters
    InvalidParameters(std::string::String),
}

impl std::fmt::Display for SearchTasksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchTasksError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            SearchTasksError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
        }
    }
}

impl std::error::Error for SearchTasksError {}

/// Arguments for search_tasks tool.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct SearchTasksArgs {
    /// Query to search in task titles (substring match)
    #[serde(default)]
    pub query: std::option::Option<std::string::String>,

    /// Filter by task status (e.g., "Todo", "InProgress", "Completed")
    #[serde(default)]
    pub status: std::option::Option<std::string::String>,

    /// Filter by agent persona/assignee
    #[serde(default)]
    pub agent_persona: std::option::Option<std::string::String>,

    /// Maximum number of results to return (default: 10, max: 50)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

/// Task search tool for finding tasks in the system.
///
/// This tool enables Rig agents to search for tasks using various filters
/// including title queries, status, and assignee. It provides formatted
/// output suitable for LLM consumption.
///
/// # Examples
///
/// ```ignore
/// let tool = SearchTasksTool::new(task_repo, Some("project-123"));
/// let results = tool.call(SearchTasksArgs {
///     query: Some("authentication".to_string()),
///     status: Some("InProgress".to_string()),
///     agent_persona: None,
///     limit: 10,
/// }).await?;
/// ```
#[derive(Clone)]
pub struct SearchTasksTool {
    task_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::task_repository_port::TaskRepositoryPort + std::marker::Send>>,
    project_id: std::option::Option<std::string::String>,
}

impl SearchTasksTool {
    /// Creates a new SearchTasksTool.
    ///
    /// # Arguments
    ///
    /// * `task_repository` - Repository for task storage and queries
    /// * `project_id` - Optional project ID to scope search (None = search all projects)
    ///
    /// # Returns
    ///
    /// A new SearchTasksTool instance.
    pub fn new(
        task_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::task_repository_port::TaskRepositoryPort + std::marker::Send>>,
        project_id: std::option::Option<std::string::String>,
    ) -> Self {
        Self {
            task_repository,
            project_id,
        }
    }

    /// Performs task search with provided filters.
    ///
    /// # Arguments
    ///
    /// * `query` - Optional substring to search in task titles
    /// * `status` - Optional status filter
    /// * `agent_persona` - Optional assignee filter
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Formatted string containing search results.
    pub async fn search(
        &self,
        query: std::option::Option<std::string::String>,
        status: std::option::Option<std::string::String>,
        agent_persona: std::option::Option<std::string::String>,
        limit: usize,
    ) -> std::result::Result<std::string::String, SearchTasksError> {
        // Validate parameters
        if limit == 0 || limit > 50 {
            return std::result::Result::Err(SearchTasksError::InvalidParameters(
                std::format!("Limit must be between 1 and 50, got {}", limit)
            ));
        }

        // Parse status if provided
        let status_filter = if let std::option::Option::Some(ref status_str) = status {
            std::option::Option::Some(self.parse_task_status(status_str)?)
        } else {
            std::option::Option::None
        };

        // Query repository
        let repo = self.task_repository.lock()
            .map_err(|e| SearchTasksError::RepositoryError(std::format!("Lock error: {}", e)))?;

        // Build filter based on parameters
        let filter = if let std::option::Option::Some(ref persona) = agent_persona {
            task_manager::ports::task_repository_port::TaskFilter::ByAgentPersona(persona.clone())
        } else if let std::option::Option::Some(ref status) = status_filter {
            task_manager::ports::task_repository_port::TaskFilter::ByStatus(status.clone())
        } else {
            task_manager::ports::task_repository_port::TaskFilter::All
        };

        let options = hexser::ports::repository::FindOptions {
            sort: std::option::Option::Some(std::vec![
                hexser::ports::repository::Sort {
                    key: task_manager::ports::task_repository_port::TaskSortKey::UpdatedAt,
                    direction: hexser::ports::repository::Direction::Desc,
                }
            ]),
            limit: std::option::Option::Some(limit as u32),
            offset: std::option::Option::None,
        };

        let mut tasks = repo.find(&filter, options)
            .map_err(|e| SearchTasksError::RepositoryError(std::format!("{:?}", e)))?;

        // Apply query filter if provided (substring match on title)
        if let std::option::Option::Some(ref q) = query {
            let q_lower = q.to_lowercase();
            tasks.retain(|task| task.title.to_lowercase().contains(&q_lower));
        }

        // Apply project filter if set
        if let std::option::Option::Some(ref proj_id) = self.project_id {
            // Filter tasks by project through PRD relationship
            tasks.retain(|task| {
                // TODO: Need to look up task's PRD to check project_id
                // For now, keep all tasks if we can't verify
                // This will be enhanced when we have PRD lookup capability
                true
            });
        }

        // Format results
        if tasks.is_empty() {
            return std::result::Result::Ok(std::string::String::from(
                "No tasks found matching your search criteria."
            ));
        }

        let mut result = std::format!("Found {} task(s):\n\n", tasks.len());

        for (i, task) in tasks.iter().enumerate() {
            let status_str = self.format_task_status(&task.status);
            let assignee_str = task.agent_persona.as_ref()
                .map(|a| a.as_str())
                .unwrap_or("Unassigned");

            result.push_str(&std::format!(
                "{}. [{}] {}\n   Status: {} | Assignee: {}\n",
                i + 1,
                &task.id[..8], // Show first 8 chars of ID
                task.title,
                status_str,
                assignee_str
            ));

            // Add description preview if available
            if !task.description.is_empty() {
                let preview = if task.description.len() > 100 {
                    std::format!("{}...", &task.description[..100])
                } else {
                    task.description.clone()
                };
                result.push_str(&std::format!("   {}\n", preview));
            }

            result.push('\n');
        }

        std::result::Result::Ok(result)
    }

    /// Parses a status string into TaskStatus enum.
    fn parse_task_status(
        &self,
        status: &str,
    ) -> std::result::Result<task_manager::domain::task_status::TaskStatus, SearchTasksError> {
        match status.to_lowercase().as_str() {
            "todo" => std::result::Result::Ok(task_manager::domain::task_status::TaskStatus::Todo),
            "inprogress" | "in_progress" | "in progress" => {
                std::result::Result::Ok(task_manager::domain::task_status::TaskStatus::InProgress)
            }
            "completed" => std::result::Result::Ok(task_manager::domain::task_status::TaskStatus::Completed),
            "archived" => std::result::Result::Ok(task_manager::domain::task_status::TaskStatus::Archived),
            "errored" | "error" => std::result::Result::Ok(task_manager::domain::task_status::TaskStatus::Errored),
            _ => std::result::Result::Err(SearchTasksError::InvalidParameters(
                std::format!("Invalid status: '{}'. Valid values: Todo, InProgress, Completed, Archived, Errored", status)
            )),
        }
    }

    /// Formats TaskStatus enum as human-readable string.
    fn format_task_status(&self, status: &task_manager::domain::task_status::TaskStatus) -> &'static str {
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
impl rig::tool::Tool for SearchTasksTool {
    const NAME: &'static str = "search_tasks";

    type Error = SearchTasksError;
    type Args = SearchTasksArgs;
    type Output = std::string::String;

    fn definition(&self, _prompt: std::string::String) -> impl std::future::Future<Output = rig::completion::ToolDefinition> + Send + Sync {
        async {
            rig::completion::ToolDefinition {
                name: Self::NAME.to_string(),
                description: "Searches for tasks in the system by title, status, or assignee. Use this to answer questions about what tasks exist, who is working on what, or the status of specific work items.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Substring to search in task titles (e.g., 'authentication', 'database', 'API')"
                        },
                        "status": {
                            "type": "string",
                            "description": "Filter by task status. Valid values: Todo, InProgress, Completed, Archived, Errored",
                            "enum": ["Todo", "InProgress", "Completed", "Archived", "Errored"]
                        },
                        "agent_persona": {
                            "type": "string",
                            "description": "Filter by assignee/agent persona (e.g., 'Backend Developer', 'Frontend Developer')"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (1-50, default: 10)",
                            "minimum": 1,
                            "maximum": 50,
                            "default": 10
                        }
                    },
                    "required": []
                }),
            }
        }
    }

    fn call(&self, args: Self::Args) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send + Sync>> {
        let tool = self.clone();
        std::boxed::Box::pin(async move {
            let handle = tokio::spawn(async move {
                tool.search(args.query, args.status, args.agent_persona, args.limit).await
            });
            handle.await
                .map_err(|e| SearchTasksError::RepositoryError(std::format!("Task join error: {}", e)))?
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
                task_manager::ports::task_repository_port::TaskFilter::ByStatus(status) => {
                    self.tasks.iter().filter(|t| &t.status == status).cloned().collect()
                }
                task_manager::ports::task_repository_port::TaskFilter::ByAgentPersona(persona) => {
                    self.tasks.iter().filter(|t| {
                        t.agent_persona.as_ref().map_or(false, |p| p == persona)
                    }).cloned().collect()
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

    fn create_test_task(id: &str, title: &str, status: task_manager::domain::task_status::TaskStatus, persona: std::option::Option<&str>) -> task_manager::domain::task::Task {
        task_manager::domain::task::Task {
            id: std::string::String::from(id),
            title: std::string::String::from(title),
            description: std::string::String::new(),
            agent_persona: persona.map(std::string::String::from),
            due_date: std::option::Option::None,
            status,
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
        }
    }

    #[tokio::test]
    async fn test_search_by_title() {
        // Test: Validates title substring search works.
        // Justification: Core search functionality.
        let mut repo = MockTaskRepository { tasks: std::vec::Vec::new() };
        hexser::ports::Repository::save(&mut repo, create_test_task(
            "task-1",
            "Implement authentication API",
            task_manager::domain::task_status::TaskStatus::InProgress,
            std::option::Option::Some("Backend Developer"),
        )).unwrap();
        hexser::ports::Repository::save(&mut repo, create_test_task(
            "task-2",
            "Design database schema",
            task_manager::domain::task_status::TaskStatus::Todo,
            std::option::Option::None,
        )).unwrap();

        let tool = SearchTasksTool::new(
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
            std::option::Option::None,
        );

        let result = tool.search(
            std::option::Option::Some(std::string::String::from("auth")),
            std::option::Option::None,
            std::option::Option::None,
            10,
        ).await;

        std::assert!(result.is_ok());
        let output = result.unwrap();
        std::assert!(output.contains("authentication"));
        std::assert!(!output.contains("database"));
    }

    #[tokio::test]
    async fn test_search_by_status() {
        // Test: Validates status filtering works.
        // Justification: Must filter by task status.
        let mut repo = MockTaskRepository { tasks: std::vec::Vec::new() };
        hexser::ports::Repository::save(&mut repo, create_test_task(
            "task-1",
            "Task A",
            task_manager::domain::task_status::TaskStatus::InProgress,
            std::option::Option::None,
        )).unwrap();
        hexser::ports::Repository::save(&mut repo, create_test_task(
            "task-2",
            "Task B",
            task_manager::domain::task_status::TaskStatus::Completed,
            std::option::Option::None,
        )).unwrap();

        let tool = SearchTasksTool::new(
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
            std::option::Option::None,
        );

        let result = tool.search(
            std::option::Option::None,
            std::option::Option::Some(std::string::String::from("InProgress")),
            std::option::Option::None,
            10,
        ).await;

        std::assert!(result.is_ok());
        let output = result.unwrap();
        std::assert!(output.contains("Task A"));
        std::assert!(!output.contains("Task B"));
    }

    #[tokio::test]
    async fn test_search_invalid_limit() {
        // Test: Validates limit bounds are enforced.
        // Justification: Must prevent excessive result sets.
        let repo = MockTaskRepository { tasks: std::vec::Vec::new() };
        let tool = SearchTasksTool::new(
            std::sync::Arc::new(std::sync::Mutex::new(repo)),
            std::option::Option::None,
        );

        let result = tool.search(std::option::Option::None, std::option::Option::None, std::option::Option::None, 0).await;
        std::assert!(result.is_err());

        let result = tool.search(std::option::Option::None, std::option::Option::None, std::option::Option::None, 100).await;
        std::assert!(result.is_err());
    }
}
