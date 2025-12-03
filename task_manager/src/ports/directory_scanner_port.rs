//! Port interface for directory scanning operations.
//!
//! This module defines the DirectoryScannerPort trait for scanning directories
//! while respecting .gitignore patterns. The port enables hexagonal architecture
//! by abstracting the file system scanning implementation, allowing different
//! adapters (e.g., ignore crate, custom walker) to be swapped.
//!
//! Revision History
//! - 2025-11-30T19:20:00Z @AI: Initial DirectoryScannerPort for Phase 2 artifact generator.

/// Port trait for directory scanning with gitignore support.
///
/// DirectoryScannerPort defines the interface for scanning directories to discover
/// files that should be processed for artifact generation. Implementations must
/// respect .gitignore patterns and support filtering by extension and size.
///
/// # Design Notes
///
/// This is a "driven" port in hexagonal architecture terms - the application core
/// drives calls to this port, and adapters implement it to interact with the file
/// system. The trait is async to support non-blocking I/O in async runtimes.
///
/// # Examples
///
/// ```ignore
/// # use task_manager::ports::directory_scanner_port::DirectoryScannerPort;
/// # use task_manager::domain::scan_config::ScanConfig;
/// async fn example<S: DirectoryScannerPort>(scanner: &S) {
///     let config = ScanConfig::new(std::string::String::from("./src"));
///     let result = scanner.scan(&config).await;
///     match result {
///         std::result::Result::Ok(files) => println!("Found {} files", files.len()),
///         std::result::Result::Err(e) => eprintln!("Scan failed: {:?}", e),
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait DirectoryScannerPort: std::marker::Send + std::marker::Sync {
    /// Scans a directory according to the provided configuration.
    ///
    /// Walks the directory tree starting from `config.source_path`, respecting
    /// .gitignore patterns (if enabled), filtering by extension, and skipping
    /// files that exceed the size limit.
    ///
    /// # Arguments
    ///
    /// * `config` - Scan configuration specifying path, filters, and options.
    ///
    /// # Returns
    ///
    /// * `Ok(ScanResult)` - Successfully scanned files and statistics.
    /// * `Err(ScanError)` - Scan failed (path not found, permission denied, etc.).
    ///
    /// # Errors
    ///
    /// Returns `ScanError::PathNotFound` if the source path does not exist.
    /// Returns `ScanError::NotADirectory` if the source path is a file.
    /// Returns `ScanError::PermissionDenied` if the directory cannot be read.
    async fn scan(
        &self,
        config: &crate::domain::scan_config::ScanConfig,
    ) -> std::result::Result<ScanResult, crate::domain::scan_config::ScanError>;

    /// Reads a single file and returns its content with metadata.
    ///
    /// This method is useful for re-scanning individual files during incremental
    /// updates. The file must pass the same filters that would apply during a
    /// full scan (extension, size).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read.
    /// * `config` - Scan configuration for applying filters.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ScannedFile))` - File read successfully.
    /// * `Ok(None)` - File exists but was filtered out (wrong extension, too large).
    /// * `Err(ScanError)` - Failed to read file.
    async fn read_file(
        &self,
        path: &std::path::Path,
        config: &crate::domain::scan_config::ScanConfig,
    ) -> std::result::Result<
        std::option::Option<crate::domain::scan_config::ScannedFile>,
        crate::domain::scan_config::ScanError,
    >;

    /// Checks if a file has changed since the last scan.
    ///
    /// Compares the current file's fingerprint against the provided previous
    /// fingerprint. This enables efficient incremental scanning by skipping
    /// unchanged files.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to check.
    /// * `previous` - Fingerprint from the previous scan.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - File has changed (different content or mtime).
    /// * `Ok(false)` - File is unchanged.
    /// * `Err(ScanError)` - Failed to read file metadata.
    async fn has_file_changed(
        &self,
        path: &std::path::Path,
        previous: &crate::domain::scan_config::FileFingerprint,
    ) -> std::result::Result<bool, crate::domain::scan_config::ScanError>;

    /// Lists files that have been deleted since the last scan.
    ///
    /// Compares the list of previously scanned paths against the current directory
    /// state to identify files that no longer exist. This enables cleanup of
    /// artifacts from deleted files.
    ///
    /// # Arguments
    ///
    /// * `config` - Scan configuration specifying the source path.
    /// * `previous_paths` - Paths of files from the previous scan.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - Paths that no longer exist.
    /// * `Err(ScanError)` - Failed to check paths.
    async fn find_deleted_files(
        &self,
        config: &crate::domain::scan_config::ScanConfig,
        previous_paths: &[String],
    ) -> std::result::Result<std::vec::Vec<String>, crate::domain::scan_config::ScanError>;
}

/// Result of a directory scan operation.
///
/// ScanResult contains all discovered files along with statistics about the
/// scan operation. This enables batch processing and progress reporting.
///
/// # Fields
///
/// * `files` - Successfully scanned files ready for artifact generation.
/// * `stats` - Statistics about the scan operation.
/// * `errors` - Non-fatal errors encountered during scanning.
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Successfully scanned files.
    pub files: std::vec::Vec<crate::domain::scan_config::ScannedFile>,

    /// Statistics about the scan operation.
    pub stats: crate::domain::scan_config::ScanStats,

    /// Non-fatal errors encountered during scanning (e.g., permission denied on specific files).
    pub errors: std::vec::Vec<ScanFileError>,
}

impl ScanResult {
    /// Creates a new empty ScanResult.
    pub fn new() -> Self {
        ScanResult {
            files: std::vec::Vec::new(),
            stats: crate::domain::scan_config::ScanStats::default(),
            errors: std::vec::Vec::new(),
        }
    }

    /// Returns true if no files were found.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the number of files found.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Returns true if errors occurred during scanning.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl std::default::Default for ScanResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Non-fatal error for a specific file during scanning.
///
/// ScanFileError captures errors that don't stop the entire scan but
/// prevent processing of a specific file.
#[derive(Debug, Clone)]
pub struct ScanFileError {
    /// Path to the file that caused the error.
    pub path: String,

    /// Description of the error.
    pub message: String,
}

impl ScanFileError {
    /// Creates a new ScanFileError.
    pub fn new(path: String, message: String) -> Self {
        ScanFileError { path, message }
    }
}

impl std::fmt::Display for ScanFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}: {}", self.path, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_result_new_is_empty() {
        // Test: Validates new ScanResult is empty.
        // Justification: Fresh result should have no files.
        let result = ScanResult::new();

        std::assert!(result.is_empty());
        std::assert_eq!(result.file_count(), 0);
        std::assert!(!result.has_errors());
    }

    #[test]
    fn test_scan_result_default() {
        // Test: Validates Default trait implementation.
        // Justification: Default should match new().
        let result: ScanResult = std::default::Default::default();

        std::assert!(result.is_empty());
    }

    #[test]
    fn test_scan_file_error_display() {
        // Test: Validates error display format.
        // Justification: Error messages should be readable.
        let err = ScanFileError::new(
            String::from("src/secret.rs"),
            String::from("Permission denied"),
        );

        let msg = std::format!("{}", err);
        std::assert!(msg.contains("src/secret.rs"));
        std::assert!(msg.contains("Permission denied"));
    }
}
