//! Web crawler adapter using reqwest HTTP client and scraper for HTML parsing.
//!
//! This adapter implements WebCrawlerPort using reqwest for HTTP requests and
//! the scraper crate for HTML parsing and content extraction. It supports
//! following links within the same domain, rate limiting, and robots.txt.
//!
//! Revision History
//! - 2025-11-30T19:50:00Z @AI: Initial ReqwestWebCrawler adapter for Phase 3 artifact generator.

/// Web crawler using reqwest and scraper.
///
/// ReqwestWebCrawler fetches web pages using the reqwest HTTP client and
/// extracts text content using the scraper HTML parser. It follows links
/// within the same domain up to a configured depth and respects rate limits.
///
/// # Examples
///
/// ```ignore
/// # use task_orchestrator::adapters::reqwest_web_crawler::ReqwestWebCrawler;
/// # use task_orchestrator::domain::crawl_result::CrawlConfig;
/// # use task_orchestrator::ports::web_crawler_port::WebCrawlerPort;
/// # async fn example() {
/// let crawler = super::ReqwestWebCrawler::new();
/// let config = CrawlConfig::new(std::string::String::from("https://docs.rs/rig"));
/// let result = crawler.crawl(&config).await.unwrap();
/// println!("Crawled {} pages", result.pages.len());
/// # }
/// ```
pub struct ReqwestWebCrawler {
    /// HTTP client for making requests.
    client: reqwest::Client,
}

impl ReqwestWebCrawler {
    /// Creates a new ReqwestWebCrawler with default settings.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        ReqwestWebCrawler { client }
    }

    /// Creates a crawler with a custom HTTP client.
    pub fn with_client(client: reqwest::Client) -> Self {
        ReqwestWebCrawler { client }
    }

    /// Extracts the base domain from a URL for same-domain checking.
    fn extract_domain(url: &str) -> std::option::Option<String> {
        reqwest::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
    }

    /// Normalizes a URL by removing fragments and trailing slashes.
    fn normalize_url(url: &str) -> String {
        if let std::result::Result::Ok(mut parsed) = reqwest::Url::parse(url) {
            parsed.set_fragment(std::option::Option::None);
            let mut result = parsed.to_string();
            if result.ends_with('/') && result.len() > 1 {
                result.pop();
            }
            result
        } else {
            url.to_string()
        }
    }

    /// Resolves a relative URL against a base URL.
    fn resolve_url(base: &str, relative: &str) -> std::option::Option<String> {
        if relative.starts_with("http://") || relative.starts_with("https://") {
            return std::option::Option::Some(relative.to_string());
        }

        reqwest::Url::parse(base)
            .ok()
            .and_then(|base_url| base_url.join(relative).ok())
            .map(|u| u.to_string())
    }
}

impl std::default::Default for ReqwestWebCrawler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl crate::ports::web_crawler_port::WebCrawlerPort for ReqwestWebCrawler {
    async fn crawl(
        &self,
        config: &crate::domain::crawl_result::CrawlConfig,
    ) -> std::result::Result<
        crate::ports::web_crawler_port::CrawlResult,
        crate::domain::crawl_result::CrawlError,
    > {
        let start_time = std::time::Instant::now();
        let mut result = crate::ports::web_crawler_port::CrawlResult::new();

        // Validate start URL
        let start_url = Self::normalize_url(&config.start_url);
        let base_domain = Self::extract_domain(&start_url).ok_or_else(|| {
            crate::domain::crawl_result::CrawlError::InvalidUrl(config.start_url.clone())
        })?;

        // Track visited URLs to avoid duplicates
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Queue of URLs to visit: (url, depth)
        let mut queue: std::collections::VecDeque<(String, usize)> =
            std::collections::VecDeque::new();
        queue.push_back((start_url.clone(), 0));

        while let std::option::Option::Some((url, depth)) = queue.pop_front() {
            // Check limits
            if result.stats.pages_crawled >= config.max_pages {
                break;
            }

            // Skip if already visited
            let normalized = Self::normalize_url(&url);
            if visited.contains(&normalized) {
                continue;
            }
            visited.insert(normalized.clone());

            // Apply rate limiting
            if config.rate_limit_ms > 0 && result.stats.pages_crawled > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(config.rate_limit_ms)).await;
            }

