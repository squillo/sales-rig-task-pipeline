//! Implementation of 'rig artifacts' commands.
//!
//! Provides CLI commands for listing, searching, and generating knowledge
//! artifacts in the RAG system.
//!
//! Revision History
//! - 2025-11-30T21:30:00Z @AI: Add generate command for Phase 5 artifact generator CLI.
//! - 2025-11-28T23:00:00Z @AI: Create artifacts CLI commands for Phase 6 (Task 6.1, 6.2).

/// Executes the 'rig artifacts list' command.
///
/// Lists all artifacts from the SQLite database with optional filtering by
/// project and source type.
///
/// # Arguments
///
/// * `project_id` - Optional project ID filter
/// * `source_type` - Optional source type filter (prd, file, web_research, user_input)
/// * `limit` - Maximum number of artifacts to display
///
/// # Errors
///
/// Returns an error if:
/// - .rigger directory doesn't exist (run 'rig init' first)
/// - Database connection fails
/// - Query execution fails
pub async fn list(
    project_id: std::option::Option<&str>,
    source_type: std::option::Option<&str>,
    limit: std::option::Option<usize>,
) -> anyhow::Result<()> {
    // Check if .rigger exists
    let current_dir = std::env::current_dir()?;
    let rigger_dir = current_dir.join(".rigger");

    if !rigger_dir.exists() {
        anyhow::bail!(
            ".rigger directory not found.\nRun 'rig init' first to initialize the project."
        );
    }

    // Connect to database
    let db_path = rigger_dir.join("tasks.db");
    let db_url = std::format!("sqlite:{}", db_path.display());

    let adapter = task_manager::adapters::sqlite_artifact_adapter::SqliteArtifactAdapter::connect_and_init(&db_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    // Build filter
    let filter = if let std::option::Option::Some(proj_id) = project_id {
        task_manager::ports::artifact_repository_port::ArtifactFilter::ByProjectId(std::string::String::from(proj_id))
    } else if let std::option::Option::Some(src_type) = source_type {
        // Parse source type string to ArtifactType enum
        let artifact_type = match src_type.to_lowercase().as_str() {
            "prd" => task_manager::domain::artifact::ArtifactType::PRD,
            "file" => task_manager::domain::artifact::ArtifactType::File,
            "web_research" | "web" => task_manager::domain::artifact::ArtifactType::WebResearch,
            "user_input" | "user" => task_manager::domain::artifact::ArtifactType::UserInput,
            _ => {
                anyhow::bail!(
                    "Invalid source_type: '{}'. Valid values: prd, file, web_research, user_input",
                    src_type
                );
            }
        };
        task_manager::ports::artifact_repository_port::ArtifactFilter::BySourceType(artifact_type)
    } else {
        task_manager::ports::artifact_repository_port::ArtifactFilter::All
    };

    // Query artifacts
    let sort = std::vec![hexser::ports::repository::Sort {
        key: task_manager::ports::artifact_repository_port::ArtifactSortKey::CreatedAt,
        direction: hexser::ports::repository::Direction::Desc,
    }];

    let find_options = hexser::ports::repository::FindOptions {
        sort: std::option::Option::Some(sort),
        limit: limit.map(|l| l as u32).or(std::option::Option::Some(20)),
        offset: std::option::Option::None,
    };

    let artifacts = hexser::ports::repository::QueryRepository::find(&adapter, &filter, find_options)
        .map_err(|e| anyhow::anyhow!("Failed to query artifacts: {}", e))?;

    // Display results
    if artifacts.is_empty() {
        println!("No artifacts found.");
        return std::result::Result::Ok(());
    }

    println!("Found {} artifacts:\n", artifacts.len());

    for (i, artifact) in artifacts.iter().enumerate() {
        let content_preview = if artifact.content.len() > 100 {
            std::format!("{}...", &artifact.content[..100])
        } else {
            artifact.content.clone()
        };

        println!("{}. [{}] {:?}", i + 1, artifact.id, artifact.source_type);
        println!("   Project: {}", artifact.project_id);
        println!("   Source: {}", artifact.source_id);
        println!("   Content: {}", content_preview);
        println!("   Created: {}", artifact.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!();
    }

    std::result::Result::Ok(())
}

/// Executes the 'rig artifacts search' command.
///
/// Performs semantic search on the artifact knowledge base using vector
/// similarity. Requires an embedding service to be available.
///
/// # Arguments
///
/// * `query` - Natural language search query
/// * `limit` - Maximum number of results to return (default: 5)
/// * `threshold` - Minimum similarity threshold 0.0-1.0 (default: 0.5)
/// * `project_id` - Optional project ID to scope search
///
/// # Errors
///
/// Returns an error if:
/// - .rigger directory doesn't exist
/// - Database connection fails
/// - Embedding service is unavailable
/// - Search fails
pub async fn search(
    query: &str,
    limit: std::option::Option<usize>,
    threshold: std::option::Option<f32>,
    project_id: std::option::Option<&str>,
) -> anyhow::Result<()> {
    // Check if .rigger exists
    let current_dir = std::env::current_dir()?;
    let rigger_dir = current_dir.join(".rigger");

    if !rigger_dir.exists() {
        anyhow::bail!(
            ".rigger directory not found.\nRun 'rig init' first to initialize the project."
        );
    }

    // Read config to determine provider
    let config_path = rigger_dir.join("config.json");
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config.json: {}", e))?;
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    let provider = config["provider"]
        .as_str()
        .unwrap_or("ollama");

    println!("Searching artifacts for: \"{}\"", query);
    println!("Using {} embedding service...\n", provider);

    // Connect to database
    let db_path = rigger_dir.join("tasks.db");
    let db_url = std::format!("sqlite:{}", db_path.display());

    let artifact_adapter = task_manager::adapters::sqlite_artifact_adapter::SqliteArtifactAdapter::connect_and_init(&db_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    // Create embedding adapter using provider factory
    let provider_factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new(provider, "default")
        .map_err(|e| anyhow::anyhow!("Failed to create provider factory: {}", e))?;

    let embedding_adapter = provider_factory.create_embedding_adapter()
        .map_err(|e| anyhow::anyhow!("Failed to create embedding adapter: {}", e))?;

    // Generate query embedding
    let query_embedding = embedding_adapter.generate_embedding(query)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate embedding: {}", e))?;

    // Search for similar artifacts
    let search_limit = limit.unwrap_or(5);
    let search_threshold = threshold.unwrap_or(0.5);
    let search_project_id = project_id.map(|s| std::string::String::from(s));

    let similar_artifacts = task_manager::ports::artifact_repository_port::ArtifactRepositoryPort::find_similar(
        &artifact_adapter,
        &query_embedding,
        search_limit,
        std::option::Option::Some(search_threshold),
        search_project_id,
    ).map_err(|e| anyhow::anyhow!("Failed to search artifacts: {}", e))?;

    // Display results
    if similar_artifacts.is_empty() {
        println!("No relevant artifacts found matching your query.");
        println!("Try:");
        println!("  - Using different keywords");
        println!("  - Lowering the threshold (--threshold 0.3)");
        println!("  - Increasing the limit (--limit 10)");
        return std::result::Result::Ok(());
    }

    println!("Found {} relevant artifacts:\n", similar_artifacts.len());

    for (i, similar) in similar_artifacts.iter().enumerate() {
        let artifact = &similar.artifact;
        let distance = similar.distance;
        let similarity = (1.0 - distance) * 100.0;

        let content_preview = if artifact.content.len() > 200 {
            std::format!("{}...", &artifact.content[..200])
        } else {
            artifact.content.clone()
        };

        println!("{}. [Similarity: {:.1}%] {:?}", i + 1, similarity, artifact.source_type);
        println!("   ID: {}", artifact.id);
        println!("   Project: {}", artifact.project_id);
        println!("   Source: {}", artifact.source_id);
        println!("   Content: {}", content_preview);
        println!("   Created: {}", artifact.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!();
    }

    std::result::Result::Ok(())
}

/// Executes the 'rig artifacts generate' command.
///
/// Generates artifacts from a directory or website by scanning/crawling,
/// chunking content, generating embeddings, and persisting to the database.
///
/// # Arguments
///
/// * `source` - Directory path or URL to generate artifacts from
/// * `project_id` - Optional project ID (defaults to directory name or domain)
/// * `depth` - Maximum recursion depth (default: 10)
/// * `max_items` - Maximum files/pages to process (default: 1000)
/// * `chunk_strategy` - Chunking strategy: paragraph, sentence, fixed_size, whole_file
/// * `chunk_size` - Max chunk size for fixed_size strategy (default: 1000)
/// * `exclude_patterns` - Additional glob patterns to exclude
///
/// # Errors
///
/// Returns an error if:
/// - .rigger directory doesn't exist
/// - Source path/URL is invalid
/// - Scanning/crawling fails
/// - Embedding generation fails
pub async fn generate(
    source: &str,
    project_id: std::option::Option<&str>,
    depth: std::option::Option<usize>,
    max_items: std::option::Option<usize>,
    chunk_strategy: std::option::Option<&str>,
    chunk_size: std::option::Option<usize>,
    exclude_patterns: std::option::Option<&str>,
) -> anyhow::Result<()> {
    // Check if .rigger exists
    let current_dir = std::env::current_dir()?;
    let rigger_dir = current_dir.join(".rigger");

    if !rigger_dir.exists() {
        anyhow::bail!(
            ".rigger directory not found.\nRun 'rig init' first to initialize the project."
        );
    }

    // Determine if source is URL or directory
    let is_url = source.starts_with("http://") || source.starts_with("https://");

    // Determine project ID
    let project = if let std::option::Option::Some(p) = project_id {
        String::from(p)
    } else if is_url {
        // Extract domain as project ID
        reqwest::Url::parse(source)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_else(|| String::from("default"))
    } else {
        // Use directory name as project ID
        std::path::Path::new(source)
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
            .unwrap_or_else(|| String::from("default"))
    };

    // Read config to determine provider
    let config_path = rigger_dir.join("config.json");
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config.json: {}", e))?;
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    let provider = config["provider"]
        .as_str()
        .unwrap_or("ollama");

    println!("Generating artifacts from: {}", source);
    println!("Project ID: {}", project);
    println!("Provider: {}", provider);
    println!();

    // Connect to database
    let db_path = rigger_dir.join("tasks.db");
    let db_url = std::format!("sqlite:{}", db_path.display());

    let artifact_adapter = task_manager::adapters::sqlite_artifact_adapter::SqliteArtifactAdapter::connect_and_init(&db_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    // Create embedding adapter
    let provider_factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new(provider, "default")
        .map_err(|e| anyhow::anyhow!("Failed to create provider factory: {}", e))?;

    let embedding_adapter = provider_factory.create_embedding_adapter()
        .map_err(|e| anyhow::anyhow!("Failed to create embedding adapter: {}", e))?;

    // Parse chunking strategy
    let strategy = match chunk_strategy.unwrap_or("paragraph") {
        "paragraph" => task_manager::domain::scan_config::ChunkStrategy::Paragraph,
        "sentence" => task_manager::domain::scan_config::ChunkStrategy::Sentence,
        "fixed_size" => task_manager::domain::scan_config::ChunkStrategy::FixedSize(chunk_size.unwrap_or(1000)),
        "whole_file" => task_manager::domain::scan_config::ChunkStrategy::WholeFile,
        other => anyhow::bail!(
            "Invalid chunk strategy: '{}'. Valid values: paragraph, sentence, fixed_size, whole_file",
            other
        ),
    };

    // Create generation config
    let gen_config = task_orchestrator::services::artifact_generator_service::GenerationConfig::new(project.clone())
        .with_chunk_strategy(strategy)
        .with_max_chunk_size(chunk_size.unwrap_or(1000));

    // Wrap adapters in Arc
    let artifact_repo = std::sync::Arc::new(std::sync::Mutex::new(artifact_adapter));

    if is_url {
        // Generate from URL
        generate_from_url(
            source,
            &gen_config,
            depth.unwrap_or(3),
            max_items.unwrap_or(100),
            embedding_adapter,
            artifact_repo,
        ).await
    } else {
        // Generate from directory
        generate_from_directory(
            source,
            &gen_config,
            depth.unwrap_or(10),
            max_items.unwrap_or(1000),
            exclude_patterns,
            embedding_adapter,
            artifact_repo,
        ).await
    }
}

/// Generates artifacts from a directory.
async fn generate_from_directory(
    path: &str,
    config: &task_orchestrator::services::artifact_generator_service::GenerationConfig,
    max_depth: usize,
    max_files: usize,
    exclude_patterns: std::option::Option<&str>,
    embedding_adapter: std::sync::Arc<dyn task_orchestrator::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>,
    artifact_repo: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
) -> anyhow::Result<()> {
    println!("Scanning directory: {}", path);

    // Create directory scanner
    let scanner = std::sync::Arc::new(task_manager::adapters::ignore_aware_scanner::IgnoreAwareScanner::new());

    // Create web crawler (not used but required by service)
    let crawler = std::sync::Arc::new(DummyWebCrawler);

    // Create generator service
    let service = task_orchestrator::services::artifact_generator_service::ArtifactGeneratorService::new(
        scanner,
        crawler,
        embedding_adapter,
        artifact_repo,
    );

    // Build scan config
    let mut scan_config = task_manager::domain::scan_config::ScanConfig::new(String::from(path));
    scan_config.max_depth = std::option::Option::Some(max_depth);

    // Add exclude patterns if provided
    if let std::option::Option::Some(patterns) = exclude_patterns {
        let excludes: std::vec::Vec<String> = patterns.split(',')
            .map(|p| p.trim().to_string())
            .collect();
        scan_config.exclude_patterns = excludes;
    }

    // Generate artifacts
    println!("Scanning files (max depth: {}, max files: {})...", max_depth, max_files);

    let report = service.generate_from_directory(path, config, &scan_config)
        .await
        .map_err(|e| anyhow::anyhow!("Generation failed: {}", e))?;

    // Display results
    println!("\nGeneration complete!");
    println!("  Files scanned: {}", report.files_scanned);
    println!("  Artifacts created: {}", report.artifacts_created);
    println!("  Bytes processed: {}", format_bytes(report.bytes_processed));
    println!("  Duration: {}ms", report.duration_ms);

    if report.has_errors() {
        println!("\n{} errors occurred:", report.error_count());
        for (i, error) in report.errors.iter().take(10).enumerate() {
            println!("  {}. {}", i + 1, error);
        }
        if report.error_count() > 10 {
            println!("  ... and {} more", report.error_count() - 10);
        }
    }

    std::result::Result::Ok(())
}

/// Generates artifacts from a URL.
async fn generate_from_url(
    url: &str,
    config: &task_orchestrator::services::artifact_generator_service::GenerationConfig,
    max_depth: usize,
    max_pages: usize,
    embedding_adapter: std::sync::Arc<dyn task_orchestrator::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>,
    artifact_repo: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
) -> anyhow::Result<()> {
    println!("Crawling website: {}", url);

    // Create directory scanner (not used but required by service)
    let scanner = std::sync::Arc::new(DummyDirectoryScanner);

    // Create web crawler
    let crawler = std::sync::Arc::new(task_orchestrator::adapters::reqwest_web_crawler::ReqwestWebCrawler::new());

    // Create generator service
    let service = task_orchestrator::services::artifact_generator_service::ArtifactGeneratorService::new(
        scanner,
        crawler,
        embedding_adapter,
        artifact_repo,
    );

    // Build crawl config
    let mut crawl_config = task_orchestrator::domain::crawl_result::CrawlConfig::new(String::from(url));
    crawl_config.max_depth = max_depth;
    crawl_config.max_pages = max_pages;

    // Generate artifacts
    println!("Crawling pages (max depth: {}, max pages: {})...", max_depth, max_pages);

    let report = service.generate_from_url(url, config, &crawl_config)
        .await
        .map_err(|e| anyhow::anyhow!("Generation failed: {}", e))?;

    // Display results
    println!("\nGeneration complete!");
    println!("  Pages crawled: {}", report.pages_crawled);
    println!("  Artifacts created: {}", report.artifacts_created);
    println!("  Bytes processed: {}", format_bytes(report.bytes_processed));
    println!("  Duration: {}ms", report.duration_ms);

    if report.has_errors() {
        println!("\n{} errors occurred:", report.error_count());
        for (i, error) in report.errors.iter().take(10).enumerate() {
            println!("  {}. {}", i + 1, error);
        }
        if report.error_count() > 10 {
            println!("  ... and {} more", report.error_count() - 10);
        }
    }

    std::result::Result::Ok(())
}

/// Formats bytes into human-readable format.
fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        std::format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        std::format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        std::format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        std::format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Dummy directory scanner for URL-only generation.
struct DummyDirectoryScanner;

#[async_trait::async_trait]
impl task_manager::ports::directory_scanner_port::DirectoryScannerPort for DummyDirectoryScanner {
    async fn scan(
        &self,
        _config: &task_manager::domain::scan_config::ScanConfig,
    ) -> std::result::Result<task_manager::ports::directory_scanner_port::ScanResult, task_manager::domain::scan_config::ScanError> {
        std::result::Result::Ok(task_manager::ports::directory_scanner_port::ScanResult::new())
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

/// Dummy web crawler for directory-only generation.
struct DummyWebCrawler;

#[async_trait::async_trait]
impl task_orchestrator::ports::web_crawler_port::WebCrawlerPort for DummyWebCrawler {
    async fn crawl(
        &self,
        _config: &task_orchestrator::domain::crawl_result::CrawlConfig,
    ) -> std::result::Result<task_orchestrator::ports::web_crawler_port::CrawlResult, task_orchestrator::domain::crawl_result::CrawlError> {
        std::result::Result::Ok(task_orchestrator::ports::web_crawler_port::CrawlResult::new())
    }

    async fn fetch_page(
        &self,
        _url: &str,
        _config: &task_orchestrator::domain::crawl_result::CrawlConfig,
    ) -> std::result::Result<task_orchestrator::domain::crawl_result::CrawledPage, task_orchestrator::domain::crawl_result::CrawlError> {
        std::result::Result::Err(task_orchestrator::domain::crawl_result::CrawlError::NetworkError(
            String::from("Not implemented"),
        ))
    }

    fn extract_text(&self, _html: &str, _config: &task_orchestrator::domain::crawl_result::CrawlConfig) -> String {
        String::new()
    }

    fn extract_title(&self, _html: &str) -> String {
        String::new()
    }

    fn extract_links(&self, _html: &str, _base_url: &str) -> std::vec::Vec<String> {
        std::vec::Vec::new()
    }

    fn should_follow(&self, _url: &str, _base_url: &str, _config: &task_orchestrator::domain::crawl_result::CrawlConfig) -> bool {
        false
    }
}
