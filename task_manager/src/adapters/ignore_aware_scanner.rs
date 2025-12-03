//! Gitignore-aware directory scanner adapter using the ignore crate.
//!
//! This adapter implements DirectoryScannerPort using the `ignore` crate to walk
//! directories while respecting .gitignore, .git/info/exclude, and global gitignore
//! patterns. It supports filtering by file extension, size limits, and produces
//! fingerprints for incremental scanning.
//!
//! Revision History
//! - 2025-11-30T19:30:00Z @AI: Initial IgnoreAwareScanner adapter for Phase 2 artifact generator.

/// Gitignore-aware directory scanner using the ignore crate.
///
/// IgnoreAwareScanner walks directories respecting all standard gitignore
/// patterns including .gitignore files, .git/info/exclude, and global
/// gitignore configuration. It filters files by extension and size,
/// computes fingerprints for change detection, and tracks line/column
/// locations for chunked content.
///
/// # Examples
///
/// ```ignore
/// # use task_manager::adapters::ignore_aware_scanner::IgnoreAwareScanner;
/// # use task_manager::domain::scan_config::ScanConfig;
/// # use task_manager::ports::directory_scanner_port::DirectoryScannerPort;
/// # async fn example() {
/// let scanner = IgnoreAwareScanner::new();
/// let config = ScanConfig::new(std::string::String::from("./src"));
/// let result = scanner.scan(&config).await.unwrap();
/// println!("Found {} files", result.files.len());
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct IgnoreAwareScanner;

impl IgnoreAwareScanner {
    /// Creates a new IgnoreAwareScanner.
    pub fn new() -> Self {
        IgnoreAwareScanner
    }

    /// Checks if a file extension is in the allowed list.
    fn is_extension_allowed(
        extension: &str,
        allowed: &[String],
    ) -> bool {
        if allowed.is_empty() {
            return true; // No filter means allow all
        }
        allowed.iter().any(|ext| ext.eq_ignore_ascii_case(extension))
    }

    /// Checks if content appears to be binary (contains null bytes).
    fn is_binary_content(content: &[u8]) -> bool {
        // Check first 8KB for null bytes (common binary indicator)
        let check_len = std::cmp::min(content.len(), 8192);
        content[..check_len].contains(&0)
    }

    /// Counts lines in content and returns (line_count, last_line_length).
    fn count_lines(content: &str) -> (usize, usize) {
        if content.is_empty() {
            return (0, 0);
        }

        let lines: std::vec::Vec<&str> = content.lines().collect();
        let line_count = if content.ends_with('\n') {
            lines.len()
        } else {
            lines.len()
        };

        let last_line_len = lines.last().map(|s| s.len()).unwrap_or(0);
        (std::cmp::max(1, line_count), last_line_len)
    }

    /// Gets the modification time of a file as Unix timestamp.
    fn get_mtime(path: &std::path::Path) -> i64 {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }
}

