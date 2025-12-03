//! Defines configuration and result types for directory scanning operations.
//!
//! This module contains value objects for configuring directory scans that respect
//! .gitignore patterns, filter by file extensions, and produce structured results.
//! The ScanConfig controls scan behavior while ScannedFile represents discovered
//! files ready for artifact generation.
//!
//! Key features for incremental updates:
//! - `SourceLocation` tracks line:col positions for each content chunk
//! - `FileFingerprint` enables change detection via content hash + mtime
//! - `ContentChunk` pairs content with its exact source location
//!
//! Revision History
//! - 2025-11-30T19:00:00Z @AI: Add SourceLocation, FileFingerprint, ContentChunk for incremental rescan support.
//! - 2025-11-30T18:30:00Z @AI: Initial scan_config module for Phase 1 artifact generator.

/// Strategy for chunking file content into artifacts.
///
/// ChunkStrategy determines how file content is split into smaller pieces
/// for embedding generation and storage. Different strategies are optimal
/// for different content types.
///
/// # Variants
///
/// * `Paragraph` - Split on double newlines (best for prose/documentation)
/// * `Sentence` - Split on sentence boundaries (best for dense technical text)
/// * `FixedSize` - Split at fixed character count (best for code)
/// * `WholeFile` - Keep entire file as single artifact (best for small files)
///
/// # Examples
///
/// ```
/// # use task_manager::domain::scan_config::ChunkStrategy;
/// let strategy = ChunkStrategy::Paragraph;
/// std::assert!(matches!(strategy, ChunkStrategy::Paragraph));
///
/// let fixed = ChunkStrategy::FixedSize(1000);
/// if let ChunkStrategy::FixedSize(size) = fixed {
///     std::assert_eq!(size, 1000);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum ChunkStrategy {
    /// Split content on double newlines (paragraph boundaries).
    Paragraph,

    /// Split content on sentence boundaries (periods followed by whitespace).
    Sentence,

    /// Split content at fixed character intervals.
    FixedSize(usize),

    /// Keep entire file as a single artifact (no chunking).
    WholeFile,
}

impl std::default::Default for ChunkStrategy {
    fn default() -> Self {
        ChunkStrategy::Paragraph
    }
}

/// Configuration for directory scanning operations.
///
/// ScanConfig controls all aspects of a directory scan including path filtering,
/// file size limits, recursion depth, and content chunking. The scanner respects
/// .gitignore patterns by default and applies additional filters from this config.
///
/// # Fields
///
/// * `source_path` - Root directory to scan (absolute or relative path).
/// * `include_extensions` - File extensions to include (e.g., ["rs", "md", "json"]).
/// * `exclude_patterns` - Glob patterns to exclude beyond .gitignore.
/// * `max_depth` - Maximum directory recursion depth (None = unlimited).
/// * `max_file_size` - Skip files larger than this (bytes).
/// * `chunk_strategy` - How to split file content into artifacts.
/// * `respect_gitignore` - Whether to honor .gitignore patterns (default: true).
///
/// # Examples
///
/// ```
/// # use task_manager::domain::scan_config::{ScanConfig, ChunkStrategy};
/// let config = ScanConfig::new(std::string::String::from("./src"));
///
/// std::assert_eq!(config.source_path, "./src");
/// std::assert!(config.respect_gitignore);
/// std::assert!(config.max_file_size > 0);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ScanConfig {
    /// Root directory path to scan.
    pub source_path: String,

    /// File extensions to include (without leading dot).
    /// Empty vec means include all text files.
    pub include_extensions: std::vec::Vec<String>,

    /// Additional glob patterns to exclude beyond .gitignore.
    pub exclude_patterns: std::vec::Vec<String>,

    /// Maximum directory recursion depth. None means unlimited.
    pub max_depth: std::option::Option<usize>,

    /// Maximum file size in bytes. Files larger than this are skipped.
    pub max_file_size: usize,

    /// Strategy for chunking file content into artifacts.
    pub chunk_strategy: ChunkStrategy,

    /// Whether to respect .gitignore patterns.
    pub respect_gitignore: bool,
}

