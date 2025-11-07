//! Defines the ActionItem domain entity for extracted action items.
//!
//! ActionItem represents a single action item extracted from a meeting transcript
//! by the LLM. It serves as a data transfer object (DTO) that will be converted
//! into a Task entity for persistence and tracking.
//!
//! Revision History
//! - 2025-11-06T18:14:00Z @AI: Add HexEntity derive for HEXSER framework alignment.
//! - 2025-11-06T17:41:00Z @AI: Initial ActionItem struct definition.

/// Represents an action item extracted from a meeting transcript.
///
/// An ActionItem is a lightweight DTO that captures the essential information
/// about a task identified during transcript analysis. It is designed to be
/// deserialized from LLM output and then transformed into a Task entity.
///
/// # Fields
///
/// * `title` - The description of the action item (required).
/// * `assignee` - The person responsible for the action item (optional).
/// * `due_date` - The deadline for completing the action item as a string (optional).
///
/// # Examples
///
/// ```
/// # use transcript_processor::domain::action_item::ActionItem;
/// let action = ActionItem {
///     title: std::string::String::from("Review design document"),
///     assignee: Some(std::string::String::from("Alice")),
///     due_date: Some(std::string::String::from("2025-11-15")),
/// };
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, hexser::HexEntity)]
pub struct ActionItem {
    /// The description or title of the action item.
    pub title: String,

    /// The person assigned to complete this action item.
    pub assignee: Option<String>,

    /// The due date for this action item in string format.
    pub due_date: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_item_creation() {
        // Test: Validates that an ActionItem can be created with all fields populated.
        // Justification: Ensures the struct correctly stores all fields for the complete case,
        // which is the expected output format from LLM extraction with full data.
        let action = ActionItem {
            title: std::string::String::from("Test action"),
            assignee: Some(std::string::String::from("Bob")),
            due_date: Some(std::string::String::from("2025-12-01")),
        };

        assert_eq!(action.title, "Test action");
        assert_eq!(action.assignee, Some(std::string::String::from("Bob")));
        assert_eq!(action.due_date, Some(std::string::String::from("2025-12-01")));
    }

    #[test]
    fn test_action_item_optional_fields() {
        // Test: Validates that an ActionItem can be created with only the required title field.
        // Justification: Ensures the struct handles missing optional fields correctly, which is a
        // common scenario when LLM extraction doesn't identify assignees or due dates in transcripts.
        let action = ActionItem {
            title: std::string::String::from("Unassigned task"),
            assignee: None,
            due_date: None,
        };

        assert_eq!(action.title, "Unassigned task");
        assert!(action.assignee.is_none());
        assert!(action.due_date.is_none());
    }
}
