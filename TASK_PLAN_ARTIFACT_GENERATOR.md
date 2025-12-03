# **PRD Artifact Generator Implementation Plan**

---
task_id: ARTIFACT-GENERATOR-001
status: completed
---

## Context

We are building a PRD Artifact Generator that scans directories and crawls websites to build knowledge artifacts for RAG-enhanced task generation. The generator respects `.gitignore` patterns, chunks content intelligently, generates embeddings, and persists artifacts to the SQLite database.

**Use Case**: Before generating tasks from a PRD, users can run `rig artifacts generate ./src` or `rig artifacts generate https://docs.example.com` to pre-populate the artifact database with relevant context. The task generation LLM can then retrieve semantically relevant artifacts to improve output quality.

## Architecture Overview

### Domain Layer (task_manager)

**Existing Entities** (reuse):
- `Artifact` - Already defined with embedding support
- `ArtifactType` - PRD, File, WebResearch, UserInput, Image, PDF

**New Value Objects**:
- `ScanConfig` - Configuration for directory/web scanning
  - source_path: String (directory path or URL)
  - include_patterns: Vec<String> (glob patterns to include)
  - exclude_patterns: Vec<String> (additional excludes beyond .gitignore)
  - max_depth: Option<usize> (directory recursion limit)
  - max_file_size: usize (skip files larger than this)
  - chunk_strategy: ChunkStrategy (paragraph, sentence, fixed_size)

### Ports Layer

