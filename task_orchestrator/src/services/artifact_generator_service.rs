//! Artifact generator service for scanning directories and crawling websites.
//!
//! ArtifactGeneratorService orchestrates the full pipeline of scanning source
//! locations (directories or URLs), chunking content, generating embeddings,
//! and persisting artifacts to the knowledge base for RAG retrieval.
//!
//! This service enables pre-populating the artifact database with context
//! from codebases, documentation sites, and other sources before task generation.
//!
//! Revision History
//! - 2025-11-30T21:00:00Z @AI: Create ArtifactGeneratorService for Phase 4 artifact generator.

/// Report of artifact generation results.
///
/// GenerationReport captures statistics and results from a directory scan or
/// web crawl operation, enabling progress tracking and error reporting.
///
/// # Examples
///
/// ```
/// # use task_orchestrator::services::artifact_generator_service::GenerationReport;
/// let report = GenerationReport::new();
/// std::assert_eq!(report.files_scanned, 0);
/// std::assert_eq!(report.artifacts_created, 0);
/// std::assert!(!report.has_errors());
/// ```
#[derive(Debug, Clone)]
pub struct GenerationReport {
    /// Number of files scanned (directory mode).
    pub files_scanned: usize,

    /// Number of web pages crawled (URL mode).
    pub pages_crawled: usize,

    /// Number of artifacts successfully created and persisted.
    pub artifacts_created: usize,

    /// Number of chunks generated from content.
    pub chunks_generated: usize,

    /// Total bytes of content processed.
    pub bytes_processed: usize,

    /// Non-fatal errors encountered during generation.
    pub errors: std::vec::Vec<String>,

    /// Duration of the generation operation in milliseconds.
    pub duration_ms: u64,
}

impl GenerationReport {
    /// Creates a new empty GenerationReport.
    pub fn new() -> Self {
        GenerationReport {
            files_scanned: 0,
            pages_crawled: 0,
            artifacts_created: 0,
            chunks_generated: 0,
            bytes_processed: 0,
            errors: std::vec::Vec::new(),
            duration_ms: 0,
        }
    }

    /// Returns true if any errors occurred during generation.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the number of errors.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Adds an error to the report.
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
}

impl std::default::Default for GenerationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for artifact generation operations.
///
/// GenerationConfig specifies options for how content should be processed,
/// chunked, and stored during artifact generation.
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    /// Project ID to associate artifacts with.
    pub project_id: String,

    /// Chunking strategy to use.
    pub chunk_strategy: task_manager::domain::scan_config::ChunkStrategy,

    /// Maximum chunk size in characters (for FixedSize strategy).
    pub max_chunk_size: usize,

    /// Whether to skip files that already have artifacts (incremental mode).
    pub incremental: bool,
}

impl GenerationConfig {
    /// Creates a new GenerationConfig for the given project.
    pub fn new(project_id: String) -> Self {
        GenerationConfig {
            project_id,
            chunk_strategy: task_manager::domain::scan_config::ChunkStrategy::Paragraph,
            max_chunk_size: 1000,
            incremental: false,
        }
    }

    /// Sets the chunking strategy.
    pub fn with_chunk_strategy(mut self, strategy: task_manager::domain::scan_config::ChunkStrategy) -> Self {
        self.chunk_strategy = strategy;
        self
    }

    /// Sets the maximum chunk size.
    pub fn with_max_chunk_size(mut self, size: usize) -> Self {
        self.max_chunk_size = size;
        self
    }

    /// Enables incremental mode (skip existing artifacts).
    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }
}

/// Service for generating artifacts from directories and websites.
///
/// ArtifactGeneratorService coordinates:
/// 1. Directory scanning via DirectoryScannerPort
/// 2. Web crawling via WebCrawlerPort
/// 3. Content chunking with configurable strategies
/// 4. Embedding generation via EmbeddingPort
/// 5. Artifact persistence via ArtifactRepositoryPort
///
/// # Type Parameters
///
/// This service uses trait objects to allow dependency injection of different
/// implementations for scanning, crawling, embedding, and storage.
pub struct ArtifactGeneratorService {
    directory_scanner: std::sync::Arc<dyn task_manager::ports::directory_scanner_port::DirectoryScannerPort + std::marker::Send + std::marker::Sync>,
    web_crawler: std::sync::Arc<dyn crate::ports::web_crawler_port::WebCrawlerPort + std::marker::Send + std::marker::Sync>,
    embedding_port: std::sync::Arc<dyn crate::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>,
    artifact_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
}

