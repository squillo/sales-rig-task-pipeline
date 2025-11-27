//! Data Transfer Object for action items extracted from external systems.
//!
//! ExtractedActionItem is a DTO used at the boundary between infrastructure
//! (LLM providers, APIs) and domain (ActionItem entity). It represents the raw,
//! unvalidated data structure returned by external systems before domain validation.
//!
//! Revision History
//! - 2025-11-23T21:05:00Z @AI: Refactor from utils/ to infrastructure/dtos/ (HEXSER compliance).
//! - 2025-11-08T08:36:00Z @AI: Create ExtractedActionItem DTO for LLM extraction boundary.

/// Data Transfer Object representing an action item extracted from an external system.
///
/// This struct is used to deserialize JSON responses from LLM providers before
/// mapping to the domain ActionItem entity. It contains only the fields that
/// external systems are expected to provide, with all fields as strings for
/// maximum parsing flexibility.
///
/// # Examples
///
/// ```
/// use task_manager::infrastructure::dtos::extracted_action_item::ExtractedActionItem;
///
/// let item = ExtractedActionItem {
///     title: "Write documentation".to_string(),
///     assignee: Some("Alice".to_string()),
///     due_date: Some("2025-12-01".to_string()),
/// };
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct ExtractedActionItem {
    /// The title or description of the action item
    pub title: String,

    /// Optional assignee name
    #[serde(default)]
    pub assignee: std::option::Option<String>,

    /// Optional due date (as string, domain will parse/validate)
    #[serde(default)]
    pub due_date: std::option::Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_full_item() {
        let json = r#"{"title": "Test task", "assignee": "Bob", "due_date": "2025-12-01"}"#;
        let item: ExtractedActionItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.title, "Test task");
        assert_eq!(item.assignee, Some("Bob".to_string()));
        assert_eq!(item.due_date, Some("2025-12-01".to_string()));
    }

    #[test]
    fn test_deserialize_minimal_item() {
        let json = r#"{"title": "Minimal task"}"#;
        let item: ExtractedActionItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.title, "Minimal task");
        assert_eq!(item.assignee, None);
        assert_eq!(item.due_date, None);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let item = ExtractedActionItem {
            title: "Roundtrip test".to_string(),
            assignee: Some("Charlie".to_string()),
            due_date: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: ExtractedActionItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, item.title);
        assert_eq!(deserialized.assignee, item.assignee);
    }
}
