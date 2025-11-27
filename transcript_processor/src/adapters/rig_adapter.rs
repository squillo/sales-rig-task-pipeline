//! Rig-based transcript extractor adapter using OpenAI provider.
//!
//! This adapter implements the TranscriptExtractorPort by leveraging the Rig
//! framework's OpenAI provider to prompt a model for JSON-structured action
//! item extraction. It reuses the shared tolerant parser in task_manager::infrastructure
//! to robustly parse model output into the local ActionItem domain type.
//!
//! Configuration:
//! - Feature: `rig_adapter` (enable with `--features rig_adapter` on transcript_processor)
//! - Env:
//!   - OPENAI_API_KEY (required by rig::providers::openai::Client)
//!   - RIG_OPENAI_MODEL (optional; default: "gpt-4o-mini")
//!
//! Usage example (from transcript_processor crate directory):
//!   EXTRACTOR=rig OPENAI_API_KEY=sk_... cargo run -p transcript_processor --features rig_adapter
//!
//! Revision History
//! - 2025-11-23T22:10:00Z @AI: Update imports from task_manager::utils to task_manager::infrastructure (HEXSER compliance).
//! - 2025-11-08T10:55:00Z @AI: Initial RigTranscriptExtractorAdapter using rig-core OpenAI provider with tolerant JSON parsing.

#[derive(hexser::HexAdapter)]
pub struct RigTranscriptExtractorAdapter {
    model_name: String,
}

impl RigTranscriptExtractorAdapter {
    /// Creates a new adapter configured via environment variables.
    ///
    /// RIG_OPENAI_MODEL may be used to override the default model name.
    pub fn new() -> Self {
        let model = std::env::var("RIG_OPENAI_MODEL")
            .unwrap_or_else(|_| std::string::String::from("gpt-4o-mini"));
        Self { model_name: model }
    }

    /// Builds the strict extraction prompt instructing the model to return only JSON.
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

    /// Delegates tolerant parsing to shared task_manager infrastructure and maps into local domain type.
    fn parse_response(
        &self,
        response_text: &str,
    ) -> std::result::Result<std::vec::Vec<crate::domain::action_item::ActionItem>, String> {
        let parsed: std::vec::Vec<task_manager::infrastructure::dtos::extracted_action_item::ExtractedActionItem> =
            task_manager::infrastructure::llm_parsers::action_item_parser::parse_action_items_tolerant(
                response_text,
            )?;
        let mapped: std::vec::Vec<crate::domain::action_item::ActionItem> = parsed
            .into_iter()
            .map(|e| crate::domain::action_item::ActionItem {
                title: e.title,
                assignee: e.assignee,
                due_date: e.due_date,
            })
            .collect();
        std::result::Result::Ok(mapped)
    }
}

#[async_trait::async_trait]
impl crate::application::ports::transcript_extractor_port::TranscriptExtractorPort
    for RigTranscriptExtractorAdapter
{
    async fn extract_analysis(
        &self,
        transcript: &str,
    ) -> std::result::Result<crate::domain::transcript_analysis::TranscriptAnalysis, std::string::String> {
        // Build prompt
        let prompt = self.build_extraction_prompt(transcript);

        // Initialize Rig OpenAI client from environment
        let client = rig::providers::openai::Client::from_env();
        let agent = client.agent(self.model_name.as_str()).build();

        // Prompt the model (non-streaming) and capture raw text
        let response = rig::completion::Prompt::prompt(&agent, prompt.as_str())
            .await
            .map_err(|e| std::format!("Rig/OpenAI error: {}", e))?;

        // Parse into ActionItems using tolerant parser
        let items = self.parse_response(response.as_str())?;
        std::result::Result::Ok(crate::domain::transcript_analysis::TranscriptAnalysis {
            action_items: items,
        })
    }
}
