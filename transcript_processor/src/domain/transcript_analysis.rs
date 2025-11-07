//! Defines the TranscriptAnalysis domain aggregate for analysis results.
//!
//! TranscriptAnalysis represents the complete structured output from analyzing
//! a meeting transcript. It aggregates all extracted action items and serves
//! as the primary output of the transcript processing pipeline.
//!
//! Revision History
//! - 2025-11-06T17:41:00Z @AI: Initial TranscriptAnalysis struct definition.

/// Represents the complete analysis result from processing a transcript.
///
/// A TranscriptAnalysis aggregates all action items extracted from a meeting
/// transcript by the LLM-powered extraction process. This struct serves as
/// the primary data structure passed between the extraction adapter and the
/// use case layer.
///
/// # Fields
///
/// * `action_items` - A vector of all action items identified in the transcript.
///
/// # Examples
///
/// ```
/// # use transcript_processor::domain::transcript_analysis::TranscriptAnalysis;
/// # use transcript_processor::domain::action_item::ActionItem;
/// let analysis = TranscriptAnalysis {
///     action_items: vec![
///         ActionItem {
///             title: std::string::String::from("Review PR"),
///             assignee: Some(std::string::String::from("Alice")),
///             due_date: None,
///         },
///     ],
/// };
///
/// assert_eq!(analysis.action_items.len(), 1);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct TranscriptAnalysis {
    /// All action items extracted from the transcript.
    pub action_items: Vec<crate::domain::action_item::ActionItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_analysis_empty() {
        // Test: Validates that a TranscriptAnalysis can be created with no action items.
        // Justification: Ensures the aggregate handles the edge case where LLM extraction finds
        // no actionable items in a transcript, which is valid for informational-only meetings.
        let analysis = TranscriptAnalysis {
            action_items: Vec::new(),
        };

        assert!(analysis.action_items.is_empty());
    }

    #[test]
    fn test_transcript_analysis_with_items() {
        // Test: Validates that a TranscriptAnalysis correctly aggregates multiple ActionItems.
        // Justification: Ensures the aggregate properly stores and provides access to the collection
        // of extracted action items, which is the primary output of the transcript processing pipeline.
        let action1 = crate::domain::action_item::ActionItem {
            title: std::string::String::from("First action"),
            assignee: None,
            due_date: None,
        };

        let action2 = crate::domain::action_item::ActionItem {
            title: std::string::String::from("Second action"),
            assignee: Some(std::string::String::from("Bob")),
            due_date: Some(std::string::String::from("2025-11-20")),
        };

        let analysis = TranscriptAnalysis {
            action_items: vec![action1, action2],
        };

        assert_eq!(analysis.action_items.len(), 2);
        assert_eq!(analysis.action_items[0].title, "First action");
        assert_eq!(analysis.action_items[1].title, "Second action");
    }
}
