//! Ollama-based transcript extraction adapter.
//!
//! This adapter implements the TranscriptExtractorPort using the Ollama
//! local LLM service. It sends transcript text to a specified language model
//! and parses the structured JSON response into ActionItem domain entities.
//!
//! The adapter constructs a carefully crafted prompt that instructs the LLM
//! to extract action items in a specific JSON format matching the ActionItem schema.
//!
//! Revision History
//! - 2025-11-06T20:45:00Z @AI: Fix assignee extraction by correcting JSON field name mismatch (assigned_to -> assignee).
//! - 2025-11-06T18:56:00Z @AI: Rename OllamaExtractorAdapter to OllamaTranscriptExtractorAdapter for clarity.
//! - 2025-11-06T18:14:00Z @AI: Add HexAdapter derive, fix method name to match port trait.
//! - 2025-11-06T18:00:00Z @AI: Initial OllamaExtractorAdapter implementation.

/// Adapter for extracting action items from transcripts using Ollama LLM service.
///
/// This struct implements the TranscriptExtractorPort by communicating with
/// a local Ollama instance. It uses a configurable language model to perform
/// the extraction and returns structured ActionItem entities.
///
/// # Fields
///
/// * `model_name` - The name of the Ollama model to use (e.g., "phi3:mini").
/// * `ollama_client` - The Ollama client instance for API communication.
///
/// # Examples
///
/// ```
/// # use transcript_processor::adapters::ollama_adapter::OllamaTranscriptExtractorAdapter;
/// let adapter = OllamaTranscriptExtractorAdapter::new(std::string::String::from("phi3:mini"));
/// // Use adapter to extract tasks from transcript text
/// ```
#[derive(hexser::HexAdapter)]
pub struct OllamaTranscriptExtractorAdapter {
    model_name: String,
    ollama_client: ollama_rs::Ollama,
}

impl OllamaTranscriptExtractorAdapter {
    /// Creates a new OllamaTranscriptExtractorAdapter with the specified model name.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the Ollama model to use for extraction.
    ///
    /// # Returns
    ///
    /// A new OllamaTranscriptExtractorAdapter instance configured to use the specified model.
    ///
    /// # Examples
    ///
    /// ```
    /// # use transcript_processor::adapters::ollama_adapter::OllamaTranscriptExtractorAdapter;
    /// let adapter = OllamaTranscriptExtractorAdapter::new(std::string::String::from("phi3:mini"));
    /// ```
    pub fn new(model_name: String) -> Self {
        let ollama_client = ollama_rs::Ollama::default();
        Self {
            model_name,
            ollama_client,
        }
    }

    /// Constructs the system prompt for the LLM extraction task.
    ///
    /// This prompt instructs the model to extract action items from a transcript
    /// and format them as a JSON array matching the ActionItem schema.
    ///
    /// The prompt emphasizes assignee extraction by providing examples of how
    /// people are assigned tasks in conversations (e.g., "I'll take", "James will").
    fn build_extraction_prompt(&self, transcript: &str) -> String {
        std::format!(
            r#"Extract all action items from the following meeting transcript.
Return ONLY a valid JSON array of objects, where each object has this exact structure:
{{
  "title": "Brief task title",
  "assignee": "Name of person assigned (or null if not specified)",
  "due_date": "YYYY-MM-DD format (or null if not specified)"
}}

IMPORTANT: Pay close attention to who is assigned each task. Look for patterns like:
- "I'll take ownership of..." -> extract the speaker's name
- "James will complete..." -> assignee is "James"
- "Maria can implement..." -> assignee is "Maria"
- "Let's have David..." -> assignee is "David"
- "Emily should..." -> assignee is "Emily"

Extract the person's first name only. If no assignee is clearly identified, use null.

Transcript:
{}

Respond with ONLY the JSON array, no other text."#,
            transcript
        )
    }

    /// Parses the LLM response string into a vector of ActionItem entities.
    ///
    /// Attempts to deserialize the JSON response into ActionItem structs.
    /// Returns an error if the response is not valid JSON or doesn't match
    /// the expected schema.
    fn parse_response(
        &self,
        response_text: &str,
    ) -> std::result::Result<std::vec::Vec<crate::domain::action_item::ActionItem>, String> {
        serde_json::from_str(response_text)
            .map_err(|e| std::format!("Failed to parse LLM response as JSON: {}", e))
    }
}

