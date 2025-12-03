//! Port interface for web crawling operations.
//!
//! This module defines the WebCrawlerPort trait for crawling websites and
//! extracting text content. The port enables hexagonal architecture by
//! abstracting the HTTP client and HTML parsing implementation.
//!
//! Revision History
//! - 2025-11-30T19:45:00Z @AI: Initial WebCrawlerPort for Phase 3 artifact generator.

/// Port trait for web crawling with content extraction.
///
/// WebCrawlerPort defines the interface for crawling websites to discover
/// pages and extract text content for artifact generation. Implementations
/// handle HTTP requests, link following, rate limiting, and HTML parsing.
///
/// # Design Notes
///
/// This is a "driven" port in hexagonal architecture terms - the application
/// core drives calls to this port, and adapters implement it to interact with
/// the web. The trait is async to support non-blocking HTTP operations.
///
/// # Examples
///
/// ```ignore
/// # use task_orchestrator::ports::web_crawler_port::WebCrawlerPort;
/// # use task_orchestrator::domain::crawl_result::CrawlConfig;
/// async fn example<C: WebCrawlerPort>(crawler: &C) {
///     let config = CrawlConfig::new(std::string::String::from("https://docs.rs/rig"));
///     let result = crawler.crawl(&config).await;
///     match result {
///         std::result::Result::Ok(pages) => println!("Crawled {} pages", pages.pages.len()),
///         std::result::Result::Err(e) => eprintln!("Crawl failed: {:?}", e),
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait WebCrawlerPort: std::marker::Send + std::marker::Sync {
    /// Crawls a website starting from the configured URL.
    ///
    /// Fetches pages starting from `config.start_url`, following links up to
    /// the configured depth and page limit. Extracts text content from HTML
    /// and returns all successfully crawled pages.
    ///
    /// # Arguments
    ///
    /// * `config` - Crawl configuration specifying URL, depth, limits, and options.
    ///
    /// # Returns
    ///
    /// * `Ok(CrawlResult)` - Successfully crawled pages and statistics.
    /// * `Err(CrawlError)` - Crawl failed (invalid URL, network error, etc.).
    ///
    /// # Errors
    ///
    /// Returns `CrawlError::InvalidUrl` if the start URL is malformed.
    /// Returns `CrawlError::NetworkError` if the initial request fails.
    async fn crawl(
        &self,
        config: &crate::domain::crawl_result::CrawlConfig,
    ) -> std::result::Result<CrawlResult, crate::domain::crawl_result::CrawlError>;

    /// Fetches a single page and extracts its content.
    ///
    /// This method is useful for fetching individual pages without following
    /// links. It respects rate limiting if configured.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch.
    /// * `config` - Crawl configuration for request settings.
    ///
    /// # Returns
    ///
    /// * `Ok(CrawledPage)` - Page fetched and content extracted.
    /// * `Err(CrawlError)` - Failed to fetch or parse page.
    async fn fetch_page(
        &self,
        url: &str,
        config: &crate::domain::crawl_result::CrawlConfig,
    ) -> std::result::Result<
        crate::domain::crawl_result::CrawledPage,
        crate::domain::crawl_result::CrawlError,
    >;

    /// Extracts text content from HTML.
    ///
    /// Parses HTML and extracts readable text, optionally using CSS selectors
    /// to target specific content areas and exclude navigation/ads.
    ///
    /// # Arguments
    ///
    /// * `html` - Raw HTML content.
    /// * `config` - Crawl configuration with content/exclude selectors.
    ///
    /// # Returns
    ///
    /// Extracted text content with HTML tags stripped.
    fn extract_text(
        &self,
        html: &str,
        config: &crate::domain::crawl_result::CrawlConfig,
    ) -> String;

    /// Extracts the page title from HTML.
    ///
    /// Finds the content of the `<title>` tag in the HTML.
    ///
    /// # Arguments
    ///
    /// * `html` - Raw HTML content.
    ///
    /// # Returns
    ///
    /// The page title, or empty string if not found.
    fn extract_title(&self, html: &str) -> String;

    /// Extracts links from HTML.
    ///
    /// Finds all `<a href="...">` links in the HTML and returns their URLs.
    /// Relative URLs are resolved against the base URL.
    ///
    /// # Arguments
    ///
    /// * `html` - Raw HTML content.
    /// * `base_url` - Base URL for resolving relative links.
    ///
    /// # Returns
    ///
    /// List of absolute URLs found in the page.
    fn extract_links(&self, html: &str, base_url: &str) -> std::vec::Vec<String>;

    /// Checks if a URL should be followed based on configuration.
    ///
    /// Determines whether a discovered link should be added to the crawl queue
    /// based on domain restrictions, depth limits, and exclusion patterns.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check.
    /// * `base_url` - The starting URL's base domain.
    /// * `config` - Crawl configuration with follow rules.
    ///
    /// # Returns
    ///
    /// `true` if the URL should be crawled, `false` otherwise.
    fn should_follow(&self, url: &str, base_url: &str, config: &crate::domain::crawl_result::CrawlConfig) -> bool;
}