impl ScanConfig {
    /// Default maximum file size (1 MB).
    const DEFAULT_MAX_FILE_SIZE: usize = 1_048_576;

    /// Creates a new ScanConfig with default settings for the given path.
    ///
    /// # Arguments
    ///
    /// * `source_path` - The root directory path to scan.
    ///
    /// # Returns
    ///
    /// A ScanConfig with sensible defaults:
    /// - All common code/doc extensions included
    /// - 1 MB max file size
    /// - Paragraph chunking
    /// - .gitignore respected
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::scan_config::ScanConfig;
    /// let config = ScanConfig::new(std::string::String::from("./src"));
    /// std::assert!(config.include_extensions.len() > 0);
    /// ```
    pub fn new(source_path: String) -> Self {
        ScanConfig {
            source_path,
            include_extensions: Self::default_extensions(),
            exclude_patterns: std::vec::Vec::new(),
            max_depth: std::option::Option::None,
            max_file_size: Self::DEFAULT_MAX_FILE_SIZE,
            chunk_strategy: ChunkStrategy::Paragraph,
            respect_gitignore: true,
        }
    }

    /// Returns the default list of file extensions to include.
    ///
    /// Covers common code, documentation, and configuration file types.
    pub fn default_extensions() -> std::vec::Vec<String> {
        std::vec![
            // Rust
            String::from("rs"),
            // TypeScript/JavaScript
            String::from("ts"),
            String::from("tsx"),
            String::from("js"),
            String::from("jsx"),
            // Python
            String::from("py"),
            // Go
            String::from("go"),
            // Java/Kotlin
            String::from("java"),
            String::from("kt"),
            // Swift
            String::from("swift"),
            // C/C++
            String::from("c"),
            String::from("cpp"),
            String::from("h"),
            String::from("hpp"),
            // C#
            String::from("cs"),
            // Ruby
            String::from("rb"),
            // PHP
            String::from("php"),
            // Documentation
            String::from("md"),
            String::from("txt"),
            String::from("rst"),
            String::from("adoc"),
            // Configuration
            String::from("json"),
            String::from("yaml"),
            String::from("yml"),
            String::from("toml"),
            String::from("xml"),
            String::from("ini"),
            // Web
            String::from("html"),
            String::from("css"),
            String::from("scss"),
            String::from("less"),
        ]
    }

    /// Creates a config for scanning code files only.
    ///
    /// # Arguments
    ///
    /// * `source_path` - The root directory path to scan.
    ///
    /// # Returns
    ///
    /// A ScanConfig with only code file extensions included.
    pub fn code_only(source_path: String) -> Self {
        ScanConfig {
            source_path,
            include_extensions: std::vec![
                String::from("rs"),
                String::from("ts"),
                String::from("tsx"),
                String::from("js"),
                String::from("jsx"),
                String::from("py"),
                String::from("go"),
                String::from("java"),
                String::from("kt"),
                String::from("swift"),
                String::from("c"),
                String::from("cpp"),
                String::from("h"),
                String::from("hpp"),
                String::from("cs"),
                String::from("rb"),
                String::from("php"),
            ],
            exclude_patterns: std::vec::Vec::new(),
            max_depth: std::option::Option::None,
            max_file_size: Self::DEFAULT_MAX_FILE_SIZE,
            chunk_strategy: ChunkStrategy::FixedSize(2000),
            respect_gitignore: true,
        }
    }

    /// Creates a config for scanning documentation files only.
    ///
    /// # Arguments
    ///
    /// * `source_path` - The root directory path to scan.
    ///
    /// # Returns
    ///
    /// A ScanConfig with only documentation file extensions included.
    pub fn docs_only(source_path: String) -> Self {
        ScanConfig {
            source_path,
            include_extensions: std::vec![
                String::from("md"),
                String::from("txt"),
                String::from("rst"),
                String::from("adoc"),
            ],
            exclude_patterns: std::vec::Vec::new(),
            max_depth: std::option::Option::None,
            max_file_size: Self::DEFAULT_MAX_FILE_SIZE,
            chunk_strategy: ChunkStrategy::Paragraph,
            respect_gitignore: true,
        }
    }