impl ArtifactGeneratorService {
    /// Creates a new ArtifactGeneratorService with the given dependencies.
    ///
    /// # Arguments
    ///
    /// * `directory_scanner` - Port for scanning directories
    /// * `web_crawler` - Port for crawling websites
    /// * `embedding_port` - Port for generating embeddings
    /// * `artifact_repository` - Repository for persisting artifacts
    pub fn new(
        directory_scanner: std::sync::Arc<dyn task_manager::ports::directory_scanner_port::DirectoryScannerPort + std::marker::Send + std::marker::Sync>,
        web_crawler: std::sync::Arc<dyn crate::ports::web_crawler_port::WebCrawlerPort + std::marker::Send + std::marker::Sync>,
        embedding_port: std::sync::Arc<dyn crate::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>,
        artifact_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
    ) -> Self {
        ArtifactGeneratorService {
            directory_scanner,
            web_crawler,
            embedding_port,
            artifact_repository,
        }
    }

    /// Generates artifacts from a directory by scanning files.
    ///
    /// Scans the directory at `path` respecting .gitignore patterns, chunks
    /// file contents, generates embeddings, and persists artifacts.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to scan
    /// * `config` - Generation configuration options
    /// * `scan_config` - Directory scan configuration
    ///
    /// # Returns
    ///
    /// Returns a GenerationReport with statistics and any errors.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be scanned.
    pub async fn generate_from_directory(
        &self,
        path: &str,
        config: &GenerationConfig,
        scan_config: &task_manager::domain::scan_config::ScanConfig,
    ) -> std::result::Result<GenerationReport, String> {
        let start_time = std::time::Instant::now();
        let mut report = GenerationReport::new();

        // 1. Scan the directory
        let scan_result = self.directory_scanner
            .scan(scan_config)
            .await
            .map_err(|e| std::format!("Directory scan failed: {:?}", e))?;

        report.files_scanned = scan_result.files.len();

        // Collect non-fatal scan errors
        for error in &scan_result.errors {
            report.add_error(std::format!("Scan error: {} - {}", error.path, error.message));
        }

        // 2. Process each file
        for file in scan_result.files {
            match self.process_file(&file, config).await {
                std::result::Result::Ok(artifacts_created) => {
                    report.artifacts_created += artifacts_created;
                    report.bytes_processed += file.size_bytes;
                }
                std::result::Result::Err(e) => {
                    report.add_error(std::format!("File processing failed for {}: {}", file.path, e));
                }
            }
        }

        report.duration_ms = start_time.elapsed().as_millis() as u64;
        std::result::Result::Ok(report)
    }

    /// Generates artifacts from a website by crawling pages.
    ///
    /// Crawls the website starting at `url`, extracts text content, chunks it,
    /// generates embeddings, and persists artifacts.
    ///
    /// # Arguments
    ///
    /// * `url` - Starting URL to crawl
    /// * `config` - Generation configuration options
    /// * `crawl_config` - Web crawl configuration
    ///
    /// # Returns
    ///
    /// Returns a GenerationReport with statistics and any errors.
    ///
    /// # Errors
    ///
    /// Returns an error if the website cannot be crawled.
    pub async fn generate_from_url(
        &self,
        url: &str,
        config: &GenerationConfig,
        crawl_config: &crate::domain::crawl_result::CrawlConfig,
    ) -> std::result::Result<GenerationReport, String> {
        let start_time = std::time::Instant::now();
        let mut report = GenerationReport::new();

        // 1. Crawl the website
        let crawl_result = self.web_crawler
            .crawl(crawl_config)
            .await
            .map_err(|e| std::format!("Web crawl failed: {:?}", e))?;

        report.pages_crawled = crawl_result.pages.len();

        // Collect non-fatal crawl errors
        for error in &crawl_result.errors {
            report.add_error(std::format!("Crawl error: {} - {}", error.url, error.message));
        }

        // 2. Process each page
        for page in crawl_result.pages {
            match self.process_page(&page, config).await {
                std::result::Result::Ok(artifacts_created) => {
                    report.artifacts_created += artifacts_created;
                    report.bytes_processed += page.content.len();
                }
                std::result::Result::Err(e) => {
                    report.add_error(std::format!("Page processing failed for {}: {}", page.url, e));
                }
            }
        }

        report.duration_ms = start_time.elapsed().as_millis() as u64;
        std::result::Result::Ok(report)
    }

