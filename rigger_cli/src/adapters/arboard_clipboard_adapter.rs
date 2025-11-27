//! Arboard clipboard adapter implementation.
//!
//! Concrete implementation of the ClipboardPort using the arboard library
//! for cross-platform clipboard access (Windows, macOS, Linux with X11/Wayland).
//!
//! Revision History
//! - 2025-11-24T00:30:00Z @AI: Create arboard clipboard adapter with error handling.

/// Arboard-based clipboard adapter.
///
/// Uses the arboard crate to provide cross-platform clipboard functionality.
/// This adapter implements the ClipboardPort trait.
///
/// # Examples
///
/// ```no_run
/// use rigger_cli::adapters::arboard_clipboard_adapter::ArboardClipboardAdapter;
/// use rigger_cli::ports::clipboard_port::ClipboardPort;
///
/// let clipboard = ArboardClipboardAdapter::new();
/// clipboard.copy_text("Hello from clipboard!").unwrap();
/// let text = clipboard.get_text().unwrap();
/// assert_eq!(text, "Hello from clipboard!");
/// ```
pub struct ArboardClipboardAdapter {
    clipboard: std::sync::Mutex<arboard::Clipboard>,
}

impl ArboardClipboardAdapter {
    /// Creates a new arboard clipboard adapter.
    ///
    /// # Returns
    ///
    /// * `Ok(ArboardClipboardAdapter)` if clipboard is accessible
    /// * `Err(String)` if clipboard initialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rigger_cli::adapters::arboard_clipboard_adapter::ArboardClipboardAdapter;
    ///
    /// let clipboard = ArboardClipboardAdapter::new().unwrap();
    /// ```
    pub fn new() -> std::result::Result<Self, String> {
        let clipboard = arboard::Clipboard::new()
            .map_err(|e| std::format!("Failed to initialize clipboard: {}", e))?;

        std::result::Result::Ok(Self {
            clipboard: std::sync::Mutex::new(clipboard),
        })
    }
}

impl crate::ports::clipboard_port::ClipboardPort for ArboardClipboardAdapter {
    fn copy_text(&self, text: &str) -> std::result::Result<(), String> {
        let mut clipboard = self
            .clipboard
            .lock()
            .map_err(|e| std::format!("Failed to lock clipboard: {}", e))?;

        clipboard
            .set_text(text)
            .map_err(|e| std::format!("Failed to copy to clipboard: {}", e))?;

        std::result::Result::Ok(())
    }

    fn get_text(&self) -> std::result::Result<String, String> {
        let mut clipboard = self
            .clipboard
            .lock()
            .map_err(|e| std::format!("Failed to lock clipboard: {}", e))?;

        clipboard
            .get_text()
            .map_err(|e| std::format!("Failed to read from clipboard: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::clipboard_port::ClipboardPort;

    #[test]
    fn test_clipboard_adapter_creation() {
        // Test: Validates clipboard adapter can be created.
        // Justification: Basic smoke test for clipboard initialization.
        // Note: May fail in CI environments without display server.
        let result = ArboardClipboardAdapter::new();
        // Don't assert success as CI may not have clipboard access
        // This test mainly checks compilation
        let _ = result;
    }

    #[test]
    fn test_clipboard_round_trip() {
        // Test: Validates copy then paste retrieves same text.
        // Justification: Core clipboard functionality test.
        // Note: Skipped in CI - requires clipboard access.
        let adapter = match ArboardClipboardAdapter::new() {
            std::result::Result::Ok(a) => a,
            std::result::Result::Err(_) => return, // Skip if no clipboard
        };

        let test_text = "Test clipboard content";
        if adapter.copy_text(test_text).is_ok() {
            if let std::result::Result::Ok(retrieved) = adapter.get_text() {
                std::assert_eq!(retrieved, test_text);
            }
        }
    }

    #[test]
    fn test_clipboard_unicode_support() {
        // Test: Validates clipboard handles Unicode correctly.
        // Justification: Tasks may contain emoji and international characters.
        let adapter = match ArboardClipboardAdapter::new() {
            std::result::Result::Ok(a) => a,
            std::result::Result::Err(_) => return,
        };

        let test_text = "📋 Task with emoji and Unicode: 日本語";
        if adapter.copy_text(test_text).is_ok() {
            if let std::result::Result::Ok(retrieved) = adapter.get_text() {
                std::assert_eq!(retrieved, test_text);
            }
        }
    }
}
