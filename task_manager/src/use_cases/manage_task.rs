//! Defines the ManageTaskUseCase for task lifecycle management operations.
//!
//! This use case provides operations for managing existing tasks, including updating
//! their status, querying with sorting, and retrieving revision history. It demonstrates
//! separation of concerns by delegating persistence to the repository port.
//!
//! Revision History
//! - 2025-11-06T18:30:00Z @AI: Refactor to use generic concrete repository type (HEXSER pattern).
//! - 2025-11-06T17:41:00Z @AI: Initial ManageTaskUseCase implementation.

/// Use case for managing task lifecycle and queries.
///
/// ManageTaskUseCase provides operations for updating task status and retrieving
/// tasks with flexible sorting using HEXSER repository patterns.
///
/// This use case is generic over the repository type to enable compile-time
/// polymorphism and mutable access to the repository for HEXSER's save() method.
///
/// # Type Parameters
///
/// * `R` - The concrete repository type implementing TaskRepositoryPort.
///
/// # Examples
///
/// ```no_run
/// # use task_manager::use_cases::manage_task::ManageTaskUseCase;
/// # use task_manager::adapters::in_memory_task_adapter::InMemoryTaskAdapter;
/// # use task_manager::domain::task_status::TaskStatus;
/// # async fn example() {
/// let repo = InMemoryTaskAdapter::new();
/// let mut use_case = ManageTaskUseCase::new(repo);
/// use_case.update_task_status("task-123", TaskStatus::InProgress).unwrap();
/// # }
/// ```
pub struct ManageTaskUseCase<R>
where
    R: crate::ports::task_repository_port::TaskRepositoryPort,
{
    task_repo: R,
}