    /// Processes a single file into artifacts.
    async fn process_file(
        &self,
        file: &task_manager::domain::scan_config::ScannedFile,
        config: &GenerationConfig,
    ) -> std::result::Result<usize, String> {
        if file.content.is_empty() {
            return std::result::Result::Ok(0);
        }

        // Chunk the content
        let chunks = self.chunk_content(&file.content, &config.chunk_strategy, config.max_chunk_size);
        if chunks.is_empty() {
            return std::result::Result::Ok(0);
        }

        // Generate embeddings for all chunks
        let chunk_refs: std::vec::Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let embeddings = self.embedding_port
            .generate_embeddings(&chunk_refs)
            .await
            .map_err(|e| std::format!("Embedding generation failed: {}", e))?;

        if embeddings.len() != chunks.len() {
            return std::result::Result::Err(std::format!(
                "Embedding count mismatch: expected {}, got {}",
                chunks.len(),
                embeddings.len()
            ));
        }

        // Determine artifact type from extension
        let artifact_type = Self::artifact_type_from_extension(&file.extension);

        // Create and persist artifacts
        let mut artifacts_created = 0;
        let mut repo = self.artifact_repository.lock()
            .map_err(|e| std::format!("Failed to acquire repository lock: {}", e))?;

        for (i, (chunk, embedding)) in chunks.into_iter().zip(embeddings.into_iter()).enumerate() {
            let artifact = task_manager::domain::artifact::Artifact {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: config.project_id.clone(),
                source_id: file.path.clone(),
                source_type: artifact_type.clone(),
                content: chunk,
                embedding,
                metadata: std::option::Option::Some(std::format!(
                    "{{\"chunk_index\": {}, \"line_count\": {}, \"file_size\": {}}}",
                    i, file.line_count, file.size_bytes
                )),
                created_at: chrono::Utc::now(),
                binary_content: std::option::Option::None,
                mime_type: std::option::Option::None,
                source_url: std::option::Option::None,
                page_number: std::option::Option::None,
            };

            repo.save(artifact)
                .map_err(|e| std::format!("Failed to save artifact: {}", e))?;

            artifacts_created += 1;
        }

        std::result::Result::Ok(artifacts_created)
    }

    /// Processes a single web page into artifacts.
    async fn process_page(
        &self,
        page: &crate::domain::crawl_result::CrawledPage,
        config: &GenerationConfig,
    ) -> std::result::Result<usize, String> {
        if page.content.is_empty() {
            return std::result::Result::Ok(0);
        }

        // Chunk the content
        let chunks = self.chunk_content(&page.content, &config.chunk_strategy, config.max_chunk_size);
        if chunks.is_empty() {
            return std::result::Result::Ok(0);
        }

        // Generate embeddings for all chunks
        let chunk_refs: std::vec::Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let embeddings = self.embedding_port
            .generate_embeddings(&chunk_refs)
            .await
            .map_err(|e| std::format!("Embedding generation failed: {}", e))?;

        if embeddings.len() != chunks.len() {
            return std::result::Result::Err(std::format!(
                "Embedding count mismatch: expected {}, got {}",
                chunks.len(),
                embeddings.len()
            ));
        }

        // Create and persist artifacts
        let mut artifacts_created = 0;
        let mut repo = self.artifact_repository.lock()
            .map_err(|e| std::format!("Failed to acquire repository lock: {}", e))?;

        for (i, (chunk, embedding)) in chunks.into_iter().zip(embeddings.into_iter()).enumerate() {
            let artifact = task_manager::domain::artifact::Artifact {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: config.project_id.clone(),
                source_id: page.url.clone(),
                source_type: task_manager::domain::artifact::ArtifactType::WebResearch,
                content: chunk,
                embedding,
                metadata: std::option::Option::Some(std::format!(
                    "{{\"chunk_index\": {}, \"page_title\": \"{}\", \"depth\": {}}}",
                    i,
                    page.title.replace('"', "\\\""),
                    page.depth
                )),
                created_at: chrono::Utc::now(),
                binary_content: std::option::Option::None,
                mime_type: std::option::Option::None,
                source_url: std::option::Option::Some(page.url.clone()),
                page_number: std::option::Option::None,
            };

            repo.save(artifact)
                .map_err(|e| std::format!("Failed to save artifact: {}", e))?;

            artifacts_created += 1;
        }

        std::result::Result::Ok(artifacts_created)
    }

