//! Vision service for PRD media processing.
//!
//! VisionService coordinates the scanning, fetching, and processing of media
//! (images and PDFs) embedded in PRD documents. It uses regex patterns to detect
//! media URLs, fetches them asynchronously, generates descriptions via vision LLMs,
//! and injects the descriptions back into the PRD content for task generation.
//!
//! # Processing Flow
//!
//! 1. Scan PRD content for media URLs (markdown images, HTML img tags, raw URLs)
//! 2. Fetch each media item and encode as base64
//! 3. Send to vision LLM for description generation
//! 4. Create Artifact entities with binary content and descriptions
//! 5. Inject descriptions into PRD content adjacent to original references
//!
//! # URL Detection Patterns
//!
//! - Markdown: `![alt text](url)`
//! - HTML: `<img src="url">`
//! - Raw URLs: `https://....(png|jpg|jpeg|gif|webp|pdf)`
//!
//! Revision History
//! - 2025-11-30T13:00:00Z @AI: Phase 6 PDF support - add extract_pdf_text() using pdf-extract crate for text extraction from PDF documents. PDFs with substantial text use extracted text as description, while image-heavy PDFs fall back to vision LLM. Added process_pdf_content() helper for multi-page handling and PdfProcessingResult struct.
//! - 2025-11-30T11:45:00Z @AI: Initial VisionService for Phase 3 media processing implementation.

/// Result of scanning PRD content for media references.
#[derive(Debug, Clone)]
pub struct MediaReference {
    /// The URL to the media resource.
    pub url: String,

    /// Type of media (Image or PDF).
    pub media_type: MediaType,

    /// Alt text from markdown images, if present.
    pub alt_text: std::option::Option<String>,

    /// Start position of the match in the original text.
    pub start_pos: usize,

    /// End position of the match in the original text.
    pub end_pos: usize,

    /// The original matched text (for replacement).
    pub original_match: String,
}

/// Categorizes detected media by type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    /// Image file (PNG, JPG, GIF, WebP, SVG).
    Image,
    /// PDF document.
    PDF,
}

/// Result of processing a PRD's media content.
#[derive(Debug, Clone)]
pub struct ProcessedPrdMedia {
    /// The PRD content with descriptions injected.
    pub enhanced_content: String,

    /// Artifacts created from processed media.
    pub artifacts: std::vec::Vec<task_manager::domain::artifact::Artifact>,

    /// Processing statistics.
    pub stats: ProcessingStats,
}

/// Statistics from media processing.
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    /// Number of media URLs detected.
    pub total_detected: usize,

    /// Number of media items successfully processed.
    pub successfully_processed: usize,

    /// Number of media items that failed to process.
    pub failed: usize,

    /// Total processing time in milliseconds.
    pub total_time_ms: u64,
}

/// Progress update during media processing.
#[derive(Debug, Clone)]
pub struct MediaProgress {
    /// Current item being processed (1-indexed).
    pub current: usize,

    /// Total number of items to process.
    pub total: usize,

    /// URL of the current item.
    pub url: String,

    /// Status message.
    pub status: String,
}

/// Result of PDF text extraction.
#[derive(Debug, Clone)]
pub struct PdfProcessingResult {
    /// Extracted text from all pages.
    pub text: String,

    /// Number of pages in the PDF.
    pub page_count: usize,

    /// Whether the PDF is text-heavy (true) or image-heavy (false).
    pub is_text_heavy: bool,

    /// Per-page text content (for multi-page PDFs).
    pub pages: std::vec::Vec<String>,
}

/// Service for processing media in PRD documents.
///
/// VisionService orchestrates the detection, fetching, and description of
/// images and PDFs embedded in PRDs. It uses a VisionPort adapter to generate
/// descriptions using vision-capable LLMs.
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::services::vision_service::VisionService;
/// # use task_orchestrator::adapters::rig_vision_adapter::RigVisionAdapter;
/// # async fn example() {
/// let vision_adapter = std::sync::Arc::new(
///     RigVisionAdapter::new_ollama(std::string::String::from("llava"))
/// );
/// let service = VisionService::new(vision_adapter);
///
/// let prd_content = "# My PRD\n\n![Architecture](https://example.com/arch.png)\n\nDescription here.";
/// let result = service.process_prd_media(
///     prd_content,
///     "project-123",
///     "prd-456",
///     std::option::Option::None,
/// ).await.unwrap();
///
/// std::println!("Enhanced content:\n{}", result.enhanced_content);
/// # }
/// ```
pub struct VisionService {
    vision_port: std::sync::Arc<dyn crate::ports::vision_port::VisionPort + std::marker::Send + std::marker::Sync>,
    http_client: reqwest::Client,
}

