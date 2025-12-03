//! Defines the VisionPort output port for vision-capable LLM interactions.
//!
//! This port represents the interface for describing images and PDF pages using
//! multimodal LLMs (GPT-4V, Claude 3, LLaVA). Vision processing enables automatic
//! extraction of textual descriptions from visual content in PRDs.
//!
//! Revision History
//! - 2025-11-30T11:00:00Z @AI: Initial VisionPort for Phase 5 image processing implementation.

/// Response from a vision LLM describing visual content.
///
/// Contains the generated text description along with timing metadata.
#[derive(Debug, Clone)]
pub struct VisionResponse {
    /// Text description of the visual content.
    pub description: String,

    /// Time taken to process the image in milliseconds.
    pub processing_time_ms: u64,
}

/// Port (interface) for vision-capable LLM interactions.
///
/// VisionPort defines the contract for adapters that can describe images and PDFs
/// using multimodal LLMs. Implementations typically use vision models like
/// OpenAI GPT-4V, Anthropic Claude 3, or Ollama LLaVA to analyze visual content
/// and generate textual descriptions.
///
/// # Object Safety
///
/// This trait is object-safe and uses async_trait to support async methods
/// in trait objects. All methods require Send + Sync for concurrent usage.
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::ports::vision_port::VisionPort;
/// # async fn example<V: VisionPort>(vision: &V) {
/// let base64_image = "iVBORw0KGgo..."; // PNG data
/// let response = vision.describe_image(base64_image, "image/png", None).await.unwrap();
/// println!("Image description: {}", response.description);
/// # }
/// ```
#[async_trait::async_trait]
pub trait VisionPort: std::marker::Send + std::marker::Sync {
    /// Describes an image from base64-encoded data.
    ///
    /// This method sends an image to a vision-capable LLM and returns a textual
    /// description of its contents. The description captures key visual elements,
    /// text, diagrams, and other relevant details.
    ///
    /// # Arguments
    ///
    /// * `base64_data` - The image data encoded as base64 string
    /// * `mime_type` - MIME type of the image (e.g., "image/png", "image/jpeg")
    /// * `context` - Optional context to guide the description (e.g., "This is an architecture diagram")
    ///
    /// # Returns
    ///
    /// A Result containing the VisionResponse with description, or an error if processing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - LLM vision request fails
    /// - Image data is invalid or corrupted
    /// - MIME type is unsupported
    /// - Model is unavailable
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::ports::vision_port::VisionPort;
    /// # async fn example<V: VisionPort>(vision: &V) {
    /// let png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJ";
    /// let response = vision.describe_image(
    ///     png_base64,
    ///     "image/png",
    ///     std::option::Option::Some("Describe this architecture diagram"),
    /// ).await.unwrap();
    /// std::assert!(!response.description.is_empty());
    /// # }
    /// ```
    async fn describe_image(
        &self,
        base64_data: &str,
        mime_type: &str,
        context: std::option::Option<&str>,
    ) -> std::result::Result<VisionResponse, std::string::String>;

    /// Describes a PDF page rendered as an image.
    ///
    /// This method processes a single page of a PDF document by rendering it
    /// as an image and describing its contents. For multi-page PDFs, call this
    /// method once per page.
    ///
    /// # Arguments
    ///
    /// * `page_image_base64` - The rendered PDF page as base64-encoded PNG image
    /// * `page_number` - The page number (1-indexed) for context
    /// * `context` - Optional context about the document (e.g., "Technical specification document")
    ///
    /// # Returns
    ///
    /// A Result containing the VisionResponse with page description, or an error if processing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - LLM vision request fails
    /// - Image data is invalid
    /// - Model is unavailable
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::ports::vision_port::VisionPort;
    /// # async fn example<V: VisionPort>(vision: &V) {
    /// let page_png = "iVBORw0KGgoAAAANSUhEUg..."; // Rendered PDF page
    /// let response = vision.describe_pdf_page(
    ///     page_png,
    ///     1,
    ///     std::option::Option::Some("API documentation"),
    /// ).await.unwrap();
    /// println!("Page 1: {}", response.description);
    /// # }
    /// ```
    async fn describe_pdf_page(
        &self,
        page_image_base64: &str,
        page_number: u32,
        context: std::option::Option<&str>,
    ) -> std::result::Result<VisionResponse, std::string::String>;

    /// Returns the name of the underlying vision model.
    ///
    /// # Returns
    ///
    /// The model identifier string (e.g., "llava", "gpt-4o", "claude-3-sonnet").
    fn model_name(&self) -> &str;

    /// Returns the name of the vision provider.
    ///
    /// # Returns
    ///
    /// The provider name (e.g., "ollama", "openai", "anthropic").
    fn provider_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_response_creation() {
        // Test: Verifies VisionResponse can be created with expected fields.
        // Justification: VisionResponse is the output type for all vision operations.
        let response = VisionResponse {
            description: String::from("An architecture diagram showing microservices"),
            processing_time_ms: 1500,
        };

        std::assert_eq!(response.description, "An architecture diagram showing microservices");
        std::assert_eq!(response.processing_time_ms, 1500);
    }

    #[test]
    fn test_vision_response_clone() {
        // Test: Verifies VisionResponse implements Clone correctly.
        // Justification: Responses may need to be stored and copied.
        let response = VisionResponse {
            description: String::from("Test description"),
            processing_time_ms: 100,
        };

        let cloned = response.clone();
        std::assert_eq!(cloned.description, response.description);
        std::assert_eq!(cloned.processing_time_ms, response.processing_time_ms);
    }
}