    /// Adds an exclude pattern to this config.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Glob pattern to exclude.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn exclude(mut self, pattern: String) -> Self {
        self.exclude_patterns.push(pattern);
        self
    }

    /// Sets the maximum recursion depth.
    ///
    /// # Arguments
    ///
    /// * `depth` - Maximum depth (0 = root only, 1 = one level deep, etc.)
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = std::option::Option::Some(depth);
        self
    }

    /// Sets the chunk strategy.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The chunking strategy to use.
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_chunk_strategy(mut self, strategy: ChunkStrategy) -> Self {
        self.chunk_strategy = strategy;
        self
    }
}

/// Represents a position in a source file (line and column).
///
/// SourcePosition tracks a specific point in a file using 1-indexed line
/// and column numbers. This enables precise location tracking for artifact
/// chunks during rescans.
///
/// # Fields
///
/// * `line` - 1-indexed line number.
/// * `col` - 1-indexed column number (byte offset within line).
///
/// # Examples
///
/// ```
/// # use task_manager::domain::scan_config::SourcePosition;
/// let pos = SourcePosition::new(10, 5);
/// std::assert_eq!(pos.line, 10);
/// std::assert_eq!(pos.col, 5);
/// std::assert_eq!(pos.to_string(), "10:5");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SourcePosition {
    /// 1-indexed line number.
    pub line: usize,

    /// 1-indexed column number (byte offset within line).
    pub col: usize,
}

impl SourcePosition {
    /// Creates a new SourcePosition.
    ///
    /// # Arguments
    ///
    /// * `line` - 1-indexed line number.
    /// * `col` - 1-indexed column number.
    pub fn new(line: usize, col: usize) -> Self {
        SourcePosition { line, col }
    }

    /// Creates a position at the start of a file (1:1).
    pub fn start() -> Self {
        SourcePosition { line: 1, col: 1 }
    }
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}:{}", self.line, self.col)
    }
}

/// Represents a range in a source file from start to end position.
///
/// SourceLocation defines a span of content in a file using start and end
/// positions. This enables tracking exactly where each artifact chunk originated,
/// supporting incremental updates during rescans.
///
/// # Fields
///
/// * `start` - Starting position (inclusive).
/// * `end` - Ending position (inclusive).
/// * `byte_start` - Byte offset from file start (0-indexed).
/// * `byte_end` - Byte offset of end (exclusive).
///
/// # Examples
///
/// ```
/// # use task_manager::domain::scan_config::{SourceLocation, SourcePosition};
/// let loc = SourceLocation::new(
///     SourcePosition::new(1, 1),
///     SourcePosition::new(5, 20),
///     0,
///     150,
/// );
/// std::assert_eq!(loc.line_count(), 5);
/// std::assert_eq!(loc.to_string(), "1:1-5:20");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SourceLocation {
    /// Starting position (inclusive).
    pub start: SourcePosition,

    /// Ending position (inclusive).
    pub end: SourcePosition,

    /// Byte offset from file start (0-indexed).
    pub byte_start: usize,

    /// Byte offset of end (exclusive).
    pub byte_end: usize,
}

impl SourceLocation {
    /// Creates a new SourceLocation.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting position.
    /// * `end` - Ending position.
    /// * `byte_start` - Byte offset of start.
    /// * `byte_end` - Byte offset of end.
    pub fn new(start: SourcePosition, end: SourcePosition, byte_start: usize, byte_end: usize) -> Self {
        SourceLocation {
            start,
            end,
            byte_start,
            byte_end,
        }
    }

