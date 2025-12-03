//! Defines configuration and result types for web crawling operations.
//!
//! This module contains value objects for configuring web crawlers and
//! representing crawled page content. The CrawlConfig controls crawl behavior
//! while CrawledPage represents fetched pages ready for artifact generation.
//!
//! Revision History
//! - 2025-11-30T18:45:00Z @AI: Initial crawl_result module for Phase 1 artifact generator.

/// Configuration for web crawling operations.
///
/// CrawlConfig controls all aspects of a web crawl including depth limits,
/// rate limiting, and domain restrictions.
///
/// # Fields
///
/// * `start_url` - The URL to begin crawling from.
/// * `max_depth` - Maximum link depth to follow (0 = start page only).
/// * `max_pages` - Maximum number of pages to crawl.
/// * `follow_external` - Whether to follow links to external domains.
/// * `rate_limit_ms` - Delay between requests in milliseconds.
/// * `respect_robots_txt` - Whether to honor robots.txt directives.
/// * `user_agent` - User-Agent header to send with requests.
///
/// # Examples
///
/// ```
/// # use task_orchestrator::domain::crawl_result::CrawlConfig;
/// let config = CrawlConfig::new(std::string::String::from("https://docs.example.com"));
///
/// std::assert_eq!(config.max_depth, 2);
/// std::assert!(!config.follow_external);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CrawlConfig {
    /// The URL to begin crawling from.
    pub start_url: String,

    /// Maximum link depth to follow. 0 = start page only, 1 = start + linked pages, etc.
    pub max_depth: usize,

    /// Maximum number of pages to crawl (prevents runaway crawls).
    pub max_pages: usize,

    /// Whether to follow links to external domains.
    pub follow_external: bool,

    /// Delay between requests in milliseconds (rate limiting).
    pub rate_limit_ms: u64,

    /// Whether to honor robots.txt directives.
    pub respect_robots_txt: bool,

    /// User-Agent header to send with requests.
    pub user_agent: String,

    /// CSS selectors for content extraction (empty = extract all text).
    pub content_selectors: std::vec::Vec<String>,

    /// CSS selectors for elements to exclude from content.
    pub exclude_selectors: std::vec::Vec<String>,
}

impl CrawlConfig {
    /// Default User-Agent string.
    const DEFAULT_USER_AGENT: &'static str = "RiggerBot/1.0 (Artifact Generator)";

    /// Default rate limit between requests.
    const DEFAULT_RATE_LIMIT_MS: u64 = 1000;

    /// Default maximum pages to crawl.
    const DEFAULT_MAX_PAGES: usize = 100;

    /// Creates a new CrawlConfig with default settings for the given URL.
    ///
    /// # Arguments
    ///
    /// * `start_url` - The URL to begin crawling from.
    ///
    /// # Returns
    ///
    /// A CrawlConfig with sensible defaults:
    /// - Depth 2 (start page + 2 levels of links)
    /// - 100 max pages
    /// - 1 second between requests
    /// - No external domains
    /// - Respects robots.txt
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_orchestrator::domain::crawl_result::CrawlConfig;
    /// let config = CrawlConfig::new(std::string::String::from("https://docs.rs/rig"));
    /// std::assert_eq!(config.max_pages, 100);
    /// ```
    pub fn new(start_url: String) -> Self {
        CrawlConfig {
            start_url,
            max_depth: 2,
            max_pages: Self::DEFAULT_MAX_PAGES,
            follow_external: false,
            rate_limit_ms: Self::DEFAULT_RATE_LIMIT_MS,
            respect_robots_txt: true,
            user_agent: String::from(Self::DEFAULT_USER_AGENT),
            content_selectors: std::vec::Vec::new(),
            exclude_selectors: Self::default_exclude_selectors(),
        }
    }

    /// Creates a config for shallow crawling (single page only).
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch.
    ///
    /// # Returns
    ///
    /// A CrawlConfig that only fetches the specified page.
    pub fn single_page(url: String) -> Self {
        CrawlConfig {
            start_url: url,
            max_depth: 0,
            max_pages: 1,
            follow_external: false,
            rate_limit_ms: 0,
            respect_robots_txt: true,
            user_agent: String::from(Self::DEFAULT_USER_AGENT),
            content_selectors: std::vec::Vec::new(),
            exclude_selectors: Self::default_exclude_selectors(),
        }
    }

    /// Returns default selectors for elements to exclude.
    ///
    /// These typically contain navigation, ads, or other non-content elements.
    fn default_exclude_selectors() -> std::vec::Vec<String> {
        std::vec![
            String::from("nav"),
            String::from("header"),
            String::from("footer"),
            String::from(".sidebar"),
            String::from(".navigation"),
            String::from(".menu"),
            String::from(".ads"),
            String::from(".advertisement"),
            String::from("script"),
            String::from("style"),
            String::from("noscript"),
        ]
    }