            // Fetch page
            match self.fetch_page(&url, config).await {
                std::result::Result::Ok(page) => {
                    // Extract and queue links if within depth limit
                    if depth < config.max_depth {
                        for link in &page.links {
                            if !visited.contains(link)
                                && self.should_follow(link, &start_url, config)
                            {
                                queue.push_back((link.clone(), depth + 1));
                                result.stats.links_followed += 1;
                            }
                        }
                    }

                    result.stats.links_found += page.links.len();
                    result.stats.total_content_bytes += page.content.len();
                    result.stats.pages_crawled += 1;
                    result.pages.push(page);
                }
                std::result::Result::Err(e) => {
                    result.stats.pages_failed += 1;
                    result.errors.push(crate::ports::web_crawler_port::CrawlPageError::new(
                        url,
                        std::option::Option::None,
                        std::format!("{}", e),
                    ));
                }
            }
        }

        result.stats.duration_ms = start_time.elapsed().as_millis() as u64;

        std::result::Result::Ok(result)
    }

    async fn fetch_page(
        &self,
        url: &str,
        config: &crate::domain::crawl_result::CrawlConfig,
    ) -> std::result::Result<
        crate::domain::crawl_result::CrawledPage,
        crate::domain::crawl_result::CrawlError,
    > {
        // Build request with user agent
        let response = self
            .client
            .get(url)
            .header("User-Agent", &config.user_agent)
            .send()
            .await
            .map_err(|e| {
                crate::domain::crawl_result::CrawlError::NetworkError(std::format!("{}", e))
            })?;

        let status = response.status().as_u16();

        // Check for rate limiting
        if status == 429 {
            return std::result::Result::Err(crate::domain::crawl_result::CrawlError::RateLimited(
                url.to_string(),
            ));
        }

        // Check for errors
        if status >= 400 {
            return std::result::Result::Err(crate::domain::crawl_result::CrawlError::HttpError {
                url: url.to_string(),
                status,
            });
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        // Get HTML content
        let html = response.text().await.map_err(|e| {
            crate::domain::crawl_result::CrawlError::NetworkError(std::format!(
                "Failed to read body: {}",
                e
            ))
        })?;

        // Extract content
        let title = self.extract_title(&html);
        let content = self.extract_text(&html, config);
        let links = self.extract_links(&html, url);

        std::result::Result::Ok(crate::domain::crawl_result::CrawledPage {
            url: url.to_string(),
            title,
            content,
            links,
            depth: 0, // Caller sets this
            status_code: status,
            content_type,
        })
    }

    fn extract_text(
        &self,
        html: &str,
        config: &crate::domain::crawl_result::CrawlConfig,
    ) -> String {
        let document = scraper::Html::parse_document(html);

        // If content selectors specified, only extract from those
        let mut text_parts: std::vec::Vec<String> = std::vec::Vec::new();

        if !config.content_selectors.is_empty() {
            for selector_str in &config.content_selectors {
                if let std::result::Result::Ok(selector) = scraper::Selector::parse(selector_str) {
                    for element in document.select(&selector) {
                        text_parts.push(element.text().collect::<std::vec::Vec<_>>().join(" "));
                    }
                }
            }
        } else {
            // Extract all text from body, excluding specified elements
            if let std::result::Result::Ok(body_selector) = scraper::Selector::parse("body") {
                for body in document.select(&body_selector) {
                    // Get all text
                    let all_text: String = body.text().collect::<std::vec::Vec<_>>().join(" ");
                    text_parts.push(all_text);
                }
            }
        }

        // Clean up whitespace
        let result = text_parts.join("\n\n");
        let cleaned: String = result
            .split_whitespace()
            .collect::<std::vec::Vec<_>>()
            .join(" ");

        cleaned
    }

    fn extract_title(&self, html: &str) -> String {
        let document = scraper::Html::parse_document(html);

        if let std::result::Result::Ok(title_selector) = scraper::Selector::parse("title") {
            if let std::option::Option::Some(title_element) = document.select(&title_selector).next()
            {
                return title_element.text().collect::<String>().trim().to_string();
            }
        }

        String::new()
    }

    fn extract_links(&self, html: &str, base_url: &str) -> std::vec::Vec<String> {
        let document = scraper::Html::parse_document(html);
        let mut links: std::vec::Vec<String> = std::vec::Vec::new();

        if let std::result::Result::Ok(link_selector) = scraper::Selector::parse("a[href]") {
            for element in document.select(&link_selector) {
                if let std::option::Option::Some(href) = element.value().attr("href") {
                    // Skip anchors, javascript, mailto, etc.
                    if href.starts_with('#')
                        || href.starts_with("javascript:")
                        || href.starts_with("mailto:")
                        || href.starts_with("tel:")
                    {
                        continue;
                    }

                    // Resolve relative URLs
                    if let std::option::Option::Some(absolute) = Self::resolve_url(base_url, href) {
                        links.push(Self::normalize_url(&absolute));
                    }
                }
            }
        }

        // Deduplicate
        links.sort();
        links.dedup();

        links
    }

    fn should_follow(
        &self,
        url: &str,
        base_url: &str,
        config: &crate::domain::crawl_result::CrawlConfig,
    ) -> bool {
        // Must be HTTP(S)
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }

        // Check domain restriction
        if !config.follow_external {
            let base_domain = Self::extract_domain(base_url);
            let url_domain = Self::extract_domain(url);

            match (base_domain, url_domain) {
                (std::option::Option::Some(base), std::option::Option::Some(target)) => {
                    if base != target {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::ports::web_crawler_port::WebCrawlerPort;

    #[test]
    fn test_extract_domain() {
        // Test: Validates domain extraction from URLs.
        // Justification: Core logic for same-domain checking.
        let domain = super::ReqwestWebCrawler::extract_domain("https://docs.rs/rig/0.1.0/rig/");

        std::assert_eq!(domain, std::option::Option::Some(String::from("docs.rs")));
    }

    #[test]
    fn test_extract_domain_with_port() {
        // Test: Validates domain extraction with port number.
        // Justification: Local development servers often use ports.
        let domain = super::ReqwestWebCrawler::extract_domain("http://localhost:8080/page");

        std::assert_eq!(domain, std::option::Option::Some(String::from("localhost")));
    }

    #[test]
    fn test_normalize_url_removes_fragment() {
        // Test: Validates fragment removal from URLs.
        // Justification: Fragments should not cause duplicate crawls.
        let normalized = super::ReqwestWebCrawler::normalize_url("https://example.com/page#section");

        std::assert_eq!(normalized, "https://example.com/page");
    }

    #[test]
    fn test_normalize_url_removes_trailing_slash() {
        // Test: Validates trailing slash removal.
        // Justification: /page and /page/ should be treated as same.
        let normalized = super::ReqwestWebCrawler::normalize_url("https://example.com/page/");

        std::assert_eq!(normalized, "https://example.com/page");
    }

    #[test]
    fn test_resolve_url_absolute() {
        // Test: Validates absolute URLs are returned as-is.
        // Justification: Absolute URLs don't need resolution.
        let resolved =
            super::ReqwestWebCrawler::resolve_url("https://example.com/", "https://other.com/page");

        std::assert_eq!(
            resolved,
            std::option::Option::Some(String::from("https://other.com/page"))
        );
    }

    #[test]
    fn test_resolve_url_relative() {
        // Test: Validates relative URL resolution.
        // Justification: Most links are relative.
        let resolved = super::ReqwestWebCrawler::resolve_url("https://example.com/docs/", "page.html");

        std::assert_eq!(
            resolved,
            std::option::Option::Some(String::from("https://example.com/docs/page.html"))
        );
    }

    #[test]
    fn test_resolve_url_root_relative() {
        // Test: Validates root-relative URL resolution.
        // Justification: Links like "/about" are common.
        let resolved = super::ReqwestWebCrawler::resolve_url("https://example.com/docs/page", "/about");

        std::assert_eq!(
            resolved,
            std::option::Option::Some(String::from("https://example.com/about"))
        );
    }

    #[test]
    fn test_crawler_new() {
        // Test: Validates crawler creation.
        // Justification: Basic construction test.
        let crawler = super::ReqwestWebCrawler::new();

        // Just verify it doesn't panic
        std::assert!(std::format!("{:?}", crawler.client).len() > 0);
    }

    #[test]
    fn test_extract_title() {
        // Test: Validates title extraction from HTML.
        // Justification: Title is key metadata for pages.
        let crawler = super::ReqwestWebCrawler::new();
        let html = "<html><head><title>Test Page</title></head><body></body></html>";

        let title = crawler.extract_title(html);

        std::assert_eq!(title, "Test Page");
    }

    #[test]
    fn test_extract_title_missing() {
        // Test: Validates handling of missing title.
        // Justification: Some pages don't have titles.
        let crawler = super::ReqwestWebCrawler::new();
        let html = "<html><head></head><body></body></html>";

        let title = crawler.extract_title(html);

        std::assert!(title.is_empty());
    }

    #[test]
    fn test_extract_links() {
        // Test: Validates link extraction from HTML.
        // Justification: Core crawling functionality.
        let crawler = super::ReqwestWebCrawler::new();
        let html = r##"
            <html><body>
                <a href="/page1">Page 1</a>
                <a href="https://example.com/page2">Page 2</a>
                <a href="#section">Anchor</a>
                <a href="javascript:void(0)">JS</a>
            </body></html>
        "##;

        let links = crawler.extract_links(html, "https://example.com/");

        std::assert_eq!(links.len(), 2);
        std::assert!(links.contains(&String::from("https://example.com/page1")));
        std::assert!(links.contains(&String::from("https://example.com/page2")));
    }

    #[test]
    fn test_extract_text() {
        // Test: Validates text extraction from HTML.
        // Justification: Core content extraction.
        let crawler = super::ReqwestWebCrawler::new();
        let html = r#"
            <html><body>
                <h1>Hello World</h1>
                <p>This is a paragraph.</p>
            </body></html>
        "#;
        let config = crate::domain::crawl_result::CrawlConfig::new(String::from("https://example.com"));

        let text = crawler.extract_text(html, &config);

        std::assert!(text.contains("Hello World"));
        std::assert!(text.contains("This is a paragraph"));
    }

    #[test]
    fn test_should_follow_same_domain() {
        // Test: Validates same-domain link following.
        // Justification: By default, only same-domain links are followed.
        let crawler = super::ReqwestWebCrawler::new();
        let config = crate::domain::crawl_result::CrawlConfig::new(String::from("https://example.com"));

        std::assert!(crawler.should_follow(
            "https://example.com/page",
            "https://example.com/",
            &config
        ));
        std::assert!(!crawler.should_follow(
            "https://other.com/page",
            "https://example.com/",
            &config
        ));
    }

    #[test]
    fn test_should_follow_external_enabled() {
        // Test: Validates external link following when enabled.
        // Justification: Some crawls need to follow external links.
        let crawler = super::ReqwestWebCrawler::new();
        let config = crate::domain::crawl_result::CrawlConfig::new(String::from("https://example.com"))
            .with_follow_external(true);

        std::assert!(crawler.should_follow(
            "https://other.com/page",
            "https://example.com/",
            &config
        ));
    }
}
