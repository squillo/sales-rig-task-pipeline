//! Defines the TranscriptExtractorPort output port for LLM-powered extraction.
//!
//! This port represents the interface for extracting structured action items from
//! unstructured meeting transcripts using an LLM. Concrete implementations (adapters)
//! will provide the actual LLM integration (e.g., via Ollama with rig-core).
//!
//! Revision History
//! - 2025-11-06T18:14:00Z @AI: Add HexPort derive for HEXSER framework alignment.
//! - 2025-11-06T17:41:00Z @AI: Initial TranscriptExtractorPort trait definition.

/// Port (interface) for extracting structured data from transcripts.
///
/// TranscriptExtractorPort defines the contract that any transcript extraction
/// adapter must implement. This enables the application layer to remain agnostic
/// of the specific LLM provider or extraction technology being used.
///
/// # Examples
///
/// ```
/// # use transcript_processor::application::ports::transcript_extractor_port::TranscriptExtractorPort;
/// # use transcript_processor::domain::transcript_analysis::TranscriptAnalysis;
/// # struct MockExtractor;
/// # #[async_trait::async_trait]
/// # impl TranscriptExtractorPort for MockExtractor {
/// #     async fn extract_analysis(&self, transcript: &str) -> std::result::Result<TranscriptAnalysis, std::string::String> {
/// #         std::result::Result::Ok(TranscriptAnalysis { action_items: Vec::new() })
/// #     }
/// # }
/// ```
#[async_trait::async_trait]
pub trait TranscriptExtractorPort: Send + Sync {
    /// Extracts structured action items from an unstructured transcript.
    ///
    /// This method processes raw transcript text and uses an LLM to identify
    /// and extract action items, returning a structured TranscriptAnalysis
    /// containing all discovered items.
    ///
    /// # Arguments
    ///
    /// * `transcript` - The raw transcript text to analyze.
    ///
    /// # Returns
    ///
    /// * `Ok(TranscriptAnalysis)` - Successfully extracted analysis.
    /// * `Err(String)` - Error message if extraction fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use transcript_processor::application::ports::transcript_extractor_port::TranscriptExtractorPort;
    /// # async fn example(extractor: &dyn TranscriptExtractorPort) {
    /// let transcript = "Meeting notes: Alice will review the PR by Friday.";
    /// let analysis = extractor.extract_analysis(transcript).await.unwrap();
    /// assert!(!analysis.action_items.is_empty());
    /// # }
    /// ```
    async fn extract_analysis(
        &self,
        transcript: &str,
    ) -> std::result::Result<crate::domain::transcript_analysis::TranscriptAnalysis, std::string::String>;
}