    /// Chunks content according to the configured strategy.
    fn chunk_content(
        &self,
        content: &str,
        strategy: &task_manager::domain::scan_config::ChunkStrategy,
        max_size: usize,
    ) -> std::vec::Vec<String> {
        match strategy {
            task_manager::domain::scan_config::ChunkStrategy::Paragraph => {
                Self::chunk_by_paragraph(content)
            }
            task_manager::domain::scan_config::ChunkStrategy::Sentence => {
                Self::chunk_by_sentence(content)
            }
            task_manager::domain::scan_config::ChunkStrategy::FixedSize(size) => {
                Self::chunk_by_size(content, *size)
            }
            task_manager::domain::scan_config::ChunkStrategy::WholeFile => {
                if content.is_empty() {
                    std::vec::Vec::new()
                } else {
                    std::vec![content.to_string()]
                }
            }
        }
    }

    /// Chunks text by paragraph (double newlines).
    fn chunk_by_paragraph(text: &str) -> std::vec::Vec<String> {
        text.split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Chunks text by sentence (periods followed by space/newline).
    fn chunk_by_sentence(text: &str) -> std::vec::Vec<String> {
        let mut chunks = std::vec::Vec::new();
        let mut current = String::new();

        for c in text.chars() {
            current.push(c);
            if c == '.' || c == '!' || c == '?' {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    chunks.push(trimmed);
                }
                current.clear();
            }
        }

        // Don't forget remaining content
        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() {
            chunks.push(trimmed);
        }

        chunks
    }

    /// Chunks text by fixed character size.
    fn chunk_by_size(text: &str, size: usize) -> std::vec::Vec<String> {
        if size == 0 {
            return std::vec::Vec::new();
        }

        let mut chunks = std::vec::Vec::new();
        let chars: std::vec::Vec<char> = text.chars().collect();

        for chunk_chars in chars.chunks(size) {
            let chunk: String = chunk_chars.iter().collect();
            let trimmed = chunk.trim().to_string();
            if !trimmed.is_empty() {
                chunks.push(trimmed);
            }
        }

        chunks
    }

