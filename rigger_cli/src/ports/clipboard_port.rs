//! Clipboard port for copy/paste operations.
//!
//! Defines the interface for interacting with the system clipboard following
//! hexagonal architecture principles. This port abstracts clipboard operations
//! allowing for different implementations (OS clipboard, test mocks, etc.).
//!
//! Revision History
//! - 2025-11-24T00:30:00Z @AI: Create clipboard port for Task 0.3 (copy/paste operations).

/// Port for clipboard operations.
///
/// This trait defines the interface for copying and pasting text to/from
/// the system clipboard. Implementations can use OS-specific clipboard
/// APIs or provide mock implementations for testing.
pub trait ClipboardPort: Send + Sync {
    /// Copies text to the clipboard.
    ///
    /// # Arguments
    ///
    /// * `text` - The text content to copy
    ///
    /// # Returns
    ///
    /// * `Ok(())` if successful
    /// * `Err(String)` with error message if clipboard access fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rigger_cli::ports::clipboard_port::ClipboardPort;
    /// # fn example(clipboard: &dyn ClipboardPort) -> Result<(), String> {
    /// clipboard.copy_text("Task ID: TUI-042\nTitle: Implement clipboard")?;
    /// # Ok(())
    /// # }
    /// ```
    fn copy_text(&self, text: &str) -> Result<(), String>;

    /// Retrieves text from the clipboard.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing clipboard contents
    /// * `Err(String)` if clipboard is empty or access fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rigger_cli::ports::clipboard_port::ClipboardPort;
    /// # fn example(clipboard: &dyn ClipboardPort) -> Result<(), String> {
    /// let text = clipboard.get_text()?;
    /// println!("Clipboard: {}", text);
    /// # Ok(())
    /// # }
    /// ```
    fn get_text(&self) -> Result<String, String>;
}