#[async_trait::async_trait]
impl crate::ports::directory_scanner_port::DirectoryScannerPort for IgnoreAwareScanner {
    async fn scan(
        &self,
        config: &crate::domain::scan_config::ScanConfig,
    ) -> std::result::Result<
        crate::ports::directory_scanner_port::ScanResult,
        crate::domain::scan_config::ScanError,
    > {
        let start_time = std::time::Instant::now();
        let source_path = std::path::Path::new(&config.source_path);

        // Validate source path
        if !source_path.exists() {
            return std::result::Result::Err(crate::domain::scan_config::ScanError::PathNotFound(
                config.source_path.clone(),
            ));
        }

        if !source_path.is_dir() {
            return std::result::Result::Err(crate::domain::scan_config::ScanError::NotADirectory(
                config.source_path.clone(),
            ));
        }

        let mut result = crate::ports::directory_scanner_port::ScanResult::new();

        // Build the walker with gitignore support
        let mut builder = ignore::WalkBuilder::new(source_path);

        // Configure gitignore handling
        if config.respect_gitignore {
            builder.git_ignore(true);
            builder.git_global(true);
            builder.git_exclude(true);
        } else {
            builder.git_ignore(false);
            builder.git_global(false);
            builder.git_exclude(false);
        }

        // Set max depth if specified
        if let std::option::Option::Some(depth) = config.max_depth {
            builder.max_depth(std::option::Option::Some(depth + 1)); // +1 because root is depth 0
        }

        // Add custom ignore patterns
        for pattern in &config.exclude_patterns {
            let mut override_builder = ignore::overrides::OverrideBuilder::new(source_path);
            // Negate the pattern to exclude it
            if let std::result::Result::Ok(_) = override_builder.add(&std::format!("!{}", pattern)) {
                if let std::result::Result::Ok(overrides) = override_builder.build() {
                    builder.overrides(overrides);
                }
            }
        }

        // Walk the directory
        let walker = builder.build();

        for entry_result in walker {
            match entry_result {
                std::result::Result::Ok(entry) => {
                    let path = entry.path();

                    // Skip directories
                    if path.is_dir() {
                        result.stats.directories_visited += 1;
                        continue;
                    }

                    // Get file extension
                    let extension = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_string();

                    // Filter by extension
                    if !Self::is_extension_allowed(&extension, &config.include_extensions) {
                        result.stats.files_skipped += 1;
                        continue;
                    }

                    // Check file size
                    let metadata = match std::fs::metadata(path) {
                        std::result::Result::Ok(m) => m,
                        std::result::Result::Err(e) => {
                            result.errors.push(
                                crate::ports::directory_scanner_port::ScanFileError::new(
                                    path.display().to_string(),
                                    std::format!("Failed to read metadata: {}", e),
                                ),
                            );
                            continue;
                        }
                    };

                    let size_bytes = metadata.len() as usize;
                    if size_bytes > config.max_file_size {
                        result.stats.files_skipped += 1;
                        continue;
                    }

                    // Read file content
                    let content_bytes = match std::fs::read(path) {
                        std::result::Result::Ok(bytes) => bytes,
                        std::result::Result::Err(e) => {
                            result.errors.push(
                                crate::ports::directory_scanner_port::ScanFileError::new(
                                    path.display().to_string(),
                                    std::format!("Failed to read file: {}", e),
                                ),
                            );
                            continue;
                        }
                    };

                    // Skip binary files
                    if Self::is_binary_content(&content_bytes) {
                        result.stats.files_skipped += 1;
                        continue;
                    }

                    // Convert to string
                    let content = match String::from_utf8(content_bytes) {
                        std::result::Result::Ok(s) => s,
                        std::result::Result::Err(_) => {
                            result.errors.push(
                                crate::ports::directory_scanner_port::ScanFileError::new(
                                    path.display().to_string(),
                                    String::from("File is not valid UTF-8"),
                                ),
                            );
                            continue;
                        }
                    };

                    // Compute fingerprint
                    let mtime = Self::get_mtime(path);
                    let fingerprint =
                        crate::domain::scan_config::FileFingerprint::from_content(&content, mtime);

                    // Count lines
                    let (line_count, _) = Self::count_lines(&content);

                    // Build relative path
                    let relative_path = path
                        .strip_prefix(source_path)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| path.display().to_string());

                    let scanned_file = crate::domain::scan_config::ScannedFile {
                        path: relative_path,
                        absolute_path: path.display().to_string(),
                        content: content.clone(),
                        extension,
                        size_bytes,
                        fingerprint,
                        line_count,
                    };

                    result.stats.total_bytes += size_bytes;
                    result.stats.files_scanned += 1;
                    result.files.push(scanned_file);
                }
                std::result::Result::Err(e) => {
                    // Non-fatal error during walking
                    result.errors.push(
                        crate::ports::directory_scanner_port::ScanFileError::new(
                            String::from("unknown"),
                            std::format!("Walk error: {}", e),
                        ),
                    );
                }
            }
        }

        result.stats.duration_ms = start_time.elapsed().as_millis() as u64;

