//! Defines the SortOrder enum for specifying sort direction.
//!
//! SortOrder represents the direction in which a collection should be sorted.
//! It is used in combination with TaskSortKey to fully specify sorting behavior
//! for task queries.
//!
//! Revision History
//! - 2025-11-06T17:41:00Z @AI: Initial SortOrder enum definition.

/// Specifies the direction for sorting operations.
///
/// SortOrder defines whether items should be sorted in ascending (smallest
/// to largest, A to Z, oldest to newest) or descending (largest to smallest,
/// Z to A, newest to oldest) order.
///
/// # Variants
///
/// * `Ascending` - Sort from smallest to largest, oldest to newest, or A to Z.
/// * `Descending` - Sort from largest to smallest, newest to oldest, or Z to A.
///
/// # Examples
///
/// ```
/// # use transcript_processor::domain::sort_order::SortOrder;
/// let ascending = SortOrder::Ascending;
/// let descending = SortOrder::Descending;
///
/// assert_ne!(ascending, descending);
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SortOrder {
    /// Sort in ascending order (smallest to largest, A to Z, oldest to newest).
    Ascending,

    /// Sort in descending order (largest to smallest, Z to A, newest to oldest).
    Descending,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_order_equality() {
        assert_eq!(SortOrder::Ascending, SortOrder::Ascending);
        assert_eq!(SortOrder::Descending, SortOrder::Descending);
    }

    #[test]
    fn test_sort_order_inequality() {
        assert_ne!(SortOrder::Ascending, SortOrder::Descending);
    }

    #[test]
    fn test_sort_order_clone() {
        let order = SortOrder::Ascending;
        let cloned = order.clone();
        assert_eq!(order, cloned);
    }

    #[test]
    fn test_both_variants_exist() {
        // Ensures both variants are usable
        let _asc = SortOrder::Ascending;
        let _desc = SortOrder::Descending;
    }
}