/// Result of a web crawl operation.
///
/// CrawlResult contains all successfully crawled pages along with statistics
/// about the crawl operation. This enables batch processing and progress reporting.
///
/// # Fields
///
/// * `pages` - Successfully crawled pages with extracted content.
/// * `stats` - Statistics about the crawl operation.
/// * `errors` - Non-fatal errors encountered during crawling.
#[derive(Debug, Clone)]
pub struct CrawlResult {
    /// Successfully crawled pages.
    pub pages: std::vec::Vec<crate::domain::crawl_result::CrawledPage>,

    /// Statistics about the crawl operation.
    pub stats: crate::domain::crawl_result::CrawlStats,

    /// Non-fatal errors encountered during crawling.
    pub errors: std::vec::Vec<CrawlPageError>,
}

impl CrawlResult {
    /// Creates a new empty CrawlResult.
    pub fn new() -> Self {
        CrawlResult {
            pages: std::vec::Vec::new(),
            stats: crate::domain::crawl_result::CrawlStats::default(),
            errors: std::vec::Vec::new(),
        }
    }

    /// Returns true if no pages were crawled.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Returns the number of pages crawled.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Returns true if errors occurred during crawling.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns total content bytes across all pages.
    pub fn total_content_bytes(&self) -> usize {
        self.pages.iter().map(|p| p.content.len()).sum()
    }
}

impl std::default::Default for CrawlResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Non-fatal error for a specific page during crawling.
///
/// CrawlPageError captures errors that don't stop the entire crawl but
/// prevent processing of a specific page.
#[derive(Debug, Clone)]
pub struct CrawlPageError {
    /// URL of the page that caused the error.
    pub url: String,

    /// HTTP status code (if applicable).
    pub status_code: std::option::Option<u16>,

    /// Description of the error.
    pub message: String,
}

impl CrawlPageError {
    /// Creates a new CrawlPageError.
    pub fn new(url: String, status_code: std::option::Option<u16>, message: String) -> Self {
        CrawlPageError {
            url,
            status_code,
            message,
        }
    }

    /// Creates an error for a network failure.
    pub fn network_error(url: String, message: String) -> Self {
        CrawlPageError {
            url,
            status_code: std::option::Option::None,
            message,
        }
    }

    /// Creates an error for an HTTP error response.
    pub fn http_error(url: String, status_code: u16) -> Self {
        CrawlPageError {
            url,
            status_code: std::option::Option::Some(status_code),
            message: std::format!("HTTP {}", status_code),
        }
    }
}

impl std::fmt::Display for CrawlPageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status_code {
            std::option::Option::Some(code) => {
                std::write!(f, "{} (HTTP {}): {}", self.url, code, self.message)
            }
            std::option::Option::None => {
                std::write!(f, "{}: {}", self.url, self.message)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawl_result_new_is_empty() {
        // Test: Validates new CrawlResult is empty.
        // Justification: Fresh result should have no pages.
        let result = CrawlResult::new();

        std::assert!(result.is_empty());
        std::assert_eq!(result.page_count(), 0);
        std::assert!(!result.has_errors());
    }

    #[test]
    fn test_crawl_result_default() {
        // Test: Validates Default trait implementation.
        // Justification: Default should match new().
        let result: CrawlResult = std::default::Default::default();

        std::assert!(result.is_empty());
    }

    #[test]
    fn test_crawl_result_total_content_bytes() {
        // Test: Validates content byte counting.
        // Justification: Used for statistics and progress.
        let mut result = CrawlResult::new();
        result.pages.push(crate::domain::crawl_result::CrawledPage {
            url: String::from("https://example.com"),
            title: String::from("Example"),
            content: String::from("Hello World"), // 11 bytes
            links: std::vec::Vec::new(),
            depth: 0,
            status_code: 200,
            content_type: String::from("text/html"),
        });

        std::assert_eq!(result.total_content_bytes(), 11);
    }

    #[test]
    fn test_crawl_page_error_display() {
        // Test: Validates error display format.
        // Justification: Error messages should be readable.
        let err = CrawlPageError::http_error(String::from("https://example.com/404"), 404);

        let msg = std::format!("{}", err);
        std::assert!(msg.contains("https://example.com/404"));
        std::assert!(msg.contains("404"));
    }

    #[test]
    fn test_crawl_page_error_network() {
        // Test: Validates network error creation.
        // Justification: Network errors have no status code.
        let err = CrawlPageError::network_error(
            String::from("https://example.com"),
            String::from("Connection refused"),
        );

        std::assert!(err.status_code.is_none());
        std::assert!(err.message.contains("Connection refused"));
    }
}
