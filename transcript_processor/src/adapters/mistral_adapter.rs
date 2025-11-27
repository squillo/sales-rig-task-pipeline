//! Mistral.rs-based transcript extractor adapter via OpenAI-compatible HTTP API.
//!
//! This adapter implements the TranscriptExtractorPort by calling a locally running
//! mistralrs-server (OpenAI-compatible) for on-device inference using Phi-3.5-mini-instruct.
//! It constructs a strict JSON-extraction prompt and parses the response using the
//! shared tolerant parser from task_manager::infrastructure.
//!
//! Configuration:
//! - Feature: `mistral_rs` (enable with `--features mistral_rs` on transcript_processor)
//! - Env:
//!   - MISTRALRS_BASE_URL (default: http://127.0.0.1:8080)
//!   - MISTRALRS_MODEL (default: microsoft/Phi-3.5-mini-instruct)
//!
//! Example run:
//!   EXTRACTOR=mistral MISTRALRS_BASE_URL=http://127.0.0.1:8080 cargo run -p transcript_processor --features mistral_rs --
//!
//! Revision History
//! - 2025-11-23T22:10:00Z @AI: Update imports from task_manager::utils to task_manager::infrastructure (HEXSER compliance).
//! - 2025-11-08T10:44:00Z @AI: Initial MistralTranscriptExtractorAdapter via HTTP to mistralrs-server with tolerant parsing reuse.

#[derive(hexser::HexAdapter)]
pub struct MistralTranscriptExtractorAdapter {
    base_url: String,
    model_name: String,
    http: reqwest::Client,
}

impl MistralTranscriptExtractorAdapter {
    /// Creates a new instance configured from environment variables.
    ///
    /// Uses MISTRALRS_BASE_URL and MISTRALRS_MODEL with sensible defaults.
    pub fn new() -> Self {
        let base_url = std::env::var("MISTRALRS_BASE_URL")
            .unwrap_or_else(|_| std::string::String::from("http://127.0.0.1:8080"));
        let model_name = std::env::var("MISTRALRS_MODEL")
            .unwrap_or_else(|_| std::string::String::from("microsoft/Phi-3.5-mini-instruct"));
        let http = reqwest::Client::new();
        Self { base_url, model_name, http }
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

    /// Calls the mistralrs-server chat completions endpoint and returns the raw content string.
    async fn chat_completion(&self, user_prompt: &str) -> std::result::Result<String, String> {
        let url = std::format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));
        // Build OpenAI-compatible request body using serde_json::Value to avoid extra types.
        let system_prompt = "You are a precise extraction engine. Output strictly valid JSON arrays without any prose.";
        let body = serde_json::json!({
            "model": self.model_name,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.0
        });
        let resp = self.http
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| std::format!("HTTP error: {}", e))?;
        if !resp.status().is_success() {
            return std::result::Result::Err(std::format!("HTTP status {}", resp.status()));
        }
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| std::format!("Failed to decode JSON response: {}", e))?;
        // Extract choices[0].message.content
        let content = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|first| first.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| std::string::String::from("Missing content in chat completion response"))?;
        std::result::Result::Ok(content)
    }

    /// Delegates tolerant parsing to shared task_manager infrastructure and maps into local domain type.
    fn parse_response(&self, response_text: &str) -> std::result::Result<std::vec::Vec<crate::domain::action_item::ActionItem>, String> {
        let parsed: std::vec::Vec<task_manager::infrastructure::dtos::extracted_action_item::ExtractedActionItem> =
            task_manager::infrastructure::llm_parsers::action_item_parser::parse_action_items_tolerant(response_text)?;
        let mapped: std::vec::Vec<crate::domain::action_item::ActionItem> = parsed
            .into_iter()
            .map(|e| crate::domain::action_item::ActionItem { title: e.title, assignee: e.assignee, due_date: e.due_date })
            .collect();
        std::result::Result::Ok(mapped)
    }
}

#[async_trait::async_trait]
impl crate::application::ports::transcript_extractor_port::TranscriptExtractorPort for MistralTranscriptExtractorAdapter {
    async fn extract_analysis(
        &self,
        transcript: &str,
    ) -> std::result::Result<crate::domain::transcript_analysis::TranscriptAnalysis, std::string::String> {
        let prompt = self.build_extraction_prompt(transcript);
        let response_text = self.chat_completion(&prompt).await?;
        let items = self.parse_response(&response_text)?;
        std::result::Result::Ok(crate::domain::transcript_analysis::TranscriptAnalysis { action_items: items })
    }
}