    /// Sets the maximum crawl depth.
    ///
    /// # Arguments
    ///
    /// * `depth` - Maximum depth (0 = single page).
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Sets the maximum number of pages to crawl.
    ///
    /// # Arguments
    ///
    /// * `pages` - Maximum page count.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_max_pages(mut self, pages: usize) -> Self {
        self.max_pages = pages;
        self
    }

    /// Enables or disables following external links.
    ///
    /// # Arguments
    ///
    /// * `follow` - Whether to follow external links.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_follow_external(mut self, follow: bool) -> Self {
        self.follow_external = follow;
        self
    }

    /// Sets the rate limit between requests.
    ///
    /// # Arguments
    ///
    /// * `ms` - Delay in milliseconds.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_rate_limit(mut self, ms: u64) -> Self {
        self.rate_limit_ms = ms;
        self
    }

    /// Adds a CSS selector for content extraction.
    ///
    /// # Arguments
    ///
    /// * `selector` - CSS selector string.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_content_selector(mut self, selector: String) -> Self {
        self.content_selectors.push(selector);
        self
    }
}

/// Represents a page fetched during web crawling.
///
/// CrawledPage contains the URL, extracted content, and metadata for a
/// successfully fetched web page. The content is cleaned text ready for
/// chunking and embedding.
///
/// # Fields
///
/// * `url` - The URL that was fetched.
/// * `title` - Page title extracted from <title> tag.
/// * `content` - Cleaned text content (HTML tags stripped).
/// * `links` - URLs of links found on this page.
/// * `depth` - How many links deep from the start URL (0 = start page).
/// * `status_code` - HTTP status code of the response.
/// * `content_type` - Content-Type header value.
///
/// # Examples
///
/// ```
/// # use task_orchestrator::domain::crawl_result::CrawledPage;
/// let page = CrawledPage {
///     url: std::string::String::from("https://example.com"),
///     title: std::string::String::from("Example Domain"),
///     content: std::string::String::from("This domain is for use in examples."),
///     links: std::vec![std::string::String::from("https://www.iana.org/domains/example")],
///     depth: 0,
///     status_code: 200,
///     content_type: std::string::String::from("text/html"),
/// };
///
/// std::assert_eq!(page.status_code, 200);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CrawledPage {
    /// The URL that was fetched.
    pub url: String,

    /// Page title extracted from <title> tag.
    pub title: String,

    /// Cleaned text content (HTML tags stripped).
    pub content: String,

    /// URLs of links found on this page.
    pub links: std::vec::Vec<String>,

    /// How many links deep from the start URL (0 = start page).
    pub depth: usize,

    /// HTTP status code of the response.
    pub status_code: u16,

    /// Content-Type header value.
    pub content_type: String,
}

impl CrawledPage {
    /// Returns true if this page was successfully fetched (2xx status).
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// Returns the content length in characters.
    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    /// Returns true if this page has meaningful content.
    ///
    /// A page is considered empty if it has less than 50 characters of content.
    pub fn has_content(&self) -> bool {
        self.content.len() >= 50
    }
}

/// Errors that can occur during web crawling.
///
/// CrawlError categorizes failures that may occur when fetching pages,
/// parsing HTML, or following links.
///
/// # Variants
///
/// * `InvalidUrl` - The URL is malformed or unsupported.
/// * `NetworkError` - Failed to connect or fetch the URL.
/// * `HttpError` - Server returned an error status code.
/// * `ParseError` - Failed to parse HTML content.
/// * `RateLimited` - Server returned 429 Too Many Requests.
/// * `RobotsTxtBlocked` - URL is disallowed by robots.txt.
/// * `Timeout` - Request timed out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrawlError {
    /// The URL is malformed or uses an unsupported scheme.
    InvalidUrl(String),

    /// Failed to connect to the server or fetch the page.
    NetworkError(String),

    /// Server returned an error status code.
    HttpError { url: String, status: u16 },

    /// Failed to parse the HTML content.
    ParseError(String),

    /// Server returned 429 Too Many Requests.
    RateLimited(String),

    /// URL is disallowed by robots.txt.
    RobotsTxtBlocked(String),

    /// Request timed out.
    Timeout(String),
}

impl std::fmt::Display for CrawlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrawlError::InvalidUrl(url) => {
                std::write!(f, "Invalid URL: {}", url)
            }
            CrawlError::NetworkError(msg) => {
                std::write!(f, "Network error: {}", msg)
            }
            CrawlError::HttpError { url, status } => {
                std::write!(f, "HTTP {} for {}", status, url)
            }
            CrawlError::ParseError(msg) => {
                std::write!(f, "Parse error: {}", msg)
            }
            CrawlError::RateLimited(url) => {
                std::write!(f, "Rate limited: {}", url)
            }
            CrawlError::RobotsTxtBlocked(url) => {
                std::write!(f, "Blocked by robots.txt: {}", url)
            }
            CrawlError::Timeout(url) => {
                std::write!(f, "Request timed out: {}", url)
            }
        }
    }
}

impl std::error::Error for CrawlError {}

