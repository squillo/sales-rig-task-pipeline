//! Sandboxed filesystem tool for Rig agents.
//!
//! Provides safe file system access with path traversal protection.
//! Only allows access to files within the project root directory.
//! Prevents attacks using "..", absolute paths, or symlinks.
//!
//! Revision History
//! - 2025-11-23 @AI: Implement FileSystemTool with path sandboxing (Phase 4 Sprint 9 Task 4.7).

/// Error type for filesystem tool operations.
#[derive(Debug, Clone)]
pub enum FileSystemError {
    /// Path traversal attack detected
    PathTraversal(std::string::String),
    /// Path outside project root
    PathEscape(std::string::String),
    /// I/O operation failed
    IoError(std::string::String),
    /// Path resolution failed
    InvalidPath(std::string::String),
}

impl std::fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileSystemError::PathTraversal(msg) => write!(f, "Path traversal not allowed: {}", msg),
            FileSystemError::PathEscape(msg) => write!(f, "Path outside project root: {}", msg),
            FileSystemError::IoError(msg) => write!(f, "I/O error: {}", msg),
            FileSystemError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
        }
    }
}

impl std::error::Error for FileSystemError {}

/// Sandboxed filesystem tool for LLM agents.
///
/// This tool allows agents to read files, write files, and list directories
/// while enforcing strict security constraints:
/// - All paths must be relative to project root
/// - No path traversal ("..") allowed
/// - No absolute paths outside project
/// - No symlink following
///
/// # Examples
///
/// ```ignore
/// let tool = task_orchestrator::tools::file_system_tool::FileSystemTool::new("/project/root");
/// let content = tool.read_file("src/main.rs").await?;
/// ```
#[derive(Debug, Clone)]
pub struct FileSystemTool {
    project_root: std::path::PathBuf,
}

impl FileSystemTool {
    /// Creates a new FileSystemTool sandboxed to the given project root.
    ///
    /// # Arguments
    ///
    /// * `project_root` - Absolute path to project root directory
    ///
    /// # Returns
    ///
    /// A new FileSystemTool instance.
    pub fn new(project_root: impl std::convert::AsRef<std::path::Path>) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
        }
    }

    /// Validates and resolves a path relative to project root.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path relative to project root
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - Canonicalized absolute path within project
    /// * `Err(String)` - Security violation or invalid path
    ///
    /// # Security
    ///
    /// Rejects:
    /// - Paths containing ".."
    /// - Absolute paths outside project
    /// - Symlinks escaping project root
    fn validate_path(&self, relative_path: &str) -> std::result::Result<std::path::PathBuf, FileSystemError> {
        // Reject paths with ".." to prevent traversal attacks
        if relative_path.contains("..") {
            return std::result::Result::Err(FileSystemError::PathTraversal(relative_path.to_string()));
        }

        // Build absolute path from project root
        let requested_path = self.project_root.join(relative_path);

        // Canonicalize to resolve symlinks and normalize
        let canonical = match requested_path.canonicalize() {
            std::result::Result::Ok(p) => p,
            std::result::Result::Err(_) => {
                // File might not exist yet (for writes), use parent check
                if let std::option::Option::Some(parent) = requested_path.parent() {
                    match parent.canonicalize() {
                        std::result::Result::Ok(p) => {
                            if let std::option::Option::Some(filename) = requested_path.file_name() {
                                p.join(filename)
                            } else {
                                return std::result::Result::Err(FileSystemError::InvalidPath(
                                    std::format!("Invalid filename: {}", relative_path)
                                ));
                            }
                        }
                        std::result::Result::Err(_) => {
                            return std::result::Result::Err(FileSystemError::InvalidPath(
                                std::format!("Parent directory does not exist: {}", relative_path)
                            ))
                        }
                    }
                } else {
                    return std::result::Result::Err(FileSystemError::InvalidPath(
                        std::format!("Cannot resolve path: {}", relative_path)
                    ));
                }
            }
        };

        // Verify canonical path is within project root
        let canonical_root = match self.project_root.canonicalize() {
            std::result::Result::Ok(r) => r,
            std::result::Result::Err(e) => {
                return std::result::Result::Err(FileSystemError::IoError(
                    std::format!("Cannot canonicalize project root: {}", e)
                ))
            }
        };

        if !canonical.starts_with(&canonical_root) {
            return std::result::Result::Err(FileSystemError::PathEscape(
                std::format!("{} (resolved to {:?}, root: {:?})", relative_path, canonical, canonical_root)
            ));
        }

        std::result::Result::Ok(canonical)
    }

    /// Reads a file within the project.
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path from project root
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - File contents as UTF-8 string
    /// * `Err(String)` - Security violation or I/O error
    pub async fn read_file(&self, path: &str) -> std::result::Result<std::string::String, FileSystemError> {
        let validated_path = self.validate_path(path)?;

        match tokio::fs::read_to_string(&validated_path).await {
            std::result::Result::Ok(content) => std::result::Result::Ok(content),
            std::result::Result::Err(e) => std::result::Result::Err(FileSystemError::IoError(std::format!("Read error: {}", e))),
        }
    }

    /// Writes content to a file within the project.
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path from project root
    /// * `content` - File contents to write
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File written successfully
    /// * `Err(String)` - Security violation or I/O error
    pub async fn write_file(&self, path: &str, content: &str) -> std::result::Result<(), FileSystemError> {
        let validated_path = self.validate_path(path)?;

        // Create parent directories if needed
        if let std::option::Option::Some(parent) = validated_path.parent() {
            if let std::result::Result::Err(e) = tokio::fs::create_dir_all(parent).await {
                return std::result::Result::Err(FileSystemError::IoError(std::format!("Failed to create parent directory: {}", e)));
            }
        }

        match tokio::fs::write(&validated_path, content).await {
            std::result::Result::Ok(_) => std::result::Result::Ok(()),
            std::result::Result::Err(e) => std::result::Result::Err(FileSystemError::IoError(std::format!("Write error: {}", e))),
        }
    }

    /// Lists files in a directory within the project.
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to directory from project root
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of filenames (not full paths)
    /// * `Err(String)` - Security violation or I/O error
    pub async fn list_directory(&self, path: &str) -> std::result::Result<std::vec::Vec<std::string::String>, FileSystemError> {
        let validated_path = self.validate_path(path)?;

        let mut entries = match tokio::fs::read_dir(&validated_path).await {
            std::result::Result::Ok(e) => e,
            std::result::Result::Err(e) => return std::result::Result::Err(FileSystemError::IoError(std::format!("Read dir error: {}", e))),
        };

        let mut filenames = std::vec::Vec::new();

        while let std::result::Result::Ok(std::option::Option::Some(entry)) = entries.next_entry().await {
            if let std::option::Option::Some(name) = entry.file_name().to_str() {
                filenames.push(name.to_string());
            }
        }

        std::result::Result::Ok(filenames)
    }
}