    /// Creates a location spanning the entire file.
    ///
    /// # Arguments
    ///
    /// * `total_lines` - Total number of lines in the file.
    /// * `last_line_cols` - Number of columns in the last line.
    /// * `total_bytes` - Total byte size of the file.
    pub fn whole_file(total_lines: usize, last_line_cols: usize, total_bytes: usize) -> Self {
        SourceLocation {
            start: SourcePosition::start(),
            end: SourcePosition::new(total_lines, last_line_cols),
            byte_start: 0,
            byte_end: total_bytes,
        }
    }

    /// Returns the number of lines this location spans.
    pub fn line_count(&self) -> usize {
        self.end.line - self.start.line + 1
    }

    /// Returns the byte length of this location.
    pub fn byte_len(&self) -> usize {
        self.byte_end - self.byte_start
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}-{}", self.start, self.end)
    }
}

/// Fingerprint for detecting file changes between scans.
///
/// FileFingerprint captures a content hash and modification time to enable
/// efficient change detection during rescans. If the fingerprint matches,
/// the file hasn't changed and existing artifacts can be kept.
///
/// # Fields
///
/// * `content_hash` - SHA-256 hash of file content (hex string).
/// * `modified_at` - File modification timestamp (Unix epoch seconds).
/// * `size_bytes` - File size in bytes.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::scan_config::FileFingerprint;
/// let fp = FileFingerprint::new(
///     std::string::String::from("abc123..."),
///     1701360000,
///     1024,
/// );
/// std::assert_eq!(fp.size_bytes, 1024);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct FileFingerprint {
    /// SHA-256 hash of file content (hex string, 64 chars).
    pub content_hash: String,

    /// File modification timestamp (Unix epoch seconds).
    pub modified_at: i64,

    /// File size in bytes.
    pub size_bytes: usize,
}

impl FileFingerprint {
    /// Creates a new FileFingerprint.
    ///
    /// # Arguments
    ///
    /// * `content_hash` - SHA-256 hash of content.
    /// * `modified_at` - Unix timestamp of last modification.
    /// * `size_bytes` - File size in bytes.
    pub fn new(content_hash: String, modified_at: i64, size_bytes: usize) -> Self {
        FileFingerprint {
            content_hash,
            modified_at,
            size_bytes,
        }
    }

    /// Computes a fingerprint from file content and metadata.
    ///
    /// Uses a simple hash of the content bytes. In production, this would
    /// use SHA-256 or similar, but for now we use a simple checksum.
    ///
    /// # Arguments
    ///
    /// * `content` - File content as string.
    /// * `modified_at` - Unix timestamp of last modification.
    pub fn from_content(content: &str, modified_at: i64) -> Self {
        // Simple hash: sum of bytes mod 2^64, formatted as hex
        // In production, replace with proper SHA-256
        let hash: u64 = content.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        FileFingerprint {
            content_hash: std::format!("{:016x}", hash),
            modified_at,
            size_bytes: content.len(),
        }
    }

    /// Returns true if this fingerprint matches another (same content).
    ///
    /// Only compares content hash and size; mtime may differ due to copy operations.
    pub fn matches(&self, other: &FileFingerprint) -> bool {
        self.content_hash == other.content_hash && self.size_bytes == other.size_bytes
    }
}

/// A chunk of content with its source location for artifact generation.
///
/// ContentChunk pairs extracted content with its precise location in the
/// source file. This enables updating specific artifacts when source content
/// changes and removing artifacts when their source is deleted.
///
/// # Fields
///
/// * `content` - The text content of this chunk.
/// * `location` - Where this chunk appears in the source file.
/// * `chunk_index` - 0-indexed position in the list of chunks from this file.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::scan_config::{ContentChunk, SourceLocation, SourcePosition};
/// let chunk = ContentChunk {
///     content: std::string::String::from("fn main() {}"),
///     location: SourceLocation::new(
///         SourcePosition::new(1, 1),
///         SourcePosition::new(1, 12),
///         0,
///         12,
///     ),
///     chunk_index: 0,
/// };
/// std::assert_eq!(chunk.chunk_index, 0);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ContentChunk {
    /// The text content of this chunk.
    pub content: String,

    /// Where this chunk appears in the source file.
    pub location: SourceLocation,

    /// 0-indexed position in the list of chunks from this file.
    pub chunk_index: usize,
}

