//! JSON Schema generation for ComprehensionTest domain entity.
//!
//! This infrastructure component generates JSON schemas used by LLM providers
//! (via Rig Extractor API) to enforce structured output format for comprehension tests.
//!
//! Revision History
//! - 2025-11-23T21:13:00Z @AI: Refactor from utils/ to infrastructure/schemas/ (HEXSER compliance).
//! - 2025-11-23T14:40:00Z @AI: Create comprehension_test schema helper for Phase 1 Sprint 2 Rig integration.

/// Returns the JSON schema for ComprehensionTest as a pretty-printed string.
///
/// This schema defines the expected structure for LLM-generated comprehension tests,
/// ensuring that the model produces properly typed output with all required fields.
///
/// # Returns
///
/// Returns `Ok(String)` with the JSON schema, or `Err(String)` if serialization fails.
pub fn comprehension_test_schema_json() -> std::result::Result<std::string::String, std::string::String> {
    let root: schemars::schema::RootSchema = schemars::schema_for!(crate::domain::comprehension_test::ComprehensionTest);
    serde_json::to_string_pretty(&root)
        .map_err(|e| std::format!("Failed to serialize ComprehensionTest schema: {}", e))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_comprehension_test_schema_json() {
        let schema_json = super::comprehension_test_schema_json().unwrap();
        std::assert!(schema_json.contains("ComprehensionTest"), "Schema should reference ComprehensionTest");
        std::assert!(schema_json.contains("test_id"), "Schema should include test_id field");
        std::assert!(schema_json.contains("task_id"), "Schema should include task_id field");
        std::assert!(schema_json.contains("timestamp"), "Schema should include timestamp field");
        std::assert!(schema_json.contains("test_type"), "Schema should include test_type field");
        std::assert!(schema_json.contains("question"), "Schema should include question field");
        std::assert!(schema_json.contains("options"), "Schema should include options field");
        std::assert!(schema_json.contains("correct_answer"), "Schema should include correct_answer field");

        let parsed: serde_json::Value = serde_json::from_str(&schema_json)
            .expect("Schema should be valid JSON");
        std::assert!(parsed.is_object(), "Schema root should be an object");
    }
}