**task_manager/src/ports/**:
- `DirectoryScannerPort` - Scans directories respecting .gitignore
  - `scan(config: &ScanConfig) -> Result<Vec<ScannedFile>, ScanError>`
  - `read_file(path: &Path) -> Result<FileContent, ScanError>`

**task_orchestrator/src/ports/**:
- `WebCrawlerPort` - Crawls websites and extracts content
  - `crawl(url: &str, depth: usize) -> Result<Vec<CrawledPage>, CrawlError>`
  - `fetch_page(url: &str) -> Result<PageContent, CrawlError>`

### Adapters Layer

**task_manager/src/adapters/**:
- `IgnoreAwareScanner` - Uses `ignore` crate for .gitignore-aware directory walking
  - Respects .gitignore, .git/info/exclude, global gitignore
  - Filters by file extension (code, docs, config files)
  - Skips binary files and large files

**task_orchestrator/src/adapters/**:
- `ReqwestWebCrawler` - HTTP-based web crawler
  - Follows links within same domain (configurable depth)
  - Extracts text content from HTML (strip tags)
  - Handles rate limiting and robots.txt (optional)

### Services Layer

**task_orchestrator/src/services/**:
- `ArtifactGeneratorService` - Orchestrates the full pipeline
  - Accepts directory path OR URL
  - Routes to appropriate scanner/crawler
  - Chunks content using existing ArtifactService logic
  - Generates embeddings via EmbeddingPort
  - Persists artifacts via ArtifactRepositoryPort

## Implementation Phases

### Phase 1: Domain Modeling ✅ COMPLETE

- [x] **Task 1.1**: Create `task_manager/src/domain/scan_config.rs`
  - ✅ Defined `ScanConfig` struct with builder pattern
  - ✅ Defined `ChunkStrategy` enum (Paragraph, Sentence, FixedSize, WholeFile)
  - ✅ Defined `ScannedFile` struct (path, absolute_path, content, extension, size_bytes)
  - ✅ Defined `ScanError` error type with 5 variants
  - ✅ Defined `ScanStats` for operation metrics
  - ✅ 10 unit tests passing

- [x] **Task 1.2**: Create `task_orchestrator/src/domain/crawl_result.rs`
  - ✅ Defined `CrawledPage` struct (url, title, content, links, depth, status_code)
  - ✅ Defined `CrawlError` error type with 7 variants
  - ✅ Defined `CrawlConfig` struct with builder pattern
  - ✅ Defined `CrawlStats` for operation metrics
  - ✅ 9 unit tests passing

### Phase 2: Directory Scanner (Muscle) ✅ COMPLETE

- [x] **Task 2.1**: Add `ignore` crate dependency to workspace
  - ✅ Added ignore = "0.4" to workspace Cargo.toml
  - ✅ Added to task_manager Cargo.toml

- [x] **Task 2.2**: Create `task_manager/src/ports/directory_scanner_port.rs`
  - ✅ Defined `DirectoryScannerPort` async trait
  - ✅ Methods: scan(), read_file(), has_file_changed(), find_deleted_files()
  - ✅ Defined `ScanResult` (files, stats, errors)
  - ✅ Defined `ScanFileError` for non-fatal errors
  - ✅ 3 unit tests passing

- [x] **Task 2.3**: Create `task_manager/src/adapters/ignore_aware_scanner.rs`
  - ✅ Implemented `DirectoryScannerPort` using `ignore::WalkBuilder`
  - ✅ Respects .gitignore, .git/info/exclude, global gitignore
  - ✅ Filters by extension (configurable allowlist)
  - ✅ Skips files > max_file_size (default 1MB)
  - ✅ Detects and skips binary files (null byte check)
  - ✅ Computes FileFingerprint for change detection
  - ✅ Tracks line_count for each file
  - ✅ 12 unit tests passing

### Phase 3: Web Crawler (Muscle) ✅ COMPLETE

- [x] **Task 3.1**: Create `task_orchestrator/src/ports/web_crawler_port.rs`
  - ✅ Defined `WebCrawlerPort` trait with async methods (crawl, fetch_page, extract_text, extract_title, extract_links, should_follow)
  - ✅ Defined `CrawlResult` struct (pages, stats, errors)
  - ✅ Defined `CrawlPageError` for non-fatal page errors
  - ✅ 5 unit tests passing

- [x] **Task 3.2**: Create `task_orchestrator/src/adapters/reqwest_web_crawler.rs`
  - ✅ Implemented `WebCrawlerPort` using `reqwest` HTTP client
  - ✅ Added `scraper` crate for HTML parsing
  - ✅ Extract text content from HTML body
  - ✅ Extract links and resolve relative URLs
  - ✅ Domain extraction for same-domain checking
  - ✅ URL normalization (remove fragments, trailing slashes)
  - ✅ Configurable depth, page limit, follow_external
  - ✅ 12 unit tests passing

### Phase 4: Generator Service (Flow) ✅ COMPLETE

- [x] **Task 4.1**: Create `task_orchestrator/src/services/artifact_generator_service.rs`
  - ✅ Constructor takes: DirectoryScannerPort, WebCrawlerPort, EmbeddingPort, ArtifactRepositoryPort
  - ✅ `generate_from_directory(path, config, scan_config) -> Result<GenerationReport>`
  - ✅ `generate_from_url(url, config, crawl_config) -> Result<GenerationReport>`
  - ✅ Multiple chunking strategies: Paragraph, Sentence, FixedSize, WholeFile
  - ✅ Artifact type detection from file extension
  - ✅ 10 unit tests passing

- [x] **Task 4.2**: Define `GenerationReport` struct
  - ✅ files_scanned, pages_crawled, artifacts_created, chunks_generated, bytes_processed
  - ✅ errors: Vec<String> with error tracking methods
  - ✅ duration_ms: u64

- [x] **Task 4.3**: Define `GenerationConfig` struct
  - ✅ project_id, chunk_strategy, max_chunk_size, incremental
  - ✅ Builder pattern with fluent API

### Phase 5: CLI Integration ✅ COMPLETE

- [x] **Task 5.1**: Add `rig artifacts generate` command
  - ✅ Added Generate variant to ArtifactsCommands enum
  - ✅ Arguments: `<source>` (path or URL)
  - ✅ Options: `--project`, `--depth`, `--max-items`, `--chunk-strategy`, `--chunk-size`, `--exclude`
  - ✅ Auto-detects source type (directory vs URL)
  - ✅ Auto-generates project ID from directory name or domain

- [x] **Task 5.2**: Add progress reporting
  - ✅ Shows files/pages count during processing
  - ✅ Shows final summary (artifacts created, bytes processed, duration)
  - ✅ Shows errors with count and details (max 10)

### Phase 6: TUI Integration ✅ COMPLETE

- [x] **Task 6.1**: Add "Generate Artifacts" action to TUI
  - ✅ New keyboard shortcut 'G' (uppercase) for artifact generation
  - ✅ Dialog to enter source path/URL with examples
  - ✅ Progress display during generation (shows status messages)
  - ✅ Notification on completion (success with stats or error message)
  - ✅ Support for both directory scanning and web crawling
  - ✅ Auto-generates project ID from source path or domain
  - ✅ Uses configured embedding provider from .rigger/config.json
  - ✅ Added to keyboard shortcuts help overlay

## Technical Guidelines

1. **No use statements**: All paths must be fully qualified
2. **Single File Responsibility**: One struct/trait per file
3. **Error Handling**: Use Result<T, String> or custom error types
4. **Async/Await**: All Ports and Adapters must be async
5. **Testing**: Every adapter must have unit tests; integration tests for CLI

## Dependencies

**New crates to add**:
- `ignore` = "0.4" - Gitignore-aware directory walking
- `scraper` = "0.22" (optional) - HTML parsing and text extraction

**Existing crates to reuse**:
- `reqwest` - HTTP client (already in workspace)
- `regex` - Pattern matching (already in workspace)
- `tokio` - Async runtime (already in workspace)

## File Extension Allowlist

Default extensions to include when scanning directories:

**Code**: `.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`, `.go`, `.java`, `.kt`, `.swift`, `.c`, `.cpp`, `.h`, `.hpp`, `.cs`, `.rb`, `.php`

**Documentation**: `.md`, `.txt`, `.rst`, `.adoc`

**Configuration**: `.json`, `.yaml`, `.yml`, `.toml`, `.xml`, `.ini`, `.env.example`

**Web**: `.html`, `.css`, `.scss`, `.less`

## Current Step

- **Action**: All phases COMPLETE
- **Details**: Full artifact generator implementation complete including:
  - Domain modeling (ScanConfig, CrawlConfig, ChunkStrategy)
  - Directory scanner with gitignore support (IgnoreAwareScanner)
  - Web crawler with link following (ReqwestWebCrawler)
  - Generator service orchestrating the full pipeline
  - CLI command `rig artifacts generate`
  - TUI dialog with 'G' shortcut

## Blockers

- None - implementation complete

