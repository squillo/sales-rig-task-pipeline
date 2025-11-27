//! LLM response parser for comprehension tests.
//!
//! This infrastructure component parses possibly noisy JSON responses from LLM
//! providers into structured ComprehensionTest domain entities. It handles common
//! schema variations and provides fallback alias mapping when strict deserialization fails.
//!
//! Revision History
//! - 2025-11-23T21:40:00Z @AI: Refactor from utils/ to infrastructure/llm_parsers/ (HEXSER compliance).
//! - 2025-11-23 @AI: Introduce tolerant parser for ComprehensionTest (Phase 1 Sprint 3 Task 1.8).

/// Parses a possibly noisy LLM response into a ComprehensionTest.
///
/// This function is designed to handle LLM responses that may:
/// - Include extra text around the JSON
/// - Use field name aliases (e.g., "options" vs "answer_options")
/// - Use inconsistent casing for test_type
///
/// # Arguments
///
/// * `response_text` - Raw text from LLM that should contain a ComprehensionTest JSON object
/// * `task_id` - The task ID to associate with this test
///
/// # Returns
///
/// Returns a ComprehensionTest if parsing succeeds, or an error string describing the failure.
///
/// # Examples
///
/// ```
/// use task_orchestrator::infrastructure::llm_parsers::comprehension_test_parser::parse_comprehension_test_tolerant;
///
/// let response = r#"{"question": "What is Rust?", "answer": "A systems programming language"}"#;
/// let test = parse_comprehension_test_tolerant(response, "task-123").unwrap();
/// assert_eq!(test.task_id, "task-123");
/// ```
pub fn parse_comprehension_test_tolerant(
    response_text: &str,
    task_id: &str,
) -> std::result::Result<task_manager::domain::comprehension_test::ComprehensionTest, std::string::String> {
    // Try to find JSON object in response (model might include extra text)
    let json_start = response_text
        .find('{')
        .ok_or_else(|| std::string::String::from("No JSON object found in response"))?;
    let json_end = response_text
        .rfind('}')
        .ok_or_else(|| std::string::String::from("No JSON object found in response"))?;

    let json_str = &response_text[json_start..=json_end];

    // Parse as loose JSON Value first
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| std::format!("Failed to parse LLM response as JSON: {}", e))?;

    let obj = match value {
        serde_json::Value::Object(m) => m,
        _ => {
            return std::result::Result::Err(std::string::String::from(
                "Top-level JSON is not an object",
            ))
        }
    };

    // Helper to extract a string value from multiple candidate keys
    fn extract_string(
        m: &serde_json::Map<std::string::String, serde_json::Value>,
        keys: &[&str],
    ) -> std::option::Option<std::string::String> {
        for k in keys {
            if let std::option::Option::Some(v) = m.get(*k) {
                match v {
                    serde_json::Value::String(s) => {
                        if !s.trim().is_empty() {
                            return std::option::Option::Some(s.trim().to_string());
                        }
                    }
                    serde_json::Value::Number(n) => {
                        return std::option::Option::Some(n.to_string());
                    }
                    serde_json::Value::Bool(b) => {
                        return std::option::Option::Some(b.to_string());
                    }
                    _ => {}
                }
            }
        }
        std::option::Option::None
    }

    // Helper to extract array of strings
    fn extract_string_array(
        m: &serde_json::Map<std::string::String, serde_json::Value>,
        keys: &[&str],
    ) -> std::option::Option<std::vec::Vec<std::string::String>> {
        for k in keys {
            if let std::option::Option::Some(v) = m.get(*k) {
                match v {
                    serde_json::Value::Array(arr) => {
                        let strings: std::vec::Vec<std::string::String> = arr
                            .iter()
                            .filter_map(|item| match item {
                                serde_json::Value::String(s) => {
                                    std::option::Option::Some(s.clone())
                                }
                                _ => std::option::Option::None,
                            })
                            .collect();
                        if !strings.is_empty() {
                            return std::option::Option::Some(strings);
                        }
                    }
                    _ => {}
                }
            }
        }
        std::option::Option::None
    }

    // Extract question (required field)
    let question = extract_string(
        &obj,
        &[
            "question",
            "q",
            "prompt",
            "test_question",
            "quiz_question",
            "query",
        ],
    )
    .ok_or_else(|| std::string::String::from("Missing required field: question"))?;

    // Extract correct answer (required field)
    let correct_answer = extract_string(
        &obj,
        &[
            "correct_answer",
            "answer",
            "correct",
            "solution",
            "right_answer",
            "expected_answer",
        ],
    )
    .ok_or_else(|| std::string::String::from("Missing required field: correct_answer"))?;

    // Extract test_type with normalization to lowercase
    let test_type = extract_string(
        &obj,
        &[
            "test_type",
            "type",
            "test_kind",
            "question_type",
            "format",
        ],
    )
    .unwrap_or_else(|| std::string::String::from("short_answer"));

    // Normalize test_type to lowercase
    let test_type = test_type.to_lowercase();

    // Extract options (optional field)
    // Support common schema variants: "options", "answer_options", "choices", "alternatives"
    let options = extract_string_array(
        &obj,
        &[
            "options",
            "answer_options",
            "choices",
            "alternatives",
            "possible_answers",
        ],
    );

    // Generate unique test_id and capture timestamp
    let test_id = std::format!("test-{}", uuid::Uuid::new_v4());
    let timestamp = chrono::Utc::now();

    std::result::Result::Ok(task_manager::domain::comprehension_test::ComprehensionTest {
        test_id,
        task_id: task_id.to_string(),
        timestamp,
        test_type,
        question,
        options,
        correct_answer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_strict_json() {
        let json = r#"{
            "question": "What is Rust?",
            "correct_answer": "A systems programming language",
            "test_type": "short_answer"
        }"#;

        let result = parse_comprehension_test_tolerant(json, "task-123");
        assert!(result.is_ok());
        let test = result.unwrap();
        assert_eq!(test.question, "What is Rust?");
        assert_eq!(test.correct_answer, "A systems programming language");
        assert_eq!(test.test_type, "short_answer");
        assert_eq!(test.task_id, "task-123");
    }

    #[test]
    fn test_parse_with_alias_fields() {
        let json = r#"{
            "q": "What is 2+2?",
            "answer": "4",
            "type": "mcq",
            "choices": ["3", "4", "5"]
        }"#;

        let result = parse_comprehension_test_tolerant(json, "task-456");
        assert!(result.is_ok());
        let test = result.unwrap();
        assert_eq!(test.question, "What is 2+2?");
        assert_eq!(test.correct_answer, "4");
        assert_eq!(test.test_type, "mcq");
        assert!(test.options.is_some());
        assert_eq!(test.options.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_normalize_test_type_to_lowercase() {
        let json = r#"{
            "question": "Test question",
            "correct_answer": "Test answer",
            "test_type": "MULTIPLE_CHOICE"
        }"#;

        let result = parse_comprehension_test_tolerant(json, "task-789");
        assert!(result.is_ok());
        let test = result.unwrap();
        assert_eq!(test.test_type, "multiple_choice");
    }

    #[test]
    fn test_parse_with_noisy_text() {
        let response = r#"Here's the comprehension test:

        {
            "question": "What is hexagonal architecture?",
            "correct_answer": "A pattern that separates domain logic from infrastructure"
        }

        Hope this helps!"#;

        let result = parse_comprehension_test_tolerant(response, "task-abc");
        assert!(result.is_ok());
        let test = result.unwrap();
        assert_eq!(test.question, "What is hexagonal architecture?");
    }

    #[test]
    fn test_missing_required_field() {
        let json = r#"{
            "question": "What is Rust?"
        }"#;

        let result = parse_comprehension_test_tolerant(json, "task-xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("correct_answer"));
    }

    #[test]
    fn test_default_test_type() {
        let json = r#"{
            "question": "What is the capital of France?",
            "correct_answer": "Paris"
        }"#;

        let result = parse_comprehension_test_tolerant(json, "task-default");
        assert!(result.is_ok());
        let test = result.unwrap();
        assert_eq!(test.test_type, "short_answer"); // Default value
    }

    #[test]
    fn test_options_variant_answer_options() {
        let json = r#"{
            "question": "Pick the right answer",
            "correct_answer": "B",
            "answer_options": ["A", "B", "C", "D"]
        }"#;

        let result = parse_comprehension_test_tolerant(json, "task-opts");
        assert!(result.is_ok());
        let test = result.unwrap();
        assert!(test.options.is_some());
        assert_eq!(test.options.as_ref().unwrap().len(), 4);
    }
}