impl ContentChunk {
    /// Creates a new ContentChunk.
    ///
    /// # Arguments
    ///
    /// * `content` - The text content.
    /// * `location` - Source location of the content.
    /// * `chunk_index` - Index of this chunk within the file.
    pub fn new(content: String, location: SourceLocation, chunk_index: usize) -> Self {
        ContentChunk {
            content,
            location,
            chunk_index,
        }
    }

    /// Returns a unique identifier for this chunk within its file.
    ///
    /// Format: "line:col-line:col" which can be used as part of artifact source_id.
    pub fn location_id(&self) -> String {
        self.location.to_string()
    }
}

/// Represents a file discovered during directory scanning.
///
/// ScannedFile contains the path, content, and metadata of a discovered file
/// that passed all filters (extension, size, gitignore). The content is the
/// raw text ready for chunking and embedding.
///
/// # Fields
///
/// * `path` - Relative path from the scan root.
/// * `absolute_path` - Full absolute path to the file.
/// * `content` - Text content of the file.
/// * `extension` - File extension without leading dot.
/// * `size_bytes` - File size in bytes.
/// * `fingerprint` - Content hash and mtime for change detection.
/// * `line_count` - Number of lines in the file.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::scan_config::{ScannedFile, FileFingerprint};
/// let file = ScannedFile {
///     path: std::string::String::from("src/main.rs"),
///     absolute_path: std::string::String::from("/home/user/project/src/main.rs"),
///     content: std::string::String::from("fn main() {}"),
///     extension: std::string::String::from("rs"),
///     size_bytes: 12,
///     fingerprint: FileFingerprint::from_content("fn main() {}", 1701360000),
///     line_count: 1,
/// };
///
/// std::assert_eq!(file.extension, "rs");
/// std::assert_eq!(file.line_count, 1);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ScannedFile {
    /// Relative path from the scan root.
    pub path: String,

    /// Full absolute path to the file.
    pub absolute_path: String,

    /// Text content of the file.
    pub content: String,

    /// File extension without leading dot.
    pub extension: String,

    /// File size in bytes.
    pub size_bytes: usize,

    /// Content hash and mtime for change detection.
    pub fingerprint: FileFingerprint,

    /// Number of lines in the file.
    pub line_count: usize,
}

impl ScannedFile {
    /// Returns true if this file's content matches another file.
    ///
    /// Uses fingerprint comparison for efficient change detection.
    pub fn content_matches(&self, other: &ScannedFile) -> bool {
        self.fingerprint.matches(&other.fingerprint)
    }

    /// Computes a unique source ID for artifacts from this file.
    ///
    /// Format: "file:{relative_path}" for use as artifact source_id.
    pub fn source_id(&self) -> String {
        std::format!("file:{}", self.path)
    }

    /// Computes a source ID for a specific chunk within this file.
    ///
    /// Format: "file:{relative_path}#{location}" for precise artifact tracking.
    ///
    /// # Arguments
    ///
    /// * `location` - The source location of the chunk.
    pub fn chunk_source_id(&self, location: &SourceLocation) -> String {
        std::format!("file:{}#{}", self.path, location)
    }
}