        std::result::Result::Ok(result)
    }

    async fn read_file(
        &self,
        path: &std::path::Path,
        config: &crate::domain::scan_config::ScanConfig,
    ) -> std::result::Result<
        std::option::Option<crate::domain::scan_config::ScannedFile>,
        crate::domain::scan_config::ScanError,
    > {
        // Check path exists
        if !path.exists() {
            return std::result::Result::Err(crate::domain::scan_config::ScanError::PathNotFound(
                path.display().to_string(),
            ));
        }

        // Get extension
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        // Filter by extension
        if !Self::is_extension_allowed(&extension, &config.include_extensions) {
            return std::result::Result::Ok(std::option::Option::None);
        }

        // Check size
        let metadata = std::fs::metadata(path).map_err(|e| {
            crate::domain::scan_config::ScanError::IoError(std::format!(
                "Failed to read metadata: {}",
                e
            ))
        })?;

        let size_bytes = metadata.len() as usize;
        if size_bytes > config.max_file_size {
            return std::result::Result::Ok(std::option::Option::None);
        }

        // Read content
        let content_bytes = std::fs::read(path).map_err(|e| {
            crate::domain::scan_config::ScanError::IoError(std::format!(
                "Failed to read file: {}",
                e
            ))
        })?;

        // Skip binary
        if Self::is_binary_content(&content_bytes) {
            return std::result::Result::Ok(std::option::Option::None);
        }

        // Convert to string
        let content = String::from_utf8(content_bytes).map_err(|_| {
            crate::domain::scan_config::ScanError::EncodingError(path.display().to_string())
        })?;

        // Compute fingerprint
        let mtime = Self::get_mtime(path);
        let fingerprint =
            crate::domain::scan_config::FileFingerprint::from_content(&content, mtime);

        // Count lines
        let (line_count, _) = Self::count_lines(&content);

        // Build relative path from config source
        let source_path = std::path::Path::new(&config.source_path);
        let relative_path = path
            .strip_prefix(source_path)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string());

        std::result::Result::Ok(std::option::Option::Some(
            crate::domain::scan_config::ScannedFile {
                path: relative_path,
                absolute_path: path.display().to_string(),
                content,
                extension,
                size_bytes,
                fingerprint,
                line_count,
            },
        ))
    }

    async fn has_file_changed(
        &self,
        path: &std::path::Path,
        previous: &crate::domain::scan_config::FileFingerprint,
    ) -> std::result::Result<bool, crate::domain::scan_config::ScanError> {
        if !path.exists() {
            // File was deleted
            return std::result::Result::Ok(true);
        }

        // Quick check: compare mtime and size first
        let metadata = std::fs::metadata(path).map_err(|e| {
            crate::domain::scan_config::ScanError::IoError(std::format!(
                "Failed to read metadata: {}",
                e
            ))
        })?;

        let size = metadata.len() as usize;
        if size != previous.size_bytes {
            return std::result::Result::Ok(true);
        }

        let mtime = Self::get_mtime(path);
        if mtime != previous.modified_at {
            // Mtime changed, need to check content hash
            let content = std::fs::read_to_string(path).map_err(|e| {
                crate::domain::scan_config::ScanError::IoError(std::format!(
                    "Failed to read file: {}",
                    e
                ))
            })?;

            let current = crate::domain::scan_config::FileFingerprint::from_content(&content, mtime);
            return std::result::Result::Ok(!current.matches(previous));
        }

        // Same size and mtime, assume unchanged
        std::result::Result::Ok(false)
    }

    async fn find_deleted_files(
        &self,
        config: &crate::domain::scan_config::ScanConfig,
        previous_paths: &[String],
    ) -> std::result::Result<std::vec::Vec<String>, crate::domain::scan_config::ScanError> {
        let source_path = std::path::Path::new(&config.source_path);

        if !source_path.exists() {
            return std::result::Result::Err(crate::domain::scan_config::ScanError::PathNotFound(
                config.source_path.clone(),
            ));
        }

        let mut deleted = std::vec::Vec::new();

        for path_str in previous_paths {
            let full_path = source_path.join(path_str);
            if !full_path.exists() {
                deleted.push(path_str.clone());
            }
        }

        std::result::Result::Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::directory_scanner_port::DirectoryScannerPort;

    #[test]
    fn test_is_extension_allowed_empty_filter() {
        // Test: Empty filter allows all extensions.
        // Justification: No filter means include everything.
        std::assert!(IgnoreAwareScanner::is_extension_allowed("rs", &[]));
        std::assert!(IgnoreAwareScanner::is_extension_allowed("txt", &[]));
    }

    #[test]
    fn test_is_extension_allowed_with_filter() {
        // Test: Filter correctly includes/excludes extensions.
        // Justification: Core filtering logic.
        let allowed = std::vec![String::from("rs"), String::from("md")];

        std::assert!(IgnoreAwareScanner::is_extension_allowed("rs", &allowed));
        std::assert!(IgnoreAwareScanner::is_extension_allowed("md", &allowed));
        std::assert!(!IgnoreAwareScanner::is_extension_allowed("txt", &allowed));
    }

    #[test]
    fn test_is_extension_allowed_case_insensitive() {
        // Test: Extension matching is case-insensitive.
        // Justification: Windows uses mixed-case extensions.
        let allowed = std::vec![String::from("rs")];

        std::assert!(IgnoreAwareScanner::is_extension_allowed("RS", &allowed));
        std::assert!(IgnoreAwareScanner::is_extension_allowed("Rs", &allowed));
    }

    #[test]
    fn test_is_binary_content_text() {
        // Test: Text content is not detected as binary.
        // Justification: Text files should pass through.
        let text = b"fn main() { println!(\"Hello\"); }";

        std::assert!(!IgnoreAwareScanner::is_binary_content(text));
    }

    #[test]
    fn test_is_binary_content_with_null() {
        // Test: Content with null bytes is detected as binary.
        // Justification: Null bytes indicate binary data.
        let binary = b"some\x00binary\x00data";

        std::assert!(IgnoreAwareScanner::is_binary_content(binary));
    }

    #[test]
    fn test_count_lines_empty() {
        // Test: Empty string has 0 lines.
        // Justification: Edge case handling.
        let (count, _) = IgnoreAwareScanner::count_lines("");

        std::assert_eq!(count, 0);
    }

    #[test]
    fn test_count_lines_single_line() {
        // Test: Single line without newline.
        // Justification: Common case for short files.
        let (count, last_len) = IgnoreAwareScanner::count_lines("fn main() {}");

        std::assert_eq!(count, 1);
        std::assert_eq!(last_len, 12);
    }

    #[test]
    fn test_count_lines_multiple_lines() {
        // Test: Multiple lines with trailing newline.
        // Justification: Standard file format.
        let content = "line 1\nline 2\nline 3\n";
        let (count, _) = IgnoreAwareScanner::count_lines(content);

        std::assert_eq!(count, 3);
    }

    #[test]
    fn test_scanner_new() {
        // Test: Scanner can be created.
        // Justification: Basic construction test.
        let scanner = IgnoreAwareScanner::new();

        // Scanner has no state to check, but this ensures it compiles
        std::assert!(std::format!("{:?}", scanner).contains("IgnoreAwareScanner"));
    }

    #[tokio::test]
    async fn test_scan_nonexistent_path() {
        // Test: Scanning nonexistent path returns PathNotFound.
        // Justification: Error handling for invalid input.
        let scanner = IgnoreAwareScanner::new();
        let config = crate::domain::scan_config::ScanConfig::new(
            String::from("/nonexistent/path/that/does/not/exist"),
        );

        let result = scanner.scan(&config).await;

        std::assert!(matches!(
            result,
            std::result::Result::Err(crate::domain::scan_config::ScanError::PathNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_find_deleted_files_none_deleted() {
        // Test: No files deleted returns empty vec.
        // Justification: Common case during incremental scan.
        let scanner = IgnoreAwareScanner::new();
        let config = crate::domain::scan_config::ScanConfig::new(String::from("."));
        let previous = std::vec![String::from("Cargo.toml")]; // This file exists

        let deleted = scanner.find_deleted_files(&config, &previous).await.unwrap();

        std::assert!(deleted.is_empty());
    }

    #[tokio::test]
    async fn test_find_deleted_files_some_deleted() {
        // Test: Deleted files are detected.
        // Justification: Core incremental update functionality.
        let scanner = IgnoreAwareScanner::new();
        let config = crate::domain::scan_config::ScanConfig::new(String::from("."));
        let previous = std::vec![
            String::from("Cargo.toml"),                // exists
            String::from("nonexistent_file_12345.rs"), // doesn't exist
        ];

        let deleted = scanner.find_deleted_files(&config, &previous).await.unwrap();

        std::assert_eq!(deleted.len(), 1);
        std::assert!(deleted.contains(&String::from("nonexistent_file_12345.rs")));
    }
}