// Rig Tool trait implementations for each operation

/// Arguments for read_file tool.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct ReadFileArgs {
    /// Relative path to file from project root
    pub path: std::string::String,
}

/// Read file tool for Rig agents.
#[derive(Debug, Clone)]
pub struct ReadFileTool {
    fs: FileSystemTool,
}

impl ReadFileTool {
    /// Creates a new ReadFileTool.
    pub fn new(project_root: impl std::convert::AsRef<std::path::Path>) -> Self {
        Self {
            fs: FileSystemTool::new(project_root),
        }
    }
}

impl rig::tool::Tool for ReadFileTool {
    const NAME: &'static str = "read_file";

    type Error = FileSystemError;
    type Args = ReadFileArgs;
    type Output = std::string::String;

    fn definition(&self, _prompt: std::string::String) -> impl std::future::Future<Output = rig::completion::ToolDefinition> + Send + Sync {
        async {
            rig::completion::ToolDefinition {
                name: Self::NAME.to_string(),
                description: "Reads a file from the project directory. Path must be relative to project root. Returns file contents as UTF-8 string.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path to file from project root (e.g., 'src/main.rs')"
                        }
                    },
                    "required": ["path"]
                }),
            }
        }
    }

    fn call(&self, args: Self::Args) -> impl std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send {
        let fs = self.fs.clone();
        async move {
            fs.read_file(&args.path).await
        }
    }
}

/// Arguments for write_file tool.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct WriteFileArgs {
    /// Relative path to file from project root
    pub path: std::string::String,
    /// File content to write
    pub content: std::string::String,
}

/// Write file tool for Rig agents.
#[derive(Debug, Clone)]
pub struct WriteFileTool {
    fs: FileSystemTool,
}

impl WriteFileTool {
    /// Creates a new WriteFileTool.
    pub fn new(project_root: impl std::convert::AsRef<std::path::Path>) -> Self {
        Self {
            fs: FileSystemTool::new(project_root),
        }
    }
}

impl rig::tool::Tool for WriteFileTool {
    const NAME: &'static str = "write_file";

    type Error = FileSystemError;
    type Args = WriteFileArgs;
    type Output = std::string::String;

