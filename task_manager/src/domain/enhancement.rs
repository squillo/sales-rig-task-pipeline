//! Defines the Enhancement domain entity representing model-generated task improvements.
//!
//! Enhancement captures structured, traceable improvements proposed for a Task.
//! Each enhancement is tied to a specific task, typed for routing, and timestamped
//! for auditability. This entity is pure domain and framework-agnostic.
//!
//! Revision History
//! - 2025-11-23T14:35:00Z @AI: Add schemars::JsonSchema derive for Rig Extractor integration (Phase 1 Sprint 2).
//! - 2025-11-12T20:28:00Z @AI: Add Enhancement struct to support orchestration Phase 1.

/// Represents a model-generated enhancement for a task.
///
/// Enhancements are suggestions or augmentations to improve the task's content,
/// clarity, or completeness. They are created by adapters and recorded in the
/// domain for downstream orchestration steps.
///
/// # Fields
///
/// * `enhancement_id` - Unique identifier for this enhancement.
/// * `task_id` - Identifier of the Task this enhancement is associated with.
/// * `timestamp` - UTC timestamp when the enhancement was created.
/// * `enhancement_type` - A short classification string (e.g., "rewrite", "summarize").
/// * `content` - The enhancement payload as a string.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, hexser::HexEntity)]
pub struct Enhancement {
    /// Unique identifier for this enhancement.
    pub enhancement_id: String,

    /// Identifier of the Task this enhancement belongs to.
    pub task_id: String,

    /// Creation timestamp in UTC.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Enhancement classification string.
    pub enhancement_type: String,

    /// The enhancement content payload.
    pub content: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_enhancement_basic_fields() {
        let e = super::Enhancement {
            enhancement_id: std::string::String::from("e1"),
            task_id: std::string::String::from("t1"),
            timestamp: chrono::Utc::now(),
            enhancement_type: std::string::String::from("rewrite"),
            content: std::string::String::from("Improve clarity."),
        };
        std::assert_eq!(e.enhancement_id, std::string::String::from("e1"));
        std::assert_eq!(e.task_id, std::string::String::from("t1"));
        std::assert_eq!(e.enhancement_type, std::string::String::from("rewrite"));
        std::assert_eq!(e.content, std::string::String::from("Improve clarity."));
    }
}
