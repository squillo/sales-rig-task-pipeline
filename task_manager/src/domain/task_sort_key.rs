//! Defines the TaskSortKey enum for specifying task sorting criteria.
//!
//! TaskSortKey represents the field by which a collection of tasks should be
//! sorted. This enables flexible querying and display of tasks based on
//! different attributes like creation time, status, or title.
//!
//! Revision History
//! - 2025-11-06T17:41:00Z @AI: Initial TaskSortKey enum definition.

/// Specifies which field to use when sorting a collection of tasks.
///
/// TaskSortKey defines the available sorting criteria for task queries.
/// It is used in combination with SortOrder to specify both the field
/// and direction for sorting operations.
///
/// # Variants
///
/// * `CreatedAt` - Sort by the task creation timestamp.
/// * `UpdatedAt` - Sort by the last modification timestamp.
/// * `Status` - Sort by the task status (lifecycle state).
/// * `Title` - Sort alphabetically by the task title.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::task_sort_key::TaskSortKey;
/// let sort_by_date = TaskSortKey::CreatedAt;
/// let sort_by_title = TaskSortKey::Title;
///
/// assert_ne!(sort_by_date, sort_by_title);
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TaskSortKey {
    /// Sort by the task creation timestamp (created_at field).
    CreatedAt,

    /// Sort by the last modification timestamp (updated_at field).
    UpdatedAt,

    /// Sort by the task lifecycle status.
    Status,

    /// Sort alphabetically by the task title.
    Title,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_sort_key_equality() {
        assert_eq!(TaskSortKey::CreatedAt, TaskSortKey::CreatedAt);
        assert_eq!(TaskSortKey::UpdatedAt, TaskSortKey::UpdatedAt);
        assert_eq!(TaskSortKey::Status, TaskSortKey::Status);
        assert_eq!(TaskSortKey::Title, TaskSortKey::Title);
    }

    #[test]
    fn test_task_sort_key_inequality() {
        assert_ne!(TaskSortKey::CreatedAt, TaskSortKey::UpdatedAt);
        assert_ne!(TaskSortKey::Status, TaskSortKey::Title);
    }

    #[test]
    fn test_task_sort_key_clone() {
        let key = TaskSortKey::CreatedAt;
        let cloned = key.clone();
        assert_eq!(key, cloned);
    }

    #[test]
    fn test_all_variants_exist() {
        // Ensures all four variants are usable
        let _created = TaskSortKey::CreatedAt;
        let _updated = TaskSortKey::UpdatedAt;
        let _status = TaskSortKey::Status;
        let _title = TaskSortKey::Title;
    }
}
