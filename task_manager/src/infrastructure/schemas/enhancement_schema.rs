//! JSON Schema generation for Enhancement domain entity.
//!
//! This infrastructure component generates JSON schemas used by LLM providers
//! (via Rig Extractor API) to enforce structured output format for task enhancements.
//!
//! Revision History
//! - 2025-11-23T21:12:00Z @AI: Refactor from utils/ to infrastructure/schemas/ (HEXSER compliance).
//! - 2025-11-23T14:35:00Z @AI: Create enhancement schema helper for Phase 1 Sprint 2 Rig integration.

/// Returns the JSON schema for Enhancement as a pretty-printed string.
///
/// This schema defines the expected structure for LLM-generated enhancements,
/// ensuring that the model produces properly typed output with all required fields.
///
/// # Returns
///
/// Returns `Ok(String)` with the JSON schema, or `Err(String)` if serialization fails.
pub fn enhancement_schema_json() -> std::result::Result<std::string::String, std::string::String> {
    let root: schemars::schema::RootSchema = schemars::schema_for!(crate::domain::enhancement::Enhancement);
    serde_json::to_string_pretty(&root)
        .map_err(|e| std::format!("Failed to serialize Enhancement schema: {}", e))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_enhancement_schema_json() {
        let schema_json = super::enhancement_schema_json().unwrap();
        std::assert!(schema_json.contains("Enhancement"), "Schema should reference Enhancement");
        std::assert!(schema_json.contains("enhancement_id"), "Schema should include enhancement_id field");
        std::assert!(schema_json.contains("task_id"), "Schema should include task_id field");
        std::assert!(schema_json.contains("timestamp"), "Schema should include timestamp field");
        std::assert!(schema_json.contains("enhancement_type"), "Schema should include enhancement_type field");
        std::assert!(schema_json.contains("content"), "Schema should include content field");

        let parsed: serde_json::Value = serde_json::from_str(&schema_json)
            .expect("Schema should be valid JSON");
        std::assert!(parsed.is_object(), "Schema root should be an object");
    }
}