impl VisionService {
    /// Creates a new VisionService with the given vision port.
    ///
    /// # Arguments
    ///
    /// * `vision_port` - The vision LLM adapter for generating descriptions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::services::vision_service::VisionService;
    /// # use task_orchestrator::adapters::rig_vision_adapter::RigVisionAdapter;
    /// let adapter = std::sync::Arc::new(
    ///     RigVisionAdapter::new_ollama(std::string::String::from("llava"))
    /// );
    /// let service = VisionService::new(adapter);
    /// ```
    pub fn new(
        vision_port: std::sync::Arc<dyn crate::ports::vision_port::VisionPort + std::marker::Send + std::marker::Sync>,
    ) -> Self {
        VisionService {
            vision_port,
            http_client: reqwest::Client::new(),
        }
    }

    /// Scans PRD content for media URLs.
    ///
    /// Detects URLs in three formats:
    /// - Markdown images: `![alt text](url)`
    /// - HTML img tags: `<img src="url">`
    /// - Raw URLs ending in image/pdf extensions
    ///
    /// # Arguments
    ///
    /// * `content` - The PRD content to scan.
    ///
    /// # Returns
    ///
    /// A vector of MediaReference objects for each detected URL.
    pub fn scan_for_media_urls(&self, content: &str) -> std::vec::Vec<MediaReference> {
        let mut references = std::vec::Vec::new();

        // Pattern 1: Markdown images - ![alt text](url)
        let markdown_pattern = regex::Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
        for cap in markdown_pattern.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let alt_text = cap.get(1).map(|m| m.as_str().to_string());
            let url = cap.get(2).unwrap().as_str().to_string();

            if let std::option::Option::Some(media_type) = Self::detect_media_type(&url) {
                references.push(MediaReference {
                    url,
                    media_type,
                    alt_text,
                    start_pos: full_match.start(),
                    end_pos: full_match.end(),
                    original_match: full_match.as_str().to_string(),
                });
            }
        }

        // Pattern 2: HTML img tags - <img src="url">
        let html_pattern = regex::Regex::new(r#"<img[^>]+src=["']([^"']+)["'][^>]*>"#).unwrap();
        for cap in html_pattern.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let url = cap.get(1).unwrap().as_str().to_string();

            // Skip if already captured by markdown
            if references.iter().any(|r| r.start_pos == full_match.start()) {
                continue;
            }

            if let std::option::Option::Some(media_type) = Self::detect_media_type(&url) {
                // Try to extract alt attribute
                let alt_pattern = regex::Regex::new(r#"alt=["']([^"']*)["']"#).unwrap();
                let alt_text = alt_pattern
                    .captures(full_match.as_str())
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string());

                references.push(MediaReference {
                    url,
                    media_type,
                    alt_text,
                    start_pos: full_match.start(),
                    end_pos: full_match.end(),
                    original_match: full_match.as_str().to_string(),
                });
            }
        }

        // Pattern 3: Raw URLs ending in media extensions
        let raw_url_pattern = regex::Regex::new(
            r"https?://[^\s<>\[\]]+\.(png|jpg|jpeg|gif|webp|svg|pdf)"
        ).unwrap();
        for cap in raw_url_pattern.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let url = full_match.as_str().to_string();

            // Skip if already captured by markdown or HTML
            if references.iter().any(|r| r.start_pos <= full_match.start() && r.end_pos >= full_match.end()) {
                continue;
            }