/// Errors that can occur during directory scanning.
///
/// ScanError categorizes failures that may occur when walking directories,
/// reading files, or applying filters.
///
/// # Variants
///
/// * `PathNotFound` - The specified source path does not exist.
/// * `NotADirectory` - The source path is a file, not a directory.
/// * `PermissionDenied` - Insufficient permissions to read path or file.
/// * `IoError` - General I/O error with description.
/// * `EncodingError` - File content is not valid UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    /// The specified source path does not exist.
    PathNotFound(String),

    /// The source path is a file, not a directory.
    NotADirectory(String),

    /// Insufficient permissions to read the path or file.
    PermissionDenied(String),

    /// General I/O error occurred.
    IoError(String),

    /// File content is not valid UTF-8 text.
    EncodingError(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::PathNotFound(path) => {
                std::write!(f, "Path not found: {}", path)
            }
            ScanError::NotADirectory(path) => {
                std::write!(f, "Not a directory: {}", path)
            }
            ScanError::PermissionDenied(path) => {
                std::write!(f, "Permission denied: {}", path)
            }
            ScanError::IoError(msg) => {
                std::write!(f, "I/O error: {}", msg)
            }
            ScanError::EncodingError(path) => {
                std::write!(f, "Encoding error (not UTF-8): {}", path)
            }
        }
    }
}

impl std::error::Error for ScanError {}