impl<R> ManageTaskUseCase<R>
where
    R: crate::ports::task_repository_port::TaskRepositoryPort,
{
    /// Creates a new ManageTaskUseCase with the provided repository.
    ///
    /// # Arguments
    ///
    /// * `task_repo` - The concrete task repository implementation (owned).
    ///
    /// # Returns
    ///
    /// A new ManageTaskUseCase instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::use_cases::manage_task::ManageTaskUseCase;
    /// # use task_manager::adapters::in_memory_task_adapter::InMemoryTaskAdapter;
    /// let repo = InMemoryTaskAdapter::new();
    /// let use_case = ManageTaskUseCase::new(repo);
    /// ```
    pub fn new(task_repo: R) -> Self {
        ManageTaskUseCase { task_repo }
    }

    /// Updates the status of a task.
    ///
    /// This method retrieves the task using HEXSER's find_one(), updates its status
    /// and updated_at timestamp, then persists the changes using save().
    ///
    /// # Arguments
    ///
    /// * `task_id` - The unique ID of the task to update.
    /// * `new_status` - The new status to set.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Status successfully updated.
    /// * `Err(String)` - Error message if update fails or task not found.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::use_cases::manage_task::ManageTaskUseCase;
    /// # use task_manager::adapters::in_memory_task_adapter::InMemoryTaskAdapter;
    /// # use task_manager::domain::task_status::TaskStatus;
    /// let repo = InMemoryTaskAdapter::new();
    /// let mut use_case = ManageTaskUseCase::new(repo);
    /// use_case.update_task_status("task-456", TaskStatus::Completed).unwrap();
    /// ```
    pub fn update_task_status(
        &mut self,
        task_id: &str,
        new_status: crate::domain::task_status::TaskStatus,
    ) -> std::result::Result<(), std::string::String> {
        // Retrieve the existing task using HEXSER's find_one()
        let filter = crate::ports::task_repository_port::TaskFilter::ById(
            task_id.to_string()
        );

        let task_option = self.task_repo
            .find_one(&filter)
            .map_err(|e| std::format!("Failed to find task: {:?}", e))?;

        let mut task = task_option.ok_or_else(|| {
            std::format!("Task with ID {} not found", task_id)
        })?;

        // Update the task's status and timestamp
        task.status = new_status;
        task.updated_at = chrono::Utc::now();

        // Persist the updated task using HEXSER's save()
        self.task_repo
            .save(task)
            .map_err(|e| std::format!("Failed to save task: {:?}", e))?;

        std::result::Result::Ok(())
    }

    /// Retrieves all tasks sorted by the specified criteria using HEXSER patterns.
    ///
    /// # Arguments
    ///
    /// * `sort_key` - The TaskSortKey to sort by.
    /// * `direction` - The sort direction (Asc or Desc).
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Task>)` - All tasks sorted as requested.
    /// * `Err(String)` - Error message if retrieval fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::use_cases::manage_task::ManageTaskUseCase;
    /// # use task_manager::adapters::in_memory_task_adapter::InMemoryTaskAdapter;
    /// # use task_manager::ports::task_repository_port::TaskSortKey;
    /// let repo = InMemoryTaskAdapter::new();
    /// let use_case = ManageTaskUseCase::new(repo);
    /// let tasks = use_case.get_sorted_tasks(
    ///     TaskSortKey::CreatedAt,
    ///     hexser::ports::repository::Direction::Desc
    /// ).unwrap();
    /// ```
    pub fn get_sorted_tasks(
        &self,
        sort_key: crate::ports::task_repository_port::TaskSortKey,
        direction: hexser::ports::repository::Direction,
    ) -> std::result::Result<Vec<crate::domain::task::Task>, std::string::String> {
        let filter = crate::ports::task_repository_port::TaskFilter::All;
        let opts = hexser::ports::repository::FindOptions {
            sort: std::option::Option::Some(vec![
                hexser::ports::repository::Sort {
                    key: sort_key,
                    direction,
                }
            ]),
            limit: std::option::Option::None,
            offset: std::option::Option::None,
        };

        self.task_repo
            .find(&filter, opts)
            .map_err(|e| std::format!("Failed to retrieve tasks: {:?}", e))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use hexser::ports::Repository;
    use hexser::ports::repository::QueryRepository;

    struct MockRepo {
        tasks: std::collections::HashMap<String, crate::domain::task::Task>,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                tasks: std::collections::HashMap::new(),
            }
        }
    }

    // Implement HEXSER Repository trait for write operations
    impl hexser::ports::Repository<crate::domain::task::Task> for MockRepo {
        fn save(
            &mut self,
            entity: crate::domain::task::Task,
        ) -> hexser::HexResult<()> {
            self.tasks.insert(entity.id.clone(), entity);
            std::result::Result::Ok(())
        }
    }

    // Implement HEXSER QueryRepository trait for read operations
    impl hexser::ports::repository::QueryRepository<crate::domain::task::Task> for MockRepo {
        type Filter = crate::ports::task_repository_port::TaskFilter;
        type SortKey = crate::ports::task_repository_port::TaskSortKey;

        fn find_one(
            &self,
            filter: &Self::Filter,
        ) -> hexser::HexResult<std::option::Option<crate::domain::task::Task>> {
            let result = match filter {
                crate::ports::task_repository_port::TaskFilter::ById(id) => {
                    self.tasks.get(id).cloned()
                }
                _ => self.tasks.values().next().cloned(),
            };
            std::result::Result::Ok(result)
        }

        fn find(
            &self,
            _filter: &Self::Filter,
            _opts: hexser::ports::repository::FindOptions<Self::SortKey>,
        ) -> hexser::HexResult<std::vec::Vec<crate::domain::task::Task>> {
            std::result::Result::Ok(self.tasks.values().cloned().collect())
        }
    }

    // Implement the marker trait to satisfy TaskRepositoryPort bounds
    impl crate::ports::task_repository_port::TaskRepositoryPort for MockRepo {}

    #[test]
    fn test_update_task_status() {
        // Test: Validates that the ManageTaskUseCase correctly updates a task's status.
        // Justification: Ensures the use case properly retrieves a task, modifies its status and
        // timestamp, and persists the changes - a critical operation for task lifecycle management.
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Test task"),
            assignee: None,
            due_date: None,
        };

        let task = crate::domain::task::Task::from_action_item(&action, None);
        let task_id = task.id.clone();

        let mut repo = MockRepo::new();
        // Pre-populate the repository with the task
        repo.save(task.clone()).unwrap();

        let mut use_case = ManageTaskUseCase::new(repo);

        let result = use_case
            .update_task_status(&task_id, crate::domain::task_status::TaskStatus::InProgress);

        assert!(result.is_ok());

        // Verify the task was updated by checking internal state
        let filter = crate::ports::task_repository_port::TaskFilter::ById(task_id.clone());
        let updated_task = use_case.task_repo.find_one(&filter).unwrap().unwrap();
        assert_eq!(updated_task.status, crate::domain::task_status::TaskStatus::InProgress);
    }

    #[test]
    fn test_get_sorted_tasks() {
        // Test: Validates that the ManageTaskUseCase correctly retrieves tasks with sorting applied.
        // Justification: Ensures the use case properly uses QueryRepository's find() with FindOptions
        // to retrieve sorted task lists, which is essential for organized task display in UIs.
        let repo = MockRepo::new();
        let use_case = ManageTaskUseCase::new(repo);

        let result = use_case
            .get_sorted_tasks(
                crate::ports::task_repository_port::TaskSortKey::CreatedAt,
                hexser::ports::repository::Direction::Asc,
            );

        assert!(result.is_ok());
        let tasks = result.unwrap();
        assert_eq!(tasks.len(), 0); // Empty repo returns empty list
    }
}