            if let std::option::Option::Some(media_type) = Self::detect_media_type(&url) {
                references.push(MediaReference {
                    url,
                    media_type,
                    alt_text: std::option::Option::None,
                    start_pos: full_match.start(),
                    end_pos: full_match.end(),
                    original_match: full_match.as_str().to_string(),
                });
            }
        }

        // Sort by position for consistent processing
        references.sort_by_key(|r| r.start_pos);
        references
    }

    /// Detects the media type from a URL based on file extension.
    fn detect_media_type(url: &str) -> std::option::Option<MediaType> {
        let lower_url = url.to_lowercase();
        if lower_url.ends_with(".png")
            || lower_url.ends_with(".jpg")
            || lower_url.ends_with(".jpeg")
            || lower_url.ends_with(".gif")
            || lower_url.ends_with(".webp")
            || lower_url.ends_with(".svg")
        {
            std::option::Option::Some(MediaType::Image)
        } else if lower_url.ends_with(".pdf") {
            std::option::Option::Some(MediaType::PDF)
        } else {
            std::option::Option::None
        }
    }

    /// Fetches media from a URL and returns base64-encoded data with MIME type.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch.
    ///
    /// # Returns
    ///
    /// A tuple of (base64_data, mime_type) or an error.
    pub async fn fetch_media(
        &self,
        url: &str,
    ) -> std::result::Result<(String, String), String> {
        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| std::format!("Failed to fetch media from {}: {:?}", url, e))?;

        if !response.status().is_success() {
            return std::result::Result::Err(std::format!(
                "HTTP {} fetching {}",
                response.status(),
                url
            ));
        }

        // Determine MIME type from Content-Type header or URL extension
        let mime_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(';').next().unwrap_or(s).trim().to_string())
            .unwrap_or_else(|| Self::mime_type_from_url(url));

        let bytes = response
            .bytes()
            .await
            .map_err(|e| std::format!("Failed to read media bytes: {:?}", e))?;

        let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

        std::result::Result::Ok((base64_data, mime_type))
    }

    /// Determines MIME type from URL extension.
    fn mime_type_from_url(url: &str) -> String {
        let lower_url = url.to_lowercase();
        if lower_url.ends_with(".png") {
            "image/png".to_string()
        } else if lower_url.ends_with(".jpg") || lower_url.ends_with(".jpeg") {
            "image/jpeg".to_string()
        } else if lower_url.ends_with(".gif") {
            "image/gif".to_string()
        } else if lower_url.ends_with(".webp") {
            "image/webp".to_string()
        } else if lower_url.ends_with(".svg") {
            "image/svg+xml".to_string()
        } else if lower_url.ends_with(".pdf") {
            "application/pdf".to_string()
        } else {
            "application/octet-stream".to_string()
        }
    }

    /// Extracts text content from a PDF using pdf-extract.
    ///
    /// This method attempts to extract text from all pages of a PDF document.
    /// It determines whether the PDF is "text-heavy" (substantial extractable text)
    /// or "image-heavy" (minimal text, likely contains diagrams/charts).
    ///
    /// # Arguments
    ///
    /// * `pdf_bytes` - Raw PDF bytes (not base64 encoded).
    ///
    /// # Returns
    ///
    /// PdfProcessingResult containing extracted text and metadata.
    ///
    /// # Notes
    ///
    /// - PDFs with less than 50 characters per page are considered image-heavy.
    /// - For image-heavy PDFs, vision LLM processing is recommended.
    pub fn extract_pdf_text(pdf_bytes: &[u8]) -> std::result::Result<PdfProcessingResult, String> {
        // Extract text using pdf-extract
        let text = pdf_extract::extract_text_from_mem(pdf_bytes)
            .map_err(|e| std::format!("Failed to extract text from PDF: {:?}", e))?;

        // Try to estimate page count by looking for form feed characters or page markers
        // pdf-extract doesn't provide page-by-page access easily, so we estimate
        let page_separators: std::vec::Vec<&str> = text.split('\u{000C}').collect(); // Form feed
        let page_count = if page_separators.len() > 1 {
            page_separators.len()
        } else {
            // Estimate based on text length (rough heuristic)
            std::cmp::max(1, text.len() / 3000)
        };

        // Split text into pages (best effort)
        let pages: std::vec::Vec<String> = page_separators
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Determine if PDF is text-heavy
        // Threshold: at least 50 characters of text per estimated page
        let avg_chars_per_page = if page_count > 0 {
            text.len() / page_count
        } else {
            0
        };
        let is_text_heavy = avg_chars_per_page >= 50;

        std::result::Result::Ok(PdfProcessingResult {
            text: text.trim().to_string(),
            page_count,
            is_text_heavy,
            pages: if pages.is_empty() {
                std::vec![text.trim().to_string()]
            } else {
                pages
            },
        })
    }

    /// Processes PDF content - extracts text if available, falls back to vision LLM.
    ///
    /// # Arguments
    ///
    /// * `base64_data` - Base64-encoded PDF data.
    /// * `url` - Original URL for error messages.
    ///
    /// # Returns
    ///
    /// A description string derived from text extraction or vision LLM.
    pub async fn process_pdf_content(
        &self,
        base64_data: &str,
        url: &str,
    ) -> std::result::Result<String, String> {
        // Decode base64 to get raw bytes
        let pdf_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_data)
            .map_err(|e| std::format!("Failed to decode PDF base64: {:?}", e))?;

        // Try text extraction first
        match Self::extract_pdf_text(&pdf_bytes) {
            std::result::Result::Ok(result) => {
                if result.is_text_heavy && !result.text.is_empty() {
                    // Text-heavy PDF - use extracted text
                    let summary = if result.page_count > 1 {
                        std::format!(
                            "PDF Document ({} pages):\n\n{}",
                            result.page_count,
                            Self::summarize_text(&result.text, 2000)
                        )
                    } else {
                        std::format!(
                            "PDF Document:\n\n{}",
                            Self::summarize_text(&result.text, 2000)
                        )
                    };
                    return std::result::Result::Ok(summary);
                }
                // Image-heavy PDF - fall through to vision LLM
                eprintln!(
                    "PDF at {} appears image-heavy ({} chars), using vision LLM",
                    url,
                    result.text.len()
                );
            }
            std::result::Result::Err(e) => {
                // Text extraction failed - fall through to vision LLM
                eprintln!("PDF text extraction failed for {}: {}", url, e);
            }
        }

        // Fall back to vision LLM for image-heavy or failed PDFs
        let vision_result = self.vision_port
            .describe_pdf_page(base64_data, 1, std::option::Option::None)
            .await?;

        std::result::Result::Ok(vision_result.description)
    }

    /// Truncates text to a maximum length, trying to break at sentence boundaries.
    fn summarize_text(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            return text.to_string();
        }

        // Try to find a sentence boundary near max_length
        let truncated = &text[..max_length];
        if let std::option::Option::Some(pos) = truncated.rfind(". ") {
            std::format!("{}...", &text[..pos + 1])
        } else if let std::option::Option::Some(pos) = truncated.rfind('\n') {
            std::format!("{}...", &text[..pos])
        } else {
            std::format!("{}...", truncated)
        }
    }

    /// Processes all media in a PRD document.
    ///
    /// This is the main entry point for media processing. It:
    /// 1. Scans for media URLs
    /// 2. Fetches and describes each item
    /// 3. Creates artifacts with binary content
    /// 4. Injects descriptions into the PRD
    ///
    /// # Arguments
    ///
    /// * `prd_content` - The original PRD content.
    /// * `project_id` - Project ID for artifact creation.
    /// * `prd_id` - PRD ID for artifact source tracking.
    /// * `progress_callback` - Optional callback for progress updates.
    ///
    /// # Returns
    ///
    /// ProcessedPrdMedia containing enhanced content and created artifacts.
    pub async fn process_prd_media(
        &self,
        prd_content: &str,
        project_id: &str,
        prd_id: &str,
        progress_callback: std::option::Option<tokio::sync::mpsc::Sender<MediaProgress>>,
    ) -> std::result::Result<ProcessedPrdMedia, String> {
        let start_time = std::time::Instant::now();

        // 1. Scan for media URLs
        let references = self.scan_for_media_urls(prd_content);

        if references.is_empty() {
            return std::result::Result::Ok(ProcessedPrdMedia {
                enhanced_content: prd_content.to_string(),
                artifacts: std::vec::Vec::new(),
                stats: ProcessingStats {
                    total_detected: 0,
                    successfully_processed: 0,
                    failed: 0,
                    total_time_ms: start_time.elapsed().as_millis() as u64,
                },
            });
        }

        let total = references.len();
        let mut artifacts = std::vec::Vec::new();
        let mut descriptions: std::vec::Vec<(usize, String, String)> = std::vec::Vec::new(); // (end_pos, url, description)
        let mut failed = 0;

        // 2. Process each media item
        for (idx, media_ref) in references.iter().enumerate() {
            // Send progress update
            if let std::option::Option::Some(ref tx) = progress_callback {
                let _ = tx.send(MediaProgress {
                    current: idx + 1,
                    total,
                    url: media_ref.url.clone(),
                    status: "Processing".to_string(),
                }).await;
            }

            // Fetch media
            let fetch_result = self.fetch_media(&media_ref.url).await;
            let (base64_data, mime_type) = match fetch_result {
                std::result::Result::Ok(data) => data,
                std::result::Result::Err(e) => {
                    eprintln!("Failed to fetch {}: {}", media_ref.url, e);
                    failed += 1;
                    continue;
                }
            };

            // Generate description based on media type
            let description = match media_ref.media_type {
                MediaType::Image => {
                    let context = media_ref.alt_text.as_deref();
                    match self.vision_port
                        .describe_image(&base64_data, &mime_type, context)
                        .await
                    {
                        std::result::Result::Ok(resp) => resp.description,
                        std::result::Result::Err(e) => {
                            eprintln!("Failed to describe image {}: {}", media_ref.url, e);
                            failed += 1;
                            continue;
                        }
                    }
                }
                MediaType::PDF => {
                    // Phase 6: Use PDF text extraction with vision LLM fallback
                    match self.process_pdf_content(&base64_data, &media_ref.url).await {
                        std::result::Result::Ok(desc) => desc,
                        std::result::Result::Err(e) => {
                            eprintln!("Failed to process PDF {}: {}", media_ref.url, e);
                            failed += 1;
                            continue;
                        }
                    }
                }
            };

            // Store description for later injection
            descriptions.push((
                media_ref.end_pos,
                media_ref.url.clone(),
                description.clone(),
            ));

            // Create artifact
            let artifact_type = match media_ref.media_type {
                MediaType::Image => task_manager::domain::artifact::ArtifactType::Image,
                MediaType::PDF => task_manager::domain::artifact::ArtifactType::PDF,
            };

            // Note: In a full implementation, we'd generate embeddings for the description
            // For now, use an empty embedding vector (will be filled by artifact service)
            let artifact = task_manager::domain::artifact::Artifact::new_media(
                project_id.to_string(),
                prd_id.to_string(),
                artifact_type,
                description,
                std::vec::Vec::new(), // Empty embedding - to be filled later
                base64_data,
                mime_type,
                media_ref.url.clone(),
                std::option::Option::None, // page_number for images
            );

            artifacts.push(artifact);
        }

        // 3. Inject descriptions into PRD content (in reverse order to preserve positions)
        let mut enhanced_content = prd_content.to_string();
        let mut sorted_descriptions = descriptions.clone();
        sorted_descriptions.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by position descending

        for (end_pos, url, description) in sorted_descriptions {
            // Determine if this is a PDF or image based on URL
            let label = if url.to_lowercase().ends_with(".pdf") {
                "PDF Content"
            } else {
                "Image Description"
            };
            let injection = std::format!(
                "\n\n> **{}**: {}\n",
                label,
                description.replace('\n', "\n> ")
            );
            enhanced_content.insert_str(end_pos, &injection);
        }

        std::result::Result::Ok(ProcessedPrdMedia {
            enhanced_content,
            artifacts,
            stats: ProcessingStats {
                total_detected: total,
                successfully_processed: total - failed,
                failed,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_markdown_images() {
        // Test: Validates detection of markdown image syntax.
        // Justification: Markdown images are the most common format in PRDs.
        let service = create_test_service();
        let content = "# Header\n\n![Architecture Diagram](https://example.com/arch.png)\n\nSome text.";

        let refs = service.scan_for_media_urls(content);

        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].url, "https://example.com/arch.png");
        assert_eq!(refs[0].media_type, MediaType::Image);
        assert_eq!(refs[0].alt_text, std::option::Option::Some(String::from("Architecture Diagram")));
    }

    #[test]
    fn test_scan_html_img_tags() {
        // Test: Validates detection of HTML img tags.
        // Justification: Some PRDs use HTML for more control over images.
        let service = create_test_service();
        let content = "Text before <img src=\"https://example.com/logo.jpg\" alt=\"Logo\"> text after";

        let refs = service.scan_for_media_urls(content);

        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].url, "https://example.com/logo.jpg");
        assert_eq!(refs[0].media_type, MediaType::Image);
        assert_eq!(refs[0].alt_text, std::option::Option::Some(String::from("Logo")));
    }

    #[test]
    fn test_scan_raw_urls() {
        // Test: Validates detection of raw image URLs.
        // Justification: Some users paste raw URLs without markdown formatting.
        let service = create_test_service();
        let content = "Check this diagram: https://example.com/diagram.png and this PDF https://example.com/spec.pdf";

        let refs = service.scan_for_media_urls(content);

        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].media_type, MediaType::Image);
        assert_eq!(refs[1].media_type, MediaType::PDF);
    }

    #[test]
    fn test_scan_multiple_formats() {
        // Test: Validates detection of mixed formats in a single document.
        // Justification: Real PRDs often mix markdown and HTML.
        let service = create_test_service();
        let content = r#"
# PRD

![Diagram](https://example.com/a.png)

<img src="https://example.com/b.jpg">

Raw: https://example.com/c.gif
"#;

        let refs = service.scan_for_media_urls(content);

        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn test_scan_no_media() {
        // Test: Validates handling of PRDs with no media.
        // Justification: Many PRDs are text-only.
        let service = create_test_service();
        let content = "# Header\n\nJust text, no images.";

        let refs = service.scan_for_media_urls(content);

        assert!(refs.is_empty());
    }

    #[test]
    fn test_detect_media_type_images() {
        // Test: Validates correct media type detection for images.
        // Justification: Correct MIME type is essential for vision LLM processing.
        assert_eq!(
            VisionService::detect_media_type("https://example.com/img.png"),
            std::option::Option::Some(MediaType::Image)
        );
        assert_eq!(
            VisionService::detect_media_type("https://example.com/img.jpg"),
            std::option::Option::Some(MediaType::Image)
        );
        assert_eq!(
            VisionService::detect_media_type("https://example.com/img.JPEG"),
            std::option::Option::Some(MediaType::Image)
        );
    }

    #[test]
    fn test_detect_media_type_pdf() {
        // Test: Validates correct media type detection for PDFs.
        // Justification: PDFs require page-by-page processing.
        assert_eq!(
            VisionService::detect_media_type("https://example.com/doc.pdf"),
            std::option::Option::Some(MediaType::PDF)
        );
        assert_eq!(
            VisionService::detect_media_type("https://example.com/doc.PDF"),
            std::option::Option::Some(MediaType::PDF)
        );
    }

    #[test]
    fn test_detect_media_type_unknown() {
        // Test: Validates handling of unsupported extensions.
        // Justification: Should gracefully skip unknown file types.
        assert_eq!(
            VisionService::detect_media_type("https://example.com/doc.txt"),
            std::option::Option::None
        );
        assert_eq!(
            VisionService::detect_media_type("https://example.com/doc.html"),
            std::option::Option::None
        );
    }

    #[test]
    fn test_mime_type_from_url() {
        // Test: Validates MIME type derivation from URL.
        // Justification: Fallback when Content-Type header is missing.
        assert_eq!(VisionService::mime_type_from_url("https://example.com/img.png"), "image/png");
        assert_eq!(VisionService::mime_type_from_url("https://example.com/img.jpg"), "image/jpeg");
        assert_eq!(VisionService::mime_type_from_url("https://example.com/doc.pdf"), "application/pdf");
    }

    // Helper to create a test service with a mock vision port
    fn create_test_service() -> VisionService {
        VisionService {
            vision_port: std::sync::Arc::new(MockVisionPort),
            http_client: reqwest::Client::new(),
        }
    }

    // Mock vision port for tests that don't need actual LLM calls
    struct MockVisionPort;

    #[async_trait::async_trait]
    impl crate::ports::vision_port::VisionPort for MockVisionPort {
        async fn describe_image(
            &self,
            _base64_data: &str,
            _mime_type: &str,
            _context: std::option::Option<&str>,
        ) -> std::result::Result<crate::ports::vision_port::VisionResponse, String> {
            std::result::Result::Ok(crate::ports::vision_port::VisionResponse {
                description: String::from("Mock description"),
                processing_time_ms: 100,
            })
        }

        async fn describe_pdf_page(
            &self,
            _page_image_base64: &str,
            _page_number: u32,
            _context: std::option::Option<&str>,
        ) -> std::result::Result<crate::ports::vision_port::VisionResponse, String> {
            std::result::Result::Ok(crate::ports::vision_port::VisionResponse {
                description: String::from("Mock PDF page description"),
                processing_time_ms: 100,
            })
        }

        fn model_name(&self) -> &str {
            "mock-model"
        }

        fn provider_name(&self) -> &str {
            "mock"
        }
    }
}
