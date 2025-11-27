//! Embedded mistral.rs-based transcript extractor adapter using TextModelBuilder.
//!
//! This adapter implements the TranscriptExtractorPort by instantiating a
//! mistralrs text model (Phi-3.5-mini-instruct) in-process with ISQ quantization
//! and paged attention. It constructs a strict JSON-extraction prompt and parses
//! the response using the shared tolerant parser from task_manager::infrastructure.
//!
//! Configuration:
//! - Feature: `mistralrs_embed` (enable with `--features mistralrs_embed` on transcript_processor)
//! - Model: microsoft/Phi-3.5-mini-instruct (downloaded on first run by mistralrs)
//! - Defaults: ISQ=Q4K, paged attention block_size=32, FP8 KV cache (F8E4M3), context size 1024
//!
//! Example run:
//!   EXTRACTOR=mistral_embed cargo run -p transcript_processor --features mistralrs_embed
//!
//! Revision History
//! - 2025-11-23T22:10:00Z @AI: Update imports from task_manager::utils to task_manager::infrastructure (HEXSER compliance).
//! - 2025-11-08T13:05:00Z @AI: Fix API usage: replace `with_cache_type` with `with_paged_cache_type` per mistral.rs docs to resolve build error.
//! - 2025-11-08T11:35:00Z @AI: Initial embedded mistral.rs adapter (ISQ Q4K + paged attention) with tolerant parsing.

#[derive(hexser::HexAdapter)]
pub struct MistralRsEmbeddedTranscriptExtractorAdapter;

impl MistralRsEmbeddedTranscriptExtractorAdapter {
    /// Creates a new instance. All configuration uses sensible defaults.
    pub fn new() -> Self { Self }

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
    ) -> std::result::Result<std::vec::Vec<crate::domain::action_item::ActionItem>, std::string::String> {
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
impl crate::application::ports::transcript_extractor_port::TranscriptExtractorPort for MistralRsEmbeddedTranscriptExtractorAdapter {
    async fn extract_analysis(
        &self,
        transcript: &str,
    ) -> std::result::Result<crate::domain::transcript_analysis::TranscriptAnalysis, std::string::String> {
        // Build the prompt
        let prompt = self.build_extraction_prompt(transcript);

        // Build an embedded mistral.rs Phi-3.5 mini model with ISQ + paged attention
        // Context7 MCP verified API: TextModelBuilder, IsqType, PagedAttentionMetaBuilder, PagedCacheType, MemoryGpuConfig
        let model = mistralrs::TextModelBuilder::new("microsoft/Phi-3.5-mini-instruct")
            .with_isq(mistralrs::IsqType::Q4K)
            .with_logging()
            .with_paged_attn(|| {
                // Default paged attention configuration tuned for consumer hardware
                mistralrs::PagedAttentionMetaBuilder::default()
                    .with_block_size(32)
                    .with_gpu_memory(mistralrs::MemoryGpuConfig::ContextSize(1024))
                    .with_paged_cache_type(mistralrs::PagedCacheType::F8E4M3)
                    .build()
            })
            .map_err(|e| std::format!("Paged attention config error: {}", e))?
            .build()
            .await
            .map_err(|e| std::format!("Failed to build mistral.rs model: {}", e))?;

        let messages = mistralrs::TextMessages::new()
            .add_message(
                mistralrs::TextMessageRole::System,
                "You are a precise extraction engine. Output strictly valid JSON arrays without any prose.",
            )
            .add_message(mistralrs::TextMessageRole::User, prompt.as_str());

        let response = model
            .send_chat_request(messages)
            .await
            .map_err(|e| std::format!("mistral.rs chat request error: {}", e))?;

        let content = response
            .choices
            .get(0)
            .and_then(|c| c.message.content.as_ref())
            .cloned()
            .ok_or_else(|| std::string::String::from("Missing content in mistral.rs chat response"))?;

        let items = self.parse_response(content.as_str())?;
        std::result::Result::Ok(crate::domain::transcript_analysis::TranscriptAnalysis { action_items: items })
    }
}
