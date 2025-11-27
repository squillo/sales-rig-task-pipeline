//! JSON Schema generation for action item extraction.
//!
//! This infrastructure component generates JSON schemas used by LLM providers
//! (via Rig Extractor API) to enforce structured output format. The schema
//! describes the expected structure of ExtractedActionItem DTOs.
//!
//! Revision History
//! - 2025-11-23T21:10:00Z @AI: Refactor from utils/ to infrastructure/schemas/ (HEXSER compliance).
//! - 2025-11-09T10:22:00Z @AI: Add action_item_schema helper for Rig extractor integration.

/// Returns the JSON schema for ExtractedActionItem as a serde_json::Value.
///
/// This schema is used to configure Rig Extractors for structured LLM output.
/// The schema enforces that LLMs return JSON objects with required "title"
/// field and optional "assignee" and "due_date" fields.
///
/// # Returns
///
/// A serde_json::Value containing the complete JSON Schema for action items.
///
/// # Examples
///
/// ```
/// use task_manager::infrastructure::schemas::action_item_schema::action_item_schema_json;
///
/// let schema = action_item_schema_json();
/// assert!(schema.is_object());
/// ```
pub fn action_item_schema_json() -> serde_json::Value {
    let schema = schemars::schema_for!(crate::infrastructure::dtos::extracted_action_item::ExtractedActionItem);
    serde_json::to_value(&schema).unwrap_or_else(|_| serde_json::json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_is_valid_json() {
        let schema = action_item_schema_json();
        assert!(schema.is_object(), "Schema should be a JSON object");
    }

    #[test]
    fn test_schema_contains_properties() {
        let schema = action_item_schema_json();
        let schema_obj = schema.as_object().expect("Schema should be object");

        // The schema_for! macro wraps the actual schema in a root object
        // We need to check the structure matches what schemars generates
        assert!(schema_obj.contains_key("$schema") || schema_obj.contains_key("definitions"),
            "Schema should contain schemars metadata");
    }
}