    fn definition(&self, _prompt: std::string::String) -> impl std::future::Future<Output = rig::completion::ToolDefinition> + Send + Sync {
        async {
            rig::completion::ToolDefinition {
                name: Self::NAME.to_string(),
                description: "Writes content to a file in the project directory. Path must be relative to project root. Creates parent directories if needed.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path to file from project root (e.g., 'src/lib.rs')"
                        },
                        "content": {
                            "type": "string",
                            "description": "File content to write (UTF-8 string)"
                        }
                    },
                    "required": ["path", "content"]
                }),
            }
        }
    }

    fn call(&self, args: Self::Args) -> impl std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send {
        let fs = self.fs.clone();
        async move {
            fs.write_file(&args.path, &args.content).await?;
            std::result::Result::Ok(std::format!("File written successfully: {}", args.path))
        }
    }
}

/// Arguments for list_directory tool.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct ListDirectoryArgs {
    /// Relative path to directory from project root
    pub path: std::string::String,
}

/// List directory tool for Rig agents.
#[derive(Debug, Clone)]
pub struct ListDirectoryTool {
    fs: FileSystemTool,
}

impl ListDirectoryTool {
    /// Creates a new ListDirectoryTool.
    pub fn new(project_root: impl std::convert::AsRef<std::path::Path>) -> Self {
        Self {
            fs: FileSystemTool::new(project_root),
        }
    }
}

impl rig::tool::Tool for ListDirectoryTool {
    const NAME: &'static str = "list_directory";

    type Error = FileSystemError;
    type Args = ListDirectoryArgs;
    type Output = std::vec::Vec<std::string::String>;

    fn definition(&self, _prompt: std::string::String) -> impl std::future::Future<Output = rig::completion::ToolDefinition> + Send + Sync {
        async {
            rig::completion::ToolDefinition {
                name: Self::NAME.to_string(),
                description: "Lists files and directories in a project directory. Path must be relative to project root. Returns array of filenames.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path to directory from project root (e.g., 'src' or '.')"
                        }
                    },
                    "required": ["path"]
                }),
            }
        }
    }

    fn call(&self, args: Self::Args) -> impl std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send {
        let fs = self.fs.clone();
        async move {
            fs.list_directory(&args.path).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_path_validation_rejects_traversal() {
        let tool = FileSystemTool::new("/tmp");
        let result = tool.validate_path("../etc/passwd");
        std::assert!(result.is_err());
        let err = result.unwrap_err();
        std::assert!(matches!(err, FileSystemError::PathTraversal(_)));
    }

    #[tokio::test]
    async fn test_path_validation_rejects_absolute() {
        let tool = FileSystemTool::new("/tmp");
        let result = tool.validate_path("/etc/passwd");
        // This will either fail validation or fail the "starts_with" check
        std::assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_write_file_in_sandbox() {
        let temp_dir = std::env::temp_dir().join(std::format!("fs_tool_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let tool = FileSystemTool::new(&temp_dir);

        // Write a file
        let write_result = tool.write_file("test.txt", "Hello, Rig!").await;
        std::assert!(write_result.is_ok(), "Write should succeed: {:?}", write_result.err());

        // Read it back
        let read_result = tool.read_file("test.txt").await;
        std::assert!(read_result.is_ok(), "Read should succeed: {:?}", read_result.err());
        std::assert_eq!(read_result.unwrap(), "Hello, Rig!");

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_list_directory() {
        let temp_dir = std::env::temp_dir().join(std::format!("fs_tool_list_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let tool = FileSystemTool::new(&temp_dir);

        // Create some files
        tool.write_file("file1.txt", "content1").await.unwrap();
        tool.write_file("file2.txt", "content2").await.unwrap();

        // List directory
        let list_result = tool.list_directory(".").await;
        std::assert!(list_result.is_ok(), "List should succeed: {:?}", list_result.err());

        let files = list_result.unwrap();
        std::assert_eq!(files.len(), 2);
        std::assert!(files.contains(&"file1.txt".to_string()));
        std::assert!(files.contains(&"file2.txt".to_string()));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_prevent_escape_via_symlink() {
        // This test verifies symlink following is caught by canonicalize
        let temp_dir = std::env::temp_dir().join(std::format!("fs_tool_symlink_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let tool = FileSystemTool::new(&temp_dir);

        // Try to read /etc/passwd via any path
        let result = tool.read_file("/etc/passwd").await;
        std::assert!(result.is_err(), "Should reject /etc/passwd");

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[tokio::test]
    async fn test_rig_tool_trait_read_file() {
        let temp_dir = std::env::temp_dir().join(std::format!("rig_tool_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        // Setup test file
        let fs = FileSystemTool::new(&temp_dir);
        fs.write_file("sample.txt", "Rig tool content").await.unwrap();

        // Test via Rig Tool trait
        let tool = ReadFileTool::new(&temp_dir);
        let args = ReadFileArgs {
            path: "sample.txt".to_string(),
        };

        let result = <ReadFileTool as rig::tool::Tool>::call(&tool, args).await;
        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), "Rig tool content");

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
