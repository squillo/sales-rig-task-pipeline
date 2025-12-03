//! Embedded sqlite-vec extension management.
//!
//! This module embeds the sqlite-vec extension binary into the compiled executable
//! and extracts it to a temporary location at runtime. This ensures the extension
//! is always available regardless of working directory or installation location.
//!
//! Revision History
//! - 2025-11-29T06:30:00Z @AI: Create embedded extension module to fix RAG availability

/// Platform-specific embedded extension binary.
///
/// On macOS, embeds vec0.dylib (158KB).
/// On Linux, would embed vec0.so.
/// On Windows, would embed vec0.dll.
#[cfg(target_os = "macos")]
const SQLITE_VEC_EXTENSION: &[u8] = include_bytes!("../../../.rigger/lib/vec0.dylib");

#[cfg(target_os = "linux")]
const SQLITE_VEC_EXTENSION: &[u8] = include_bytes!("../../../.rigger/lib/vec0.so");

#[cfg(target_os = "windows")]
const SQLITE_VEC_EXTENSION: &[u8] = include_bytes!("../../../.rigger/lib/vec0.dll");

/// Extracts the embedded sqlite-vec extension to a temporary file and returns the path.
///
/// The extension is written to the system temp directory with a unique name to avoid
/// conflicts. The file will be automatically cleaned up when the process exits.
///
/// # Returns
///
/// Returns the absolute path to the extracted extension file.
///
/// # Errors
///
/// Returns an error if:
/// - Failed to create temp directory
/// - Failed to write extension file
/// - Failed to set file permissions (Unix only)
pub fn extract_extension() -> std::result::Result<std::path::PathBuf, String> {
    // Create a unique temp directory for this process
    let temp_dir = std::env::temp_dir().join(std::format!("rigger-sqlite-vec-{}", std::process::id()));

    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| std::format!("Failed to create temp directory: {:?}", e))?;

    // Determine extension filename based on platform
    #[cfg(target_os = "macos")]
    let filename = "vec0.dylib";

    #[cfg(target_os = "linux")]
    let filename = "vec0.so";

    #[cfg(target_os = "windows")]
    let filename = "vec0.dll";

    let extension_path = temp_dir.join(filename);

    // Only write if it doesn't already exist (avoid rewriting on subsequent calls)
    if !extension_path.exists() {
        std::fs::write(&extension_path, SQLITE_VEC_EXTENSION)
            .map_err(|e| std::format!("Failed to write extension file: {:?}", e))?;

        // On Unix, ensure the file is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&extension_path, permissions)
                .map_err(|e| std::format!("Failed to set permissions: {:?}", e))?;
        }
    }

    std::result::Result::Ok(extension_path)
}

/// Returns the extension path without the file extension (as required by SQLite).
///
/// SQLite's load_extension function expects the path without the .dylib/.so/.dll suffix.
pub fn get_extension_path_for_sqlite() -> std::result::Result<String, String> {
    let full_path = extract_extension()?;

    // Convert to string and remove extension
    let path_str = full_path
        .to_str()
        .ok_or_else(|| String::from("Invalid path encoding"))?;

    // Remove the file extension (.dylib, .so, or .dll)
    let without_ext = if let std::option::Option::Some(stripped) = path_str.strip_suffix(".dylib") {
        stripped
    } else if let std::option::Option::Some(stripped) = path_str.strip_suffix(".so") {
        stripped
    } else if let std::option::Option::Some(stripped) = path_str.strip_suffix(".dll") {
        stripped
    } else {
        path_str
    };

    std::result::Result::Ok(String::from(without_ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_extension() {
        let path = extract_extension().expect("Failed to extract extension");
        assert!(path.exists(), "Extension file should exist");

        // Verify it's in the temp directory
        assert!(path.starts_with(std::env::temp_dir()));

        // Verify it has the correct extension
        #[cfg(target_os = "macos")]
        assert!(path.to_str().unwrap().ends_with(".dylib"));

        #[cfg(target_os = "linux")]
        assert!(path.to_str().unwrap().ends_with(".so"));

        #[cfg(target_os = "windows")]
        assert!(path.to_str().unwrap().ends_with(".dll"));
    }

    #[test]
    fn test_get_extension_path_for_sqlite() {
        let path = get_extension_path_for_sqlite().expect("Failed to get SQLite extension path");

        // Should not have file extension
        assert!(!path.ends_with(".dylib"));
        assert!(!path.ends_with(".so"));
        assert!(!path.ends_with(".dll"));

        // Should be in temp directory
        assert!(path.contains(std::env::temp_dir().to_str().unwrap()));
    }
}