#[async_trait::async_trait]
impl crate::application::ports::transcript_extractor_port::TranscriptExtractorPort
    for OllamaTranscriptExtractorAdapter
{
    async fn extract_analysis(
        &self,
        transcript: &str,
    ) -> std::result::Result<crate::domain::transcript_analysis::TranscriptAnalysis, String> {
        // Build the extraction prompt
        let prompt = self.build_extraction_prompt(transcript);

        // Create generation request
        let request = ollama_rs::generation::completion::request::GenerationRequest::new(
            self.model_name.clone(),
            prompt,
        );

        // Call Ollama API
        let response = self
            .ollama_client
            .generate(request)
            .await
            .map_err(|e| std::format!("Ollama API error: {}", e))?;

        // Parse the response text into ActionItems and wrap in TranscriptAnalysis
        let action_items = self.parse_response(&response.response)?;
        std::result::Result::Ok(crate::domain::transcript_analysis::TranscriptAnalysis {
            action_items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        // Test: Validates that the OllamaTranscriptExtractorAdapter can be instantiated with a model name.
        // Justification: Ensures the adapter's constructor correctly initializes and stores the
        // model name, which is essential for making LLM API calls with the correct model.
        let adapter = OllamaTranscriptExtractorAdapter::new(std::string::String::from("phi3:mini"));
        assert_eq!(adapter.model_name, "phi3:mini");
    }

    #[test]
    fn test_build_extraction_prompt() {
        // Test: Validates that the prompt builder creates a properly formatted LLM instruction.
        // Justification: Ensures the prompt contains all necessary instructions and the transcript text,
        // which is critical for reliable LLM extraction of structured action items.
        let adapter = OllamaTranscriptExtractorAdapter::new(std::string::String::from("test-model"));
        let transcript = "John will review the document by Friday.";
        let prompt = adapter.build_extraction_prompt(transcript);

        assert!(prompt.contains("Extract all action items"));
        assert!(prompt.contains(transcript));
        assert!(prompt.contains("JSON array"));
    }

    #[test]
    fn test_parse_response_valid_json() {
        // Test: Validates that the parser correctly deserializes valid JSON into ActionItem structs.
        // Justification: Ensures the adapter can handle well-formed LLM responses and extract all
        // action item fields, which is the primary success path for the extraction pipeline.
        let adapter = OllamaTranscriptExtractorAdapter::new(std::string::String::from("test-model"));
        let json_response = r#"[
            {
                "title": "Review document",
                "assignee": "John",
                "due_date": "2025-11-10"
            }
        ]"#;

        let result = adapter.parse_response(json_response);
        assert!(result.is_ok());

        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Review document");
        assert_eq!(items[0].assignee, std::option::Option::Some(std::string::String::from("John")));
        assert_eq!(items[0].due_date, std::option::Option::Some(std::string::String::from("2025-11-10")));
    }

    #[test]
    fn test_parse_response_invalid_json() {
        // Test: Validates that the parser returns an error when given invalid JSON.
        // Justification: Ensures the adapter gracefully handles malformed LLM responses and provides
        // clear error messages, which is critical for debugging extraction failures.
        let adapter = OllamaTranscriptExtractorAdapter::new(std::string::String::from("test-model"));
        let invalid_json = "This is not valid JSON";

        let result = adapter.parse_response(invalid_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse"));
    }

    #[test]
    fn test_parse_response_empty_array() {
        // Test: Validates that the parser correctly handles an empty JSON array.
        // Justification: Ensures the adapter handles transcripts with no action items gracefully,
        // which is a valid edge case for informational-only meetings or non-actionable discussions.
        let adapter = OllamaTranscriptExtractorAdapter::new(std::string::String::from("test-model"));
        let empty_json = "[]";

        let result = adapter.parse_response(empty_json);
        assert!(result.is_ok());

        let items = result.unwrap();
        assert_eq!(items.len(), 0);
    }
}