/// Statistics from a completed web crawl.
///
/// CrawlStats provides metrics about a crawl operation including page counts,
/// success rates, and timing information.
///
/// # Fields
///
/// * `pages_crawled` - Number of pages successfully fetched.
/// * `pages_failed` - Number of pages that failed to fetch.
/// * `links_found` - Total number of links discovered.
/// * `links_followed` - Number of links that were actually followed.
/// * `total_content_bytes` - Total bytes of content extracted.
/// * `duration_ms` - Time taken for the crawl in milliseconds.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct CrawlStats {
    /// Number of pages successfully fetched.
    pub pages_crawled: usize,

    /// Number of pages that failed to fetch.
    pub pages_failed: usize,

    /// Total number of links discovered on crawled pages.
    pub links_found: usize,

    /// Number of links that were actually followed.
    pub links_followed: usize,

    /// Total bytes of extracted text content.
    pub total_content_bytes: usize,

    /// Time taken for the crawl in milliseconds.
    pub duration_ms: u64,
}

impl CrawlStats {
    /// Returns the success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        let total = self.pages_crawled + self.pages_failed;
        if total == 0 {
            0.0
        } else {
            (self.pages_crawled as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawl_config_new_defaults() {
        // Test: Validates new config has sensible defaults.
        // Justification: Users expect reasonable defaults for quick starts.
        let config = CrawlConfig::new(String::from("https://example.com"));

        std::assert_eq!(config.max_depth, 2);
        std::assert_eq!(config.max_pages, 100);
        std::assert!(!config.follow_external);
        std::assert!(config.respect_robots_txt);
        std::assert_eq!(config.rate_limit_ms, 1000);
    }

    #[test]
    fn test_crawl_config_single_page() {
        // Test: Validates single_page preset is minimal.
        // Justification: Single page crawl should not follow links.
        let config = CrawlConfig::single_page(String::from("https://example.com/page"));

        std::assert_eq!(config.max_depth, 0);
        std::assert_eq!(config.max_pages, 1);
    }

    #[test]
    fn test_crawl_config_builder_pattern() {
        // Test: Validates builder methods work correctly.
        // Justification: Fluent API should allow method chaining.
        let config = CrawlConfig::new(String::from("https://example.com"))
            .with_max_depth(5)
            .with_max_pages(50)
            .with_follow_external(true)
            .with_rate_limit(500);

        std::assert_eq!(config.max_depth, 5);
        std::assert_eq!(config.max_pages, 50);
        std::assert!(config.follow_external);
        std::assert_eq!(config.rate_limit_ms, 500);
    }

    #[test]
    fn test_crawled_page_is_success() {
        // Test: Validates success detection for 2xx status codes.
        // Justification: Core logic for determining if page was fetched.
        let success_page = CrawledPage {
            url: String::from("https://example.com"),
            title: String::from("Example"),
            content: String::from("Content here"),
            links: std::vec::Vec::new(),
            depth: 0,
            status_code: 200,
            content_type: String::from("text/html"),
        };

        let redirect_page = CrawledPage {
            status_code: 301,
            ..success_page.clone()
        };

        let error_page = CrawledPage {
            status_code: 404,
            ..success_page.clone()
        };

        std::assert!(success_page.is_success());
        std::assert!(!redirect_page.is_success());
        std::assert!(!error_page.is_success());
    }

    #[test]
    fn test_crawled_page_has_content() {
        // Test: Validates content detection threshold.
        // Justification: Pages with minimal content should be flagged.
        let has_content = CrawledPage {
            url: String::from("https://example.com"),
            title: String::from("Example"),
            content: String::from("This is a page with enough content to be meaningful for processing."),
            links: std::vec::Vec::new(),
            depth: 0,
            status_code: 200,
            content_type: String::from("text/html"),
        };

        let no_content = CrawledPage {
            content: String::from("Short"),
            ..has_content.clone()
        };

        std::assert!(has_content.has_content());
        std::assert!(!no_content.has_content());
    }

    #[test]
    fn test_crawl_error_display() {
        // Test: Validates error messages are user-friendly.
        // Justification: Error messages should clearly explain the problem.
        let err = CrawlError::HttpError {
            url: String::from("https://example.com"),
            status: 404,
        };
        let msg = std::format!("{}", err);

        std::assert!(msg.contains("404"));
        std::assert!(msg.contains("example.com"));
    }

    #[test]
    fn test_crawl_stats_success_rate() {
        // Test: Validates success rate calculation.
        // Justification: Metrics should be accurately computed.
        let stats = CrawlStats {
            pages_crawled: 80,
            pages_failed: 20,
            links_found: 500,
            links_followed: 100,
            total_content_bytes: 1000000,
            duration_ms: 60000,
        };

        std::assert!((stats.success_rate() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_crawl_stats_success_rate_zero() {
        // Test: Validates success rate with no pages.
        // Justification: Division by zero must be handled.
        let stats: CrawlStats = std::default::Default::default();

        std::assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_crawl_config_default_exclude_selectors() {
        // Test: Validates default exclusion selectors are set.
        // Justification: Navigation and ads should be excluded by default.
        let config = CrawlConfig::new(String::from("https://example.com"));

        std::assert!(!config.exclude_selectors.is_empty());
        std::assert!(config.exclude_selectors.contains(&String::from("nav")));
        std::assert!(config.exclude_selectors.contains(&String::from("script")));
    }
}
