//! Rig-powered vision adapter for multimodal LLM image description.
//!
//! RigVisionAdapter provides image and PDF description capabilities using vision-capable
//! LLMs from multiple providers (Ollama LLaVA, OpenAI GPT-4V, Anthropic Claude 3).
//! It enables PRD preprocessing by describing embedded images before task generation.
//!
//! Revision History
//! - 2025-11-30T11:15:00Z @AI: Initial RigVisionAdapter for Phase 5 image processing implementation.

/// Adapter for vision-capable LLM interactions.
///
/// RigVisionAdapter implements VisionPort by making direct HTTP calls to vision
/// LLM APIs. Unlike text completion, vision requires provider-specific request
/// formats for embedding base64 image data.
///
/// # Supported Providers
///
/// * **Ollama**: Uses LLaVA, LLaVA-LLaMA3, BakLLaVA models via `/api/generate`
/// * **OpenAI**: Uses GPT-4o, GPT-4-vision-preview via `/v1/chat/completions`
/// * **Anthropic**: Uses Claude 3 models via `/v1/messages`
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::adapters::rig_vision_adapter::RigVisionAdapter;
/// # use task_orchestrator::ports::vision_port::VisionPort;
/// let adapter = RigVisionAdapter::new_ollama(std::string::String::from("llava"));
///
/// # async fn example(adapter: RigVisionAdapter) {
/// let base64_image = "iVBORw0KGgo..."; // PNG data
/// let response = adapter.describe_image(
///     base64_image,
///     "image/png",
///     std::option::Option::Some("Describe this architecture diagram"),
/// ).await.unwrap();
/// std::println!("Description: {}", response.description);
/// # }
/// ```
pub struct RigVisionAdapter {
    provider: VisionProvider,
    model: String,
    http_client: reqwest::Client,
}

/// Enum representing the vision provider backend.
#[derive(Debug, Clone)]
enum VisionProvider {
    /// Ollama local vision provider with LLaVA models.
    Ollama { base_url: String },
    /// OpenAI remote vision provider with GPT-4V models.
    OpenAI { api_key: String },
    /// Anthropic remote vision provider with Claude 3 models.
    Anthropic { api_key: String },
}

impl RigVisionAdapter {
    /// Creates a new RigVisionAdapter with Ollama provider.
    ///
    /// Uses the default Ollama server at http://localhost:11434.
    ///
    /// # Arguments
    ///
    /// * `model` - The vision model name (e.g., "llava", "llava-llama3", "bakllava")
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_orchestrator::adapters::rig_vision_adapter::RigVisionAdapter;
    /// # use task_orchestrator::ports::vision_port::VisionPort;
    /// let adapter = RigVisionAdapter::new_ollama(std::string::String::from("llava"));
    /// std::assert_eq!(adapter.model_name(), "llava");
    /// std::assert_eq!(adapter.provider_name(), "ollama");
    /// ```
    pub fn new_ollama(model: String) -> Self {
        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        RigVisionAdapter {
            provider: VisionProvider::Ollama { base_url },
            model,
            http_client: reqwest::Client::new(),
        }
    }

    /// Creates a new RigVisionAdapter with OpenAI provider.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The OpenAI API key
    /// * `model` - The vision model name (e.g., "gpt-4o", "gpt-4-vision-preview")
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_orchestrator::adapters::rig_vision_adapter::RigVisionAdapter;
    /// # use task_orchestrator::ports::vision_port::VisionPort;
    /// let adapter = RigVisionAdapter::new_openai(
    ///     std::string::String::from("sk-..."),
    ///     std::string::String::from("gpt-4o"),
    /// );
    /// std::assert_eq!(adapter.provider_name(), "openai");
    /// ```
    pub fn new_openai(api_key: String, model: String) -> Self {
        RigVisionAdapter {
            provider: VisionProvider::OpenAI { api_key },
            model,
            http_client: reqwest::Client::new(),
        }
    }

    /// Creates a new RigVisionAdapter with Anthropic provider.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The Anthropic API key
    /// * `model` - The vision model name (e.g., "claude-3-5-sonnet-20241022", "claude-3-opus-20240229")
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_orchestrator::adapters::rig_vision_adapter::RigVisionAdapter;
    /// # use task_orchestrator::ports::vision_port::VisionPort;
    /// let adapter = RigVisionAdapter::new_anthropic(
    ///     std::string::String::from("sk-ant-..."),
    ///     std::string::String::from("claude-3-5-sonnet-20241022"),
    /// );
    /// std::assert_eq!(adapter.provider_name(), "anthropic");
    /// ```
    pub fn new_anthropic(api_key: String, model: String) -> Self {
        RigVisionAdapter {
            provider: VisionProvider::Anthropic { api_key },
            model,
            http_client: reqwest::Client::new(),
        }
    }

