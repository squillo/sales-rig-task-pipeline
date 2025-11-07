//! Defines the ChecklistItem domain value object for task sub-items.
//!
//! ChecklistItem represents a single step or sub-task within an action item.
//! It allows for granular tracking of progress on complex tasks by breaking
//! them down into smaller, completable units.
//!
//! Revision History
//! - 2025-11-06T18:14:00Z @AI: Add HexEntity derive for HEXSER framework alignment.
//! - 2025-11-06T17:41:00Z @AI: Initial ChecklistItem struct definition.

/// Represents a single item in a checklist associated with an action item.
///
/// A ChecklistItem is a granular unit of work that can be marked as completed
/// or incomplete. Multiple ChecklistItems can be associated with a single
/// ActionItem to track progress on complex tasks.
///
/// # Fields
///
/// * `description` - The text describing this checklist item (required).
/// * `completed` - Whether this item has been completed (required, defaults to false).
///
/// # Examples
///
/// ```
/// # use transcript_processor::domain::checklist_item::ChecklistItem;
/// let item = ChecklistItem {
///     description: std::string::String::from("Review section 1"),
///     completed: false,
/// };
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, hexser::HexEntity)]
pub struct ChecklistItem {
    /// The description of this checklist item.
    pub description: String,

    /// Whether this checklist item has been completed.
    pub completed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checklist_item_creation() {
        // Test: Validates that a ChecklistItem can be created with completed set to false.
        // Justification: Ensures the struct correctly initializes for the typical case of
        // a new, uncompleted checklist item tracking granular sub-task progress.
        let item = ChecklistItem {
            description: std::string::String::from("Test item"),
            completed: false,
        };

        assert_eq!(item.description, "Test item");
        assert!(!item.completed);
    }

    #[test]
    fn test_checklist_item_completed() {
        // Test: Validates that a ChecklistItem can be created with completed set to true.
        // Justification: Ensures the struct correctly represents the completed state, which is
        // essential for tracking progress on complex tasks with multiple sub-items.
        let item = ChecklistItem {
            description: std::string::String::from("Completed item"),
            completed: true,
        };

        assert_eq!(item.description, "Completed item");
        assert!(item.completed);
    }
}
