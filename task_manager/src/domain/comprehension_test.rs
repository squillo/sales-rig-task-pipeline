//! Defines the ComprehensionTest domain entity for validating task understanding.
//!
//! ComprehensionTest captures a check of understanding for a task, including
//! the question, optional multiple-choice options, and the correct answer.
//! Tests are attached to tasks and used by the orchestrator to decide routing.
//!
//! Revision History
//! - 2025-11-23T14:40:00Z @AI: Add schemars::JsonSchema derive for Rig Extractor integration (Phase 1 Sprint 2).
//! - 2025-11-12T20:28:00Z @AI: Add ComprehensionTest struct to support orchestration Phase 1.

/// Represents a comprehension test associated with a task.
///
/// These tests can be used to validate whether the task and its enhancements
/// are sufficiently clear. The `options` field is optional to support both
/// free-form and multiple-choice styles.
///
/// # Fields
///
/// * `test_id` - Unique identifier for this test.
/// * `task_id` - Identifier of the Task this test refers to.
/// * `timestamp` - UTC timestamp when the test was created.
/// * `test_type` - A classification of the test (e.g., "mcq", "short_answer").
/// * `question` - The test question prompt.
/// * `options` - Optional list of answer choices for multiple-choice tests.
/// * `correct_answer` - The correct answer or rubric.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, hexser::HexEntity)]
pub struct ComprehensionTest {
    /// Unique identifier for this test.
    pub test_id: String,

    /// Identifier of the Task being tested.
    pub task_id: String,

    /// Creation timestamp in UTC.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Test classification string.
    pub test_type: String,

    /// The prompt/question for the test.
    pub question: String,

    /// Optional set of options for multiple-choice tests.
    pub options: Option<std::vec::Vec<String>>,

    /// The correct answer or rubric.
    pub correct_answer: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_comprehension_test_basic_fields() {
        let t = super::ComprehensionTest {
            test_id: std::string::String::from("ct1"),
            task_id: std::string::String::from("t1"),
            timestamp: chrono::Utc::now(),
            test_type: std::string::String::from("mcq"),
            question: std::string::String::from("What is 2+2?"),
            options: Some(std::vec::Vec::from([
                std::string::String::from("3"),
                std::string::String::from("4"),
            ])),
            correct_answer: std::string::String::from("4"),
        };
        std::assert_eq!(t.test_id, std::string::String::from("ct1"));
        std::assert_eq!(t.task_id, std::string::String::from("t1"));
        std::assert_eq!(t.test_type, std::string::String::from("mcq"));
        std::assert_eq!(t.question, std::string::String::from("What is 2+2?"));
        std::assert!(t.options.is_some());
        std::assert_eq!(t.correct_answer, std::string::String::from("4"));
    }
}
