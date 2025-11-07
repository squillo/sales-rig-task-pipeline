//! In-memory task repository adapter.
//!
//! This adapter implements the TaskRepositoryPort using an in-memory HashMap
//! for task storage following HEXSER framework patterns. It provides thread-safe
//! access to tasks through Arc and Mutex synchronization primitives.
//!
//! This implementation is suitable for prototyping and testing. For production
//! use, consider replacing with a persistent storage adapter (e.g., database).
//!
//! Revision History
//! - 2025-11-06T18:14:00Z @AI: Rewrite to implement HEXSER Repository and QueryRepository traits.
//! - 2025-11-06T18:00:00Z @AI: Initial InMemoryTaskAdapter implementation.

/// In-memory implementation of the TaskRepositoryPort.
///
/// This adapter stores Task entities in a HashMap protected by a Mutex
/// for thread-safe concurrent access. Tasks are indexed by their ID (String)
/// for efficient retrieval and updates.
///
/// # Fields
///
/// * `tasks` - A thread-safe HashMap storing all tasks indexed by ID.
///
/// # Examples
///
/// ```
/// # use task_manager::adapters::in_memory_task_adapter::InMemoryTaskAdapter;
/// let adapter = InMemoryTaskAdapter::new();
/// // Use adapter to store and retrieve tasks
/// ```
#[derive(hexser::HexAdapter)]
pub struct InMemoryTaskAdapter {
    tasks: std::sync::Arc<
        parking_lot::Mutex<std::collections::HashMap<String, crate::domain::task::Task>>,
    >,
}

impl InMemoryTaskAdapter {
    /// Creates a new InMemoryTaskAdapter with an empty task store.
    ///
    /// # Returns
    ///
    /// A new InMemoryTaskAdapter instance with no tasks.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::adapters::in_memory_task_adapter::InMemoryTaskAdapter;
    /// let adapter = InMemoryTaskAdapter::new();
    /// ```
    pub fn new() -> Self {
        Self {
            tasks: std::sync::Arc::new(parking_lot::Mutex::new(
                std::collections::HashMap::new(),
            )),
        }
    }
}

// Implement HEXSER's Repository trait for write operations
impl hexser::ports::Repository<crate::domain::task::Task> for InMemoryTaskAdapter {
    fn save(
        &mut self,
        entity: crate::domain::task::Task,
    ) -> hexser::HexResult<()> {
        let mut tasks = self.tasks.lock();
        tasks.insert(entity.id.clone(), entity);
        std::result::Result::Ok(())
    }
}