    /// Determines artifact type from file extension.
    fn artifact_type_from_extension(extension: &str) -> task_manager::domain::artifact::ArtifactType {
        match extension.to_lowercase().as_str() {
            "md" | "txt" | "rst" | "adoc" => task_manager::domain::artifact::ArtifactType::PRD,
            "pdf" => task_manager::domain::artifact::ArtifactType::PDF,
            "png" | "jpg" | "jpeg" | "gif" | "webp" => task_manager::domain::artifact::ArtifactType::Image,
            _ => task_manager::domain::artifact::ArtifactType::File,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock directory scanner for testing.
    struct MockDirectoryScanner {
        files: std::vec::Vec<task_manager::domain::scan_config::ScannedFile>,
    }

    #[async_trait::async_trait]
    impl task_manager::ports::directory_scanner_port::DirectoryScannerPort for MockDirectoryScanner {
        async fn scan(
            &self,
            _config: &task_manager::domain::scan_config::ScanConfig,
        ) -> std::result::Result<task_manager::ports::directory_scanner_port::ScanResult, task_manager::domain::scan_config::ScanError> {
            let mut result = task_manager::ports::directory_scanner_port::ScanResult::new();
            result.files = self.files.clone();
            std::result::Result::Ok(result)
        }

        async fn read_file(
            &self,
            _path: &std::path::Path,
            _config: &task_manager::domain::scan_config::ScanConfig,
        ) -> std::result::Result<std::option::Option<task_manager::domain::scan_config::ScannedFile>, task_manager::domain::scan_config::ScanError> {
            std::result::Result::Ok(std::option::Option::None)
        }

        async fn has_file_changed(
            &self,
            _path: &std::path::Path,
            _previous: &task_manager::domain::scan_config::FileFingerprint,
        ) -> std::result::Result<bool, task_manager::domain::scan_config::ScanError> {
            std::result::Result::Ok(false)
        }

        async fn find_deleted_files(
            &self,
            _config: &task_manager::domain::scan_config::ScanConfig,
            _previous_paths: &[String],
        ) -> std::result::Result<std::vec::Vec<String>, task_manager::domain::scan_config::ScanError> {
            std::result::Result::Ok(std::vec::Vec::new())
        }
    }

    /// Mock web crawler for testing.
    struct MockWebCrawler {
        pages: std::vec::Vec<crate::domain::crawl_result::CrawledPage>,
    }

    #[async_trait::async_trait]
    impl crate::ports::web_crawler_port::WebCrawlerPort for MockWebCrawler {
        async fn crawl(
            &self,
            _config: &crate::domain::crawl_result::CrawlConfig,
        ) -> std::result::Result<crate::ports::web_crawler_port::CrawlResult, crate::domain::crawl_result::CrawlError> {
            let mut result = crate::ports::web_crawler_port::CrawlResult::new();
            result.pages = self.pages.clone();
            std::result::Result::Ok(result)
        }

        async fn fetch_page(
            &self,
            _url: &str,
            _config: &crate::domain::crawl_result::CrawlConfig,
        ) -> std::result::Result<crate::domain::crawl_result::CrawledPage, crate::domain::crawl_result::CrawlError> {
            std::result::Result::Err(crate::domain::crawl_result::CrawlError::NetworkError(
                String::from("Not implemented"),
            ))
        }

        fn extract_text(&self, _html: &str, _config: &crate::domain::crawl_result::CrawlConfig) -> String {
            String::new()
        }

        fn extract_title(&self, _html: &str) -> String {
            String::new()
        }

        fn extract_links(&self, _html: &str, _base_url: &str) -> std::vec::Vec<String> {
            std::vec::Vec::new()
        }

        fn should_follow(&self, _url: &str, _base_url: &str, _config: &crate::domain::crawl_result::CrawlConfig) -> bool {
            false
        }
    }

    /// Mock embedding port for testing.
    struct MockEmbeddingPort {
        dimension: usize,
    }

    #[async_trait::async_trait]
    impl crate::ports::embedding_port::EmbeddingPort for MockEmbeddingPort {
        async fn generate_embedding(&self, _text: &str) -> std::result::Result<std::vec::Vec<f32>, String> {
            std::result::Result::Ok(std::vec![0.1; self.dimension])
        }

        async fn generate_embeddings(&self, texts: &[&str]) -> std::result::Result<std::vec::Vec<std::vec::Vec<f32>>, String> {
            std::result::Result::Ok(std::vec![std::vec![0.1; self.dimension]; texts.len()])
        }

        async fn embedding_dimension(&self) -> usize {
            self.dimension
        }
    }

    /// Mock artifact repository for testing.
    struct MockArtifactRepository {
        saved_count: std::sync::atomic::AtomicUsize,
    }

    impl MockArtifactRepository {
        fn new() -> Self {
            MockArtifactRepository {
                saved_count: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn get_saved_count(&self) -> usize {
            self.saved_count.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl hexser::ports::Repository<task_manager::domain::artifact::Artifact> for MockArtifactRepository {
        fn save(&mut self, _entity: task_manager::domain::artifact::Artifact) -> hexser::HexResult<()> {
            self.saved_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            std::result::Result::Ok(())
        }
    }

    impl hexser::ports::repository::QueryRepository<task_manager::domain::artifact::Artifact> for MockArtifactRepository {
        type Filter = task_manager::ports::artifact_repository_port::ArtifactFilter;
        type SortKey = task_manager::ports::artifact_repository_port::ArtifactSortKey;

        fn find_one(&self, _filter: &Self::Filter) -> hexser::HexResult<std::option::Option<task_manager::domain::artifact::Artifact>> {
            std::result::Result::Ok(std::option::Option::None)
        }

        fn find(&self, _filter: &Self::Filter, _options: hexser::ports::repository::FindOptions<Self::SortKey>) -> hexser::HexResult<std::vec::Vec<task_manager::domain::artifact::Artifact>> {
            std::result::Result::Ok(std::vec::Vec::new())
        }
    }

    impl task_manager::ports::artifact_repository_port::ArtifactRepositoryPort for MockArtifactRepository {
        fn find_similar(
            &self,
            _query_embedding: &[f32],
            _limit: usize,
            _threshold: std::option::Option<f32>,
            _project_id: std::option::Option<String>,
        ) -> std::result::Result<std::vec::Vec<task_manager::ports::artifact_repository_port::SimilarArtifact>, String> {
            std::result::Result::Ok(std::vec::Vec::new())
        }
    }

    #[test]
    fn test_generation_report_new() {
        // Test: Validates new report is empty.
        // Justification: Fresh reports should have zero counts.
        let report = GenerationReport::new();

        std::assert_eq!(report.files_scanned, 0);
        std::assert_eq!(report.pages_crawled, 0);
        std::assert_eq!(report.artifacts_created, 0);
        std::assert!(!report.has_errors());
    }

    #[test]
    fn test_generation_report_add_error() {
        // Test: Validates error tracking.
        // Justification: Errors should be recorded for reporting.
        let mut report = GenerationReport::new();

        report.add_error(String::from("Test error"));

        std::assert!(report.has_errors());
        std::assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_generation_config_builder() {
        // Test: Validates config builder pattern.
        // Justification: Config should be configurable via fluent API.
        let config = GenerationConfig::new(String::from("project-123"))
            .with_chunk_strategy(task_manager::domain::scan_config::ChunkStrategy::Sentence)
            .with_max_chunk_size(500)
            .with_incremental(true);

        std::assert_eq!(config.project_id, "project-123");
        std::assert!(matches!(config.chunk_strategy, task_manager::domain::scan_config::ChunkStrategy::Sentence));
        std::assert_eq!(config.max_chunk_size, 500);
        std::assert!(config.incremental);
    }

    #[test]
    fn test_chunk_by_paragraph() {
        // Test: Validates paragraph chunking.
        // Justification: Core chunking strategy.
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = ArtifactGeneratorService::chunk_by_paragraph(text);

        std::assert_eq!(chunks.len(), 3);
        std::assert_eq!(chunks[0], "First paragraph.");
        std::assert_eq!(chunks[1], "Second paragraph.");
        std::assert_eq!(chunks[2], "Third paragraph.");
    }

    #[test]
    fn test_chunk_by_sentence() {
        // Test: Validates sentence chunking.
        // Justification: Alternative chunking strategy.
        let text = "First sentence. Second sentence! Third sentence?";
        let chunks = ArtifactGeneratorService::chunk_by_sentence(text);

        std::assert_eq!(chunks.len(), 3);
        std::assert_eq!(chunks[0], "First sentence.");
        std::assert_eq!(chunks[1], "Second sentence!");
        std::assert_eq!(chunks[2], "Third sentence?");
    }

    #[test]
    fn test_chunk_by_size() {
        // Test: Validates fixed-size chunking.
        // Justification: Size-based strategy for predictable chunks.
        let text = "ABCDEFGHIJ";
        let chunks = ArtifactGeneratorService::chunk_by_size(text, 3);

        std::assert_eq!(chunks.len(), 4);
        std::assert_eq!(chunks[0], "ABC");
        std::assert_eq!(chunks[1], "DEF");
        std::assert_eq!(chunks[2], "GHI");
        std::assert_eq!(chunks[3], "J");
    }

    #[test]
    fn test_artifact_type_from_extension() {
        // Test: Validates extension to type mapping.
        // Justification: Correct type classification for artifacts.
        std::assert!(matches!(
            ArtifactGeneratorService::artifact_type_from_extension("md"),
            task_manager::domain::artifact::ArtifactType::PRD
        ));
        std::assert!(matches!(
            ArtifactGeneratorService::artifact_type_from_extension("rs"),
            task_manager::domain::artifact::ArtifactType::File
        ));
        std::assert!(matches!(
            ArtifactGeneratorService::artifact_type_from_extension("pdf"),
            task_manager::domain::artifact::ArtifactType::PDF
        ));
        std::assert!(matches!(
            ArtifactGeneratorService::artifact_type_from_extension("png"),
            task_manager::domain::artifact::ArtifactType::Image
        ));
    }

    #[tokio::test]
    async fn test_generate_from_directory_empty() {
        // Test: Validates empty directory handling.
        // Justification: No files should produce zero artifacts.
        let scanner = std::sync::Arc::new(MockDirectoryScanner { files: std::vec::Vec::new() });
        let crawler = std::sync::Arc::new(MockWebCrawler { pages: std::vec::Vec::new() });
        let embedding = std::sync::Arc::new(MockEmbeddingPort { dimension: 384 });
        let repo = std::sync::Arc::new(std::sync::Mutex::new(MockArtifactRepository::new()));

        let service = ArtifactGeneratorService::new(scanner, crawler, embedding, repo);

        let config = GenerationConfig::new(String::from("project-123"));
        let scan_config = task_manager::domain::scan_config::ScanConfig::new(String::from("/test"));

        let result = service.generate_from_directory("/test", &config, &scan_config).await;

        std::assert!(result.is_ok());
        let report = result.unwrap();
        std::assert_eq!(report.files_scanned, 0);
        std::assert_eq!(report.artifacts_created, 0);
    }

    #[tokio::test]
    async fn test_generate_from_directory_with_files() {
        // Test: Validates file processing creates artifacts.
        // Justification: End-to-end directory generation.
        let files = std::vec![
            task_manager::domain::scan_config::ScannedFile {
                path: String::from("src/main.rs"),
                absolute_path: String::from("/test/src/main.rs"),
                content: String::from("fn main() {}\n\nfn helper() {}"),
                extension: String::from("rs"),
                size_bytes: 30,
                fingerprint: task_manager::domain::scan_config::FileFingerprint::new(
                    String::from("abc123"),
                    1234567890,
                    30,
                ),
                line_count: 4,
            },
        ];

        let scanner = std::sync::Arc::new(MockDirectoryScanner { files });
        let crawler = std::sync::Arc::new(MockWebCrawler { pages: std::vec::Vec::new() });
        let embedding = std::sync::Arc::new(MockEmbeddingPort { dimension: 384 });
        let repo = std::sync::Arc::new(std::sync::Mutex::new(MockArtifactRepository::new()));

        let service = ArtifactGeneratorService::new(
            scanner,
            crawler,
            embedding,
            repo.clone(),
        );

        let config = GenerationConfig::new(String::from("project-123"));
        let scan_config = task_manager::domain::scan_config::ScanConfig::new(String::from("/test"));

        let result = service.generate_from_directory("/test", &config, &scan_config).await;

        std::assert!(result.is_ok());
        let report = result.unwrap();
        std::assert_eq!(report.files_scanned, 1);
        std::assert_eq!(report.artifacts_created, 2); // Two paragraphs

        // Verify repository was called
        let saved = repo.lock().unwrap().get_saved_count();
        std::assert_eq!(saved, 2);
    }

    #[tokio::test]
    async fn test_generate_from_url_with_pages() {
        // Test: Validates page processing creates artifacts.
        // Justification: End-to-end URL generation.
        let pages = std::vec![
            crate::domain::crawl_result::CrawledPage {
                url: String::from("https://example.com/"),
                title: String::from("Example"),
                content: String::from("First section.\n\nSecond section."),
                links: std::vec::Vec::new(),
                depth: 0,
                status_code: 200,
                content_type: String::from("text/html"),
            },
        ];

        let scanner = std::sync::Arc::new(MockDirectoryScanner { files: std::vec::Vec::new() });
        let crawler = std::sync::Arc::new(MockWebCrawler { pages });
        let embedding = std::sync::Arc::new(MockEmbeddingPort { dimension: 384 });
        let repo = std::sync::Arc::new(std::sync::Mutex::new(MockArtifactRepository::new()));

        let service = ArtifactGeneratorService::new(
            scanner,
            crawler,
            embedding,
            repo.clone(),
        );

        let config = GenerationConfig::new(String::from("project-123"));
        let crawl_config = crate::domain::crawl_result::CrawlConfig::new(String::from("https://example.com"));

        let result = service.generate_from_url("https://example.com", &config, &crawl_config).await;

        std::assert!(result.is_ok());
        let report = result.unwrap();
        std::assert_eq!(report.pages_crawled, 1);
        std::assert_eq!(report.artifacts_created, 2); // Two paragraphs

        // Verify repository was called
        let saved = repo.lock().unwrap().get_saved_count();
        std::assert_eq!(saved, 2);
    }
}