/// Statistics from a completed directory scan.
///
/// ScanStats provides metrics about a scan operation including counts of
/// files processed, skipped, and any errors encountered.
///
/// # Fields
///
/// * `files_scanned` - Number of files successfully read.
/// * `files_skipped` - Number of files skipped (wrong extension, too large, etc.).
/// * `directories_visited` - Number of directories traversed.
/// * `total_bytes` - Total bytes of content read.
/// * `duration_ms` - Time taken for the scan in milliseconds.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ScanStats {
    /// Number of files successfully read and returned.
    pub files_scanned: usize,

    /// Number of files skipped due to filters.
    pub files_skipped: usize,

    /// Number of directories visited during traversal.
    pub directories_visited: usize,

    /// Total bytes of content read from all files.
    pub total_bytes: usize,

    /// Time taken for the scan in milliseconds.
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_config_new_has_default_extensions() {
        // Test: Validates new config includes default extensions.
        // Justification: Users expect common file types to be included by default.
        let config = ScanConfig::new(String::from("./src"));

        std::assert!(!config.include_extensions.is_empty());
        std::assert!(config.include_extensions.contains(&String::from("rs")));
        std::assert!(config.include_extensions.contains(&String::from("md")));
        std::assert!(config.include_extensions.contains(&String::from("json")));
    }

    #[test]
    fn test_scan_config_respects_gitignore_by_default() {
        // Test: Validates gitignore is respected by default.
        // Justification: Critical for not indexing build artifacts, node_modules, etc.
        let config = ScanConfig::new(String::from("./"));

        std::assert!(config.respect_gitignore);
    }

    #[test]
    fn test_scan_config_default_max_file_size() {
        // Test: Validates default max file size is 1MB.
        // Justification: Large files should be skipped to avoid memory issues.
        let config = ScanConfig::new(String::from("./"));

        std::assert_eq!(config.max_file_size, 1_048_576);
    }

    #[test]
    fn test_scan_config_code_only() {
        // Test: Validates code_only preset includes only code extensions.
        // Justification: Users may want to scan only source code files.
        let config = ScanConfig::code_only(String::from("./src"));

        std::assert!(config.include_extensions.contains(&String::from("rs")));
        std::assert!(config.include_extensions.contains(&String::from("py")));
        std::assert!(!config.include_extensions.contains(&String::from("md")));
        std::assert!(!config.include_extensions.contains(&String::from("json")));
    }

    #[test]
    fn test_scan_config_docs_only() {
        // Test: Validates docs_only preset includes only documentation extensions.
        // Justification: Users may want to scan only documentation files.
        let config = ScanConfig::docs_only(String::from("./docs"));

        std::assert!(config.include_extensions.contains(&String::from("md")));
        std::assert!(config.include_extensions.contains(&String::from("txt")));
        std::assert!(!config.include_extensions.contains(&String::from("rs")));
    }

    #[test]
    fn test_scan_config_builder_pattern() {
        // Test: Validates builder methods work correctly.
        // Justification: Fluent API should allow method chaining.
        let config = ScanConfig::new(String::from("./"))
            .exclude(String::from("*.log"))
            .with_max_depth(3)
            .with_chunk_strategy(ChunkStrategy::Sentence);

        std::assert_eq!(config.exclude_patterns.len(), 1);
        std::assert_eq!(config.max_depth, std::option::Option::Some(3));
        std::assert!(matches!(config.chunk_strategy, ChunkStrategy::Sentence));
    }

    #[test]
    fn test_chunk_strategy_default() {
        // Test: Validates default chunk strategy is Paragraph.
        // Justification: Paragraph chunking works well for most content.
        let strategy: ChunkStrategy = std::default::Default::default();

        std::assert!(matches!(strategy, ChunkStrategy::Paragraph));
    }

    #[test]
    fn test_scan_error_display() {
        // Test: Validates error messages are user-friendly.
        // Justification: Error messages should clearly explain the problem.
        let err = ScanError::PathNotFound(String::from("/nonexistent"));
        let msg = std::format!("{}", err);

        std::assert!(msg.contains("Path not found"));
        std::assert!(msg.contains("/nonexistent"));
    }

    #[test]
    fn test_scanned_file_creation() {
        // Test: Validates ScannedFile struct creation.
        // Justification: Core data structure for scan results.
        let content = "//! Library";
        let file = ScannedFile {
            path: String::from("src/lib.rs"),
            absolute_path: String::from("/home/user/project/src/lib.rs"),
            content: String::from(content),
            extension: String::from("rs"),
            size_bytes: 11,
            fingerprint: FileFingerprint::from_content(content, 1701360000),
            line_count: 1,
        };

        std::assert_eq!(file.path, "src/lib.rs");
        std::assert_eq!(file.extension, "rs");
        std::assert_eq!(file.size_bytes, 11);
        std::assert_eq!(file.line_count, 1);
        std::assert!(!file.fingerprint.content_hash.is_empty());
    }

    #[test]
    fn test_source_position_display() {
        // Test: Validates SourcePosition formatting as "line:col".
        // Justification: Used for artifact source IDs and logging.
        let pos = SourcePosition::new(10, 5);

        std::assert_eq!(pos.to_string(), "10:5");
        std::assert_eq!(pos.line, 10);
        std::assert_eq!(pos.col, 5);
    }

    #[test]
    fn test_source_position_start() {
        // Test: Validates start position is 1:1.
        // Justification: Files start at line 1, column 1.
        let pos = SourcePosition::start();

        std::assert_eq!(pos.line, 1);
        std::assert_eq!(pos.col, 1);
    }

    #[test]
    fn test_source_location_display() {
        // Test: Validates SourceLocation formatting as "start-end".
        // Justification: Used for artifact source IDs.
        let loc = SourceLocation::new(
            SourcePosition::new(1, 1),
            SourcePosition::new(5, 20),
            0,
            150,
        );

        std::assert_eq!(loc.to_string(), "1:1-5:20");
    }

    #[test]
    fn test_source_location_line_count() {
        // Test: Validates line count calculation.
        // Justification: Used for statistics and display.
        let loc = SourceLocation::new(
            SourcePosition::new(10, 1),
            SourcePosition::new(15, 10),
            100,
            200,
        );

        std::assert_eq!(loc.line_count(), 6); // 10, 11, 12, 13, 14, 15
        std::assert_eq!(loc.byte_len(), 100);
    }

    #[test]
    fn test_source_location_whole_file() {
        // Test: Validates whole_file constructor.
        // Justification: Common case for small files.
        let loc = SourceLocation::whole_file(100, 50, 5000);

        std::assert_eq!(loc.start.line, 1);
        std::assert_eq!(loc.start.col, 1);
        std::assert_eq!(loc.end.line, 100);
        std::assert_eq!(loc.end.col, 50);
        std::assert_eq!(loc.byte_start, 0);
        std::assert_eq!(loc.byte_end, 5000);
    }

    #[test]
    fn test_file_fingerprint_from_content() {
        // Test: Validates fingerprint generation from content.
        // Justification: Core change detection mechanism.
        let fp = FileFingerprint::from_content("hello world", 1701360000);

        std::assert!(!fp.content_hash.is_empty());
        std::assert_eq!(fp.size_bytes, 11);
        std::assert_eq!(fp.modified_at, 1701360000);
    }

    #[test]
    fn test_file_fingerprint_matches() {
        // Test: Validates fingerprint comparison for change detection.
        // Justification: Same content should match regardless of mtime.
        let fp1 = FileFingerprint::from_content("hello world", 1701360000);
        let fp2 = FileFingerprint::from_content("hello world", 1701360001); // different mtime
        let fp3 = FileFingerprint::from_content("hello WORLD", 1701360000); // different content

        std::assert!(fp1.matches(&fp2)); // same content, different mtime
        std::assert!(!fp1.matches(&fp3)); // different content
    }

    #[test]
    fn test_content_chunk_location_id() {
        // Test: Validates chunk location ID generation.
        // Justification: Used for artifact source_id construction.
        let chunk = ContentChunk::new(
            String::from("fn main() {}"),
            SourceLocation::new(
                SourcePosition::new(1, 1),
                SourcePosition::new(1, 12),
                0,
                12,
            ),
            0,
        );

        std::assert_eq!(chunk.location_id(), "1:1-1:12");
    }

    #[test]
    fn test_scanned_file_source_id() {
        // Test: Validates source ID generation for files.
        // Justification: Used for artifact tracking.
        let file = ScannedFile {
            path: String::from("src/main.rs"),
            absolute_path: String::from("/home/user/project/src/main.rs"),
            content: String::from("fn main() {}"),
            extension: String::from("rs"),
            size_bytes: 12,
            fingerprint: FileFingerprint::from_content("fn main() {}", 1701360000),
            line_count: 1,
        };

        std::assert_eq!(file.source_id(), "file:src/main.rs");
    }

    #[test]
    fn test_scanned_file_chunk_source_id() {
        // Test: Validates chunk source ID includes location.
        // Justification: Enables precise artifact tracking during rescans.
        let file = ScannedFile {
            path: String::from("src/main.rs"),
            absolute_path: String::from("/home/user/project/src/main.rs"),
            content: String::from("fn main() {}"),
            extension: String::from("rs"),
            size_bytes: 12,
            fingerprint: FileFingerprint::from_content("fn main() {}", 1701360000),
            line_count: 1,
        };

        let loc = SourceLocation::new(
            SourcePosition::new(1, 1),
            SourcePosition::new(1, 12),
            0,
            12,
        );

        std::assert_eq!(file.chunk_source_id(&loc), "file:src/main.rs#1:1-1:12");
    }

    #[test]
    fn test_scanned_file_content_matches() {
        // Test: Validates content comparison via fingerprint.
        // Justification: Core rescan change detection.
        let file1 = ScannedFile {
            path: String::from("src/main.rs"),
            absolute_path: String::from("/home/user/project/src/main.rs"),
            content: String::from("fn main() {}"),
            extension: String::from("rs"),
            size_bytes: 12,
            fingerprint: FileFingerprint::from_content("fn main() {}", 1701360000),
            line_count: 1,
        };

        let file2 = ScannedFile {
            fingerprint: FileFingerprint::from_content("fn main() {}", 1701360001),
            ..file1.clone()
        };

        let file3 = ScannedFile {
            content: String::from("fn main() { println!(); }"),
            fingerprint: FileFingerprint::from_content("fn main() { println!(); }", 1701360000),
            ..file1.clone()
        };

        std::assert!(file1.content_matches(&file2)); // same content
        std::assert!(!file1.content_matches(&file3)); // different content
    }

    #[test]
    fn test_scan_stats_default() {
        // Test: Validates ScanStats default values are zero.
        // Justification: Stats should start at zero before counting.
        let stats: ScanStats = std::default::Default::default();

        std::assert_eq!(stats.files_scanned, 0);
        std::assert_eq!(stats.files_skipped, 0);
        std::assert_eq!(stats.directories_visited, 0);
    }
}