// Implement HEXSER's QueryRepository trait for read operations
impl hexser::ports::repository::QueryRepository<crate::domain::task::Task>
    for InMemoryTaskAdapter
{
    type Filter = crate::ports::task_repository_port::TaskFilter;
    type SortKey = crate::ports::task_repository_port::TaskSortKey;

    fn find_one(
        &self,
        filter: &Self::Filter,
    ) -> hexser::HexResult<std::option::Option<crate::domain::task::Task>> {
        let tasks = self.tasks.lock();

        let found = match filter {
            crate::ports::task_repository_port::TaskFilter::ById(id) => {
                tasks.get(id).cloned()
            }
            crate::ports::task_repository_port::TaskFilter::ByStatus(status) => {
                tasks.values().find(|task| &task.status == status).cloned()
            }
            crate::ports::task_repository_port::TaskFilter::ByAssignee(assignee) => {
                tasks
                    .values()
                    .find(|task| {
                        task.assignee
                            .as_ref()
                            .map(|a| a == assignee)
                            .unwrap_or(false)
                    })
                    .cloned()
            }
            crate::ports::task_repository_port::TaskFilter::All => {
                tasks.values().next().cloned()
            }
        };

        std::result::Result::Ok(found)
    }

    fn find(
        &self,
        filter: &Self::Filter,
        opts: hexser::ports::repository::FindOptions<Self::SortKey>,
    ) -> hexser::HexResult<std::vec::Vec<crate::domain::task::Task>> {
        let tasks = self.tasks.lock();

        // First, apply the filter
        let mut filtered: std::vec::Vec<crate::domain::task::Task> = match filter {
            crate::ports::task_repository_port::TaskFilter::ById(id) => {
                tasks.get(id).cloned().into_iter().collect()
            }
            crate::ports::task_repository_port::TaskFilter::ByStatus(status) => {
                tasks
                    .values()
                    .filter(|task| &task.status == status)
                    .cloned()
                    .collect()
            }
            crate::ports::task_repository_port::TaskFilter::ByAssignee(assignee) => {
                tasks
                    .values()
                    .filter(|task| {
                        task.assignee
                            .as_ref()
                            .map(|a| a == assignee)
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect()
            }
            crate::ports::task_repository_port::TaskFilter::All => {
                tasks.values().cloned().collect()
            }
        };

        // Apply sorting if specified
        if let std::option::Option::Some(sort_specs) = opts.sort {
            for sort_spec in sort_specs.iter().rev() {
                match &sort_spec.key {
                    crate::ports::task_repository_port::TaskSortKey::CreatedAt => {
                        filtered.sort_by_key(|task| task.created_at);
                    }
                    crate::ports::task_repository_port::TaskSortKey::UpdatedAt => {
                        filtered.sort_by_key(|task| task.updated_at);
                    }
                    crate::ports::task_repository_port::TaskSortKey::Status => {
                        filtered.sort_by(|a, b| {
                            std::format!("{:?}", a.status).cmp(&std::format!("{:?}", b.status))
                        });
                    }
                    crate::ports::task_repository_port::TaskSortKey::Title => {
                        filtered.sort_by(|a, b| a.title.cmp(&b.title));
                    }
                    crate::ports::task_repository_port::TaskSortKey::DueDate => {
                        filtered.sort_by(|a, b| {
                            match (&a.due_date, &b.due_date) {
                                (std::option::Option::Some(date_a), std::option::Option::Some(date_b)) => {
                                    date_a.cmp(date_b)
                                }
                                (std::option::Option::Some(_), std::option::Option::None) => {
                                    std::cmp::Ordering::Less
                                }
                                (std::option::Option::None, std::option::Option::Some(_)) => {
                                    std::cmp::Ordering::Greater
                                }
                                (std::option::Option::None, std::option::Option::None) => {
                                    std::cmp::Ordering::Equal
                                }
                            }
                        });
                    }
                }
                // Apply direction if descending
                if sort_spec.direction == hexser::ports::repository::Direction::Desc {
                    filtered.reverse();
                }
            }
        }

        // Apply offset
        if let std::option::Option::Some(offset) = opts.offset {
            filtered = filtered.into_iter().skip(offset as usize).collect();
        }

        // Apply limit
        if let std::option::Option::Some(limit) = opts.limit {
            filtered = filtered.into_iter().take(limit as usize).collect();
        }

        std::result::Result::Ok(filtered)
    }
}

// Implement the marker trait
impl crate::ports::task_repository_port::TaskRepositoryPort
    for InMemoryTaskAdapter
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexser::ports::Repository;
    use hexser::ports::repository::QueryRepository;

    fn create_test_task(id: &str, title: &str, status: crate::domain::task_status::TaskStatus) -> crate::domain::task::Task {
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from(title),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };

        let mut task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);
        task.id = std::string::String::from(id);
        task.status = status;
        task
    }

    #[test]
    fn test_adapter_creation() {
        // Test: Validates that the InMemoryTaskAdapter can be instantiated with an empty store.
        // Justification: Ensures the adapter initializes correctly with no tasks, which is the
        // starting state for all new sessions before any tasks are persisted.
        let adapter = InMemoryTaskAdapter::new();
        let result = adapter.find(
            &crate::ports::task_repository_port::TaskFilter::All,
            hexser::ports::repository::FindOptions::default()
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_save_and_find_one() {
        // Test: Validates that save() and find_one() work correctly for basic CRUD operations.
        // Justification: Ensures the core Repository trait methods function properly for persisting
        // and retrieving tasks by ID, which are fundamental operations for the entire system.
        let mut adapter = InMemoryTaskAdapter::new();
        let task = create_test_task("task-1", "Test Task", crate::domain::task_status::TaskStatus::Todo);

        let save_result = adapter.save(task);
        assert!(save_result.is_ok());

        let find_result = adapter.find_one(
            &crate::ports::task_repository_port::TaskFilter::ById(std::string::String::from("task-1"))
        );
        assert!(find_result.is_ok());

        let retrieved = find_result.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Task");
    }

    #[test]
    fn test_find_by_status_filter() {
        // Test: Validates that the QueryRepository find() method correctly filters tasks by status.
        // Justification: Ensures the filtering logic works correctly for status-based queries, which
        // is essential for displaying tasks organized by their lifecycle state (Todo, InProgress, etc).
        let mut adapter = InMemoryTaskAdapter::new();

        let task1 = create_test_task("task-1", "Todo Task", crate::domain::task_status::TaskStatus::Todo);
        let task2 = create_test_task("task-2", "InProgress Task", crate::domain::task_status::TaskStatus::InProgress);

        adapter.save(task1).unwrap();
        adapter.save(task2).unwrap();

        let result = adapter.find(
            &crate::ports::task_repository_port::TaskFilter::ByStatus(
                crate::domain::task_status::TaskStatus::Todo
            ),
            hexser::ports::repository::FindOptions::default()
        );

        assert!(result.is_ok());
        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "Todo Task");
    }

    #[test]
    fn test_find_with_title_sorting() {
        // Test: Validates that the QueryRepository find() method correctly sorts tasks by title.
        // Justification: Ensures the sorting logic works correctly with FindOptions, which is
        // critical for providing organized, user-friendly task lists with custom ordering.
        let mut adapter = InMemoryTaskAdapter::new();

        let task_c = create_test_task("task-1", "Charlie", crate::domain::task_status::TaskStatus::Todo);
        let task_a = create_test_task("task-2", "Alice", crate::domain::task_status::TaskStatus::Todo);
        let task_b = create_test_task("task-3", "Bob", crate::domain::task_status::TaskStatus::Todo);

        adapter.save(task_c).unwrap();
        adapter.save(task_a).unwrap();
        adapter.save(task_b).unwrap();

        let result = adapter.find(
            &crate::ports::task_repository_port::TaskFilter::All,
            hexser::ports::repository::FindOptions {
                sort: std::option::Option::Some(vec![
                    hexser::ports::repository::Sort {
                        key: crate::ports::task_repository_port::TaskSortKey::Title,
                        direction: hexser::ports::repository::Direction::Asc,
                    }
                ]),
                limit: std::option::Option::None,
                offset: std::option::Option::None,
            }
        );

        assert!(result.is_ok());
        let sorted = result.unwrap();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].title, "Alice");
        assert_eq!(sorted[1].title, "Bob");
        assert_eq!(sorted[2].title, "Charlie");
    }
}