    /// Generates a description using Ollama's vision API.
    ///
    /// Ollama's `/api/generate` endpoint accepts images as a base64 array in the
    /// request body alongside the prompt.
    async fn describe_with_ollama(
        &self,
        base_url: &str,
        base64_data: &str,
        prompt: &str,
    ) -> std::result::Result<crate::ports::vision_port::VisionResponse, std::string::String> {
        let start_time = std::time::Instant::now();

        // Build Ollama generate request with image
        let request_body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "images": [base64_data],
            "stream": false
        });

        let url = std::format!("{}/api/generate", base_url);

        let response = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| std::format!("Ollama vision request failed: {:?}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return std::result::Result::Err(std::format!(
                "Ollama vision API returned {}: {}",
                status,
                body
            ));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| std::format!("Failed to parse Ollama response: {:?}", e))?;

        let description = response_json["response"]
            .as_str()
            .ok_or_else(|| String::from("Ollama response missing 'response' field"))?
            .to_string();

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        std::result::Result::Ok(crate::ports::vision_port::VisionResponse {
            description,
            processing_time_ms,
        })
    }

    /// Generates a description using OpenAI's vision API.
    ///
    /// OpenAI's `/v1/chat/completions` accepts images as data URLs in the
    /// content array with type "image_url".
    async fn describe_with_openai(
        &self,
        api_key: &str,
        base64_data: &str,
        mime_type: &str,
        prompt: &str,
    ) -> std::result::Result<crate::ports::vision_port::VisionResponse, std::string::String> {
        let start_time = std::time::Instant::now();

        // Build data URL for image
        let data_url = std::format!("data:{};base64,{}", mime_type, base64_data);

        // Build OpenAI chat completion request with image
        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": data_url
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 1024
        });

        let response = self
            .http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", std::format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| std::format!("OpenAI vision request failed: {:?}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return std::result::Result::Err(std::format!(
                "OpenAI vision API returned {}: {}",
                status,
                body
            ));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| std::format!("Failed to parse OpenAI response: {:?}", e))?;

        let description = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| String::from("OpenAI response missing content"))?
            .to_string();

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        std::result::Result::Ok(crate::ports::vision_port::VisionResponse {
            description,
            processing_time_ms,
        })
    }

    /// Generates a description using Anthropic's vision API.
    ///
    /// Anthropic's `/v1/messages` accepts images as base64 content blocks
    /// with `source.type: "base64"`.
    async fn describe_with_anthropic(
        &self,
        api_key: &str,
        base64_data: &str,
        mime_type: &str,
        prompt: &str,
    ) -> std::result::Result<crate::ports::vision_port::VisionResponse, std::string::String> {
        let start_time = std::time::Instant::now();

        // Map MIME types to Anthropic's supported media types
        let media_type = match mime_type {
            "image/png" => "image/png",
            "image/jpeg" | "image/jpg" => "image/jpeg",
            "image/gif" => "image/gif",
            "image/webp" => "image/webp",
            _ => {
                return std::result::Result::Err(std::format!(
                    "Anthropic does not support MIME type: {}",
                    mime_type
                ))
            }
        };

        // Build Anthropic messages request with image
        let request_body = serde_json::json!({
            "model": self.model,
            "max_tokens": 1024,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": media_type,
                                "data": base64_data
                            }
                        },
                        {
                            "type": "text",
                            "text": prompt
                        }
                    ]
                }
            ]
        });

        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| std::format!("Anthropic vision request failed: {:?}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return std::result::Result::Err(std::format!(
                "Anthropic vision API returned {}: {}",
                status,
                body
            ));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| std::format!("Failed to parse Anthropic response: {:?}", e))?;

        // Anthropic returns content as an array of blocks
        let description = response_json["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .ok_or_else(|| String::from("Anthropic response missing content text"))?
            .to_string();

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        std::result::Result::Ok(crate::ports::vision_port::VisionResponse {
            description,
            processing_time_ms,
        })
    }

    /// Builds a prompt for image description based on optional context.
    fn build_image_prompt(context: std::option::Option<&str>) -> String {
        match context {
            std::option::Option::Some(ctx) => std::format!(
                "You are analyzing an image. Context: {}\n\n\
                 Please describe this image in detail, focusing on:\n\
                 1. The main content and purpose\n\
                 2. Any text, labels, or annotations\n\
                 3. Key visual elements and their relationships\n\
                 4. Technical details if it appears to be a diagram or technical image\n\n\
                 Provide a clear, structured description that would be useful for understanding \
                 this image without seeing it.",
                ctx
            ),
            std::option::Option::None => String::from(
                "Please describe this image in detail, focusing on:\n\
                 1. The main content and purpose\n\
                 2. Any text, labels, or annotations\n\
                 3. Key visual elements and their relationships\n\
                 4. Technical details if it appears to be a diagram or technical image\n\n\
                 Provide a clear, structured description that would be useful for understanding \
                 this image without seeing it."
            ),
        }
    }

    /// Builds a prompt for PDF page description based on page number and context.
    fn build_pdf_page_prompt(page_number: u32, context: std::option::Option<&str>) -> String {
        match context {
            std::option::Option::Some(ctx) => std::format!(
                "You are analyzing page {} of a PDF document. Context: {}\n\n\
                 Please describe the content of this page in detail, focusing on:\n\
                 1. Main topics and headings\n\
                 2. Any diagrams, charts, or images\n\
                 3. Important text content and key points\n\
                 4. Tables or structured data\n\
                 5. How this page relates to the overall document context\n\n\
                 Provide a structured summary that captures the essential information.",
                page_number, ctx
            ),
            std::option::Option::None => std::format!(
                "You are analyzing page {} of a PDF document.\n\n\
                 Please describe the content of this page in detail, focusing on:\n\
                 1. Main topics and headings\n\
                 2. Any diagrams, charts, or images\n\
                 3. Important text content and key points\n\
                 4. Tables or structured data\n\n\
                 Provide a structured summary that captures the essential information.",
                page_number
            ),
        }
    }
}

#[async_trait::async_trait]
impl crate::ports::vision_port::VisionPort for RigVisionAdapter {
    async fn describe_image(
        &self,
        base64_data: &str,
        mime_type: &str,
        context: std::option::Option<&str>,
    ) -> std::result::Result<crate::ports::vision_port::VisionResponse, std::string::String> {
        // Validate input
        if base64_data.is_empty() {
            return std::result::Result::Err(String::from("Cannot describe empty image data"));
        }

        let prompt = Self::build_image_prompt(context);

        match &self.provider {
            VisionProvider::Ollama { base_url } => {
                self.describe_with_ollama(base_url, base64_data, &prompt)
                    .await
            }
            VisionProvider::OpenAI { api_key } => {
                self.describe_with_openai(api_key, base64_data, mime_type, &prompt)
                    .await
            }
            VisionProvider::Anthropic { api_key } => {
                self.describe_with_anthropic(api_key, base64_data, mime_type, &prompt)
                    .await
            }
        }
    }

    async fn describe_pdf_page(
        &self,
        page_image_base64: &str,
        page_number: u32,
        context: std::option::Option<&str>,
    ) -> std::result::Result<crate::ports::vision_port::VisionResponse, std::string::String> {
        // Validate input
        if page_image_base64.is_empty() {
            return std::result::Result::Err(String::from("Cannot describe empty PDF page data"));
        }

        if page_number == 0 {
            return std::result::Result::Err(String::from("Page number must be >= 1"));
        }

        let prompt = Self::build_pdf_page_prompt(page_number, context);

        // PDF pages are rendered as PNG images for vision processing
        let mime_type = "image/png";

        match &self.provider {
            VisionProvider::Ollama { base_url } => {
                self.describe_with_ollama(base_url, page_image_base64, &prompt)
                    .await
            }
            VisionProvider::OpenAI { api_key } => {
                self.describe_with_openai(api_key, page_image_base64, mime_type, &prompt)
                    .await
            }
            VisionProvider::Anthropic { api_key } => {
                self.describe_with_anthropic(api_key, page_image_base64, mime_type, &prompt)
                    .await
            }
        }
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn provider_name(&self) -> &str {
        match &self.provider {
            VisionProvider::Ollama { .. } => "ollama",
            VisionProvider::OpenAI { .. } => "openai",
            VisionProvider::Anthropic { .. } => "anthropic",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::vision_port::VisionPort;

    #[test]
    fn test_ollama_adapter_creation() {
        // Test: Validates Ollama vision adapter instantiation.
        // Justification: Ensures factory method creates valid adapter with correct provider.
        let adapter = RigVisionAdapter::new_ollama(String::from("llava"));
        assert_eq!(adapter.model_name(), "llava");
        assert_eq!(adapter.provider_name(), "ollama");
    }

    #[test]
    fn test_openai_adapter_creation() {
        // Test: Validates OpenAI vision adapter instantiation.
        // Justification: Ensures factory method creates valid adapter with correct provider.
        let adapter = RigVisionAdapter::new_openai(
            String::from("sk-test-key"),
            String::from("gpt-4o"),
        );
        assert_eq!(adapter.model_name(), "gpt-4o");
        assert_eq!(adapter.provider_name(), "openai");
    }

    #[test]
    fn test_anthropic_adapter_creation() {
        // Test: Validates Anthropic vision adapter instantiation.
        // Justification: Ensures factory method creates valid adapter with correct provider.
        let adapter = RigVisionAdapter::new_anthropic(
            String::from("sk-ant-test-key"),
            String::from("claude-3-5-sonnet-20241022"),
        );
        assert_eq!(adapter.model_name(), "claude-3-5-sonnet-20241022");
        assert_eq!(adapter.provider_name(), "anthropic");
    }

    #[test]
    fn test_build_image_prompt_with_context() {
        // Test: Validates prompt building with context.
        // Justification: Context should be included in the prompt for better descriptions.
        let prompt = RigVisionAdapter::build_image_prompt(
            std::option::Option::Some("Architecture diagram"),
        );
        assert!(prompt.contains("Architecture diagram"));
        assert!(prompt.contains("main content and purpose"));
    }

    #[test]
    fn test_build_image_prompt_without_context() {
        // Test: Validates prompt building without context.
        // Justification: Should provide a valid generic prompt.
        let prompt = RigVisionAdapter::build_image_prompt(std::option::Option::None);
        assert!(prompt.contains("main content and purpose"));
        assert!(!prompt.contains("Context:"));
    }

    #[test]
    fn test_build_pdf_page_prompt_with_context() {
        // Test: Validates PDF page prompt building with context.
        // Justification: Page number and context should be included.
        let prompt = RigVisionAdapter::build_pdf_page_prompt(
            3,
            std::option::Option::Some("API documentation"),
        );
        assert!(prompt.contains("page 3"));
        assert!(prompt.contains("API documentation"));
    }

    #[test]
    fn test_build_pdf_page_prompt_without_context() {
        // Test: Validates PDF page prompt building without context.
        // Justification: Should include page number without context.
        let prompt = RigVisionAdapter::build_pdf_page_prompt(1, std::option::Option::None);
        assert!(prompt.contains("page 1"));
        assert!(!prompt.contains("Context:"));
    }

    #[tokio::test]
    async fn test_empty_image_data_rejection() {
        // Test: Validates rejection of empty image data.
        // Justification: Empty data cannot be processed by vision models.
        let adapter = RigVisionAdapter::new_ollama(String::from("llava"));

        let result = adapter
            .describe_image("", "image/png", std::option::Option::None)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty image data"));
    }

    #[tokio::test]
    async fn test_empty_pdf_page_data_rejection() {
        // Test: Validates rejection of empty PDF page data.
        // Justification: Empty data cannot be processed by vision models.
        let adapter = RigVisionAdapter::new_ollama(String::from("llava"));

        let result = adapter
            .describe_pdf_page("", 1, std::option::Option::None)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty PDF page data"));
    }

    #[tokio::test]
    async fn test_zero_page_number_rejection() {
        // Test: Validates rejection of page number 0.
        // Justification: Page numbers are 1-indexed.
        let adapter = RigVisionAdapter::new_ollama(String::from("llava"));

        let result = adapter
            .describe_pdf_page("iVBORw0KGgo=", 0, std::option::Option::None)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Page number must be >= 1"));
    }

    #[tokio::test]
    #[ignore] // Ignored: Requires running Ollama with llava model
    async fn test_ollama_image_description() {
        // Test: Validates actual Ollama vision processing.
        // Justification: Integration test ensuring end-to-end vision works.
        let adapter = RigVisionAdapter::new_ollama(String::from("llava"));

        // Minimal 1x1 PNG (valid base64)
        let tiny_png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

        let result = adapter
            .describe_image(tiny_png, "image/png", std::option::Option::None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.description.is_empty());
        assert!(response.processing_time_ms > 0);
    }

    #[tokio::test]
    #[ignore] // Ignored: Requires valid OpenAI API key
    async fn test_openai_image_description() {
        // Test: Validates actual OpenAI vision processing.
        // Justification: Integration test ensuring GPT-4V integration works.
        let api_key = std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| String::from("sk-test-key"));

        let adapter = RigVisionAdapter::new_openai(api_key, String::from("gpt-4o"));

        // Minimal 1x1 PNG
        let tiny_png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

        let result = adapter
            .describe_image(tiny_png, "image/png", std::option::Option::None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.description.is_empty());
    }

    #[tokio::test]
    #[ignore] // Ignored: Requires valid Anthropic API key
    async fn test_anthropic_image_description() {
        // Test: Validates actual Anthropic vision processing.
        // Justification: Integration test ensuring Claude 3 integration works.
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .unwrap_or_else(|_| String::from("sk-ant-test-key"));

        let adapter = RigVisionAdapter::new_anthropic(
            api_key,
            String::from("claude-3-5-sonnet-20241022"),
        );

        // Minimal 1x1 PNG
        let tiny_png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

        let result = adapter
            .describe_image(tiny_png, "image/png", std::option::Option::None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.description.is_empty());
    }
}
