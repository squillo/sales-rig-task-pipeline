//! MLX Subprocess Adapter for macOS Apple Silicon optimization.
//!
//! This adapter provides 30-50% faster LLM inference on macOS by using Apple's
//! MLX framework via Python subprocess. It implements TaskEnhancementPort and
//! TaskDecompositionPort using the mlx-lm Python package.
//!
//! # Platform Support
//!
//! **macOS Apple Silicon only** (M1/M2/M3/M4 chips). Automatically falls back
//! to Ollama on other platforms.
//!
//! # Prerequisites
//!
//! 1. Python 3.9+ installed
//! 2. mlx-lm package: `pip install mlx-lm`
//! 3. Environment variable: `INFERENCE_BACKEND=mlx`
//!
//! # Performance
//!
//! Benchmarks show 30-50% speed improvement over Ollama for the same models:
//! - Phi-3-mini: ~45 t/s (MLX) vs ~30 t/s (Ollama) on M3
//! - Orca-2: ~35 t/s (MLX) vs ~25 t/s (Ollama) on M3
//!
//! See docs/MLX_RESEARCH.md for detailed benchmarks.
//!
//! Revision History
//! - 2025-11-24T00:00:00Z @AI: Create MLX subprocess adapter for Phase 5 Sprint 11 Task 5.8.

/// MLX subprocess adapter for Apple Silicon optimization.
///
/// This adapter spawns Python processes to use the mlx-lm package for LLM
/// inference. While not "pure Rust", this approach provides immediate access
/// to MLX's performance benefits without the risks of immature mlx-rs bindings.
///
/// # Examples
///
/// ```no_run
/// use task_orchestrator::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter;
///
/// // Check if MLX is available on this system
/// if MlxSubprocessAdapter::is_available() {
///     let adapter = MlxSubprocessAdapter::new(String::from("mlx-community/Phi-3-mini-4k-instruct"));
///     std::println!("Using MLX for 30-50% faster inference!");
/// } else {
///     std::println!("MLX not available, using Ollama fallback");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MlxSubprocessAdapter {
    model_name: String,
    python_path: String,
}

/// Internal struct for parsing enhancement responses from LLM.
#[derive(Debug, serde::Deserialize)]
struct EnhancementExtraction {
    enhancement_type: String,
    content: String,
}

/// Internal struct for parsing subtask responses from LLM.
#[derive(Debug, serde::Deserialize)]
struct SubtaskExtraction {
    title: String,
}

impl MlxSubprocessAdapter {
    /// Creates a new MLX subprocess adapter with the specified model.
    ///
    /// # Arguments
    ///
    /// * `model_name` - MLX model identifier (e.g., "mlx-community/Phi-3-mini-4k-instruct")
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter;
    ///
    /// let adapter = MlxSubprocessAdapter::new(String::from("mlx-community/Phi-3-mini-4k-instruct"));
    /// ```
    pub fn new(model_name: String) -> Self {
        // Detect Python path (prefer python3)
        let python_path = std::env::var("PYTHON_PATH")
            .unwrap_or_else(|_| Self::detect_python_path());

        Self {
            model_name,
            python_path,
        }
    }

    /// Detects the Python executable path.
    ///
    /// Tries `python3` first, then falls back to `python`.
    fn detect_python_path() -> String {
        // Try python3 first (recommended)
        if std::process::Command::new("python3")
            .arg("--version")
            .output()
            .is_ok()
        {
            return String::from("python3");
        }

        // Fallback to python
        String::from("python")
    }

    /// Checks if MLX is available on the current system.
    ///
    /// Returns `true` if:
    /// 1. Running on macOS Apple Silicon (aarch64)
    /// 2. Python is installed
    /// 3. mlx-lm package is installed
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter;
    ///
    /// if MlxSubprocessAdapter::is_available() {
    ///     std::println!("MLX backend available!");
    /// }
    /// ```
    pub fn is_available() -> bool {
        // Only supported on macOS Apple Silicon
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            return false;
        }

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            // Check if mlx-lm is installed
            let check_script = "import mlx_lm; print('OK')";
            let result = std::process::Command::new(Self::detect_python_path())
                .arg("-c")
                .arg(check_script)
                .output();

            match result {
                std::result::Result::Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.trim() == "OK"
                }
                std::result::Result::Err(_) => false,
            }
        }
    }

    /// Parses enhancement response from LLM JSON output.
    ///
    /// Expects JSON format: `{"enhancement_type": "...", "content": "..."}`
    fn parse_enhancement_from_response(response: &str) -> std::result::Result<EnhancementExtraction, String> {
        // Try to extract JSON from code blocks or find raw JSON
        let json_str = if let std::option::Option::Some(start) = response.find('{') {
            if let std::option::Option::Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        serde_json::from_str::<EnhancementExtraction>(json_str)
            .map_err(|e| std::format!("Failed to parse enhancement JSON: {}", e))
    }

    /// Parses subtask extractions from LLM JSON array output.
    ///
    /// Expects JSON array: `[{"title": "..."}, {"title": "..."}, ...]`
    fn parse_subtasks_from_response(response: &str) -> std::result::Result<std::vec::Vec<SubtaskExtraction>, String> {
        // Try to extract JSON from code blocks or find raw JSON
        let json_str = if let std::option::Option::Some(start) = response.find('[') {
            if let std::option::Option::Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        serde_json::from_str::<std::vec::Vec<SubtaskExtraction>>(json_str)
            .map_err(|e| std::format!("Failed to parse subtasks JSON: {}", e))
    }

    /// Generates text using MLX-LM via Python subprocess.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The input prompt for generation
    /// * `max_tokens` - Maximum tokens to generate (default: 256)
    ///
    /// # Returns
    ///
    /// Generated text or error message.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Python execution fails
    /// - mlx-lm is not installed
    /// - Model loading fails
    /// - Generation fails
    async fn generate_text(&self, prompt: &str, max_tokens: usize) -> std::result::Result<String, String> {
        // Escape prompt for Python string (basic escaping)
        let escaped_prompt = prompt.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");

        let script = std::format!(
            r#"
import mlx_lm
import sys

try:
    # Load model and tokenizer
    model, tokenizer = mlx_lm.load("{model_name}")

    # Generate text
    response = mlx_lm.generate(
        model,
        tokenizer,
        prompt="{prompt}",
        max_tokens={max_tokens},
        verbose=False
    )

    # Print only the generated text (no debug output)
    print(response, end='')
except Exception as e:
    print(f"ERROR: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
            model_name = self.model_name,
            prompt = escaped_prompt,
            max_tokens = max_tokens
        );

        // Execute Python script via tokio async subprocess
        let output = tokio::process::Command::new(&self.python_path)
            .arg("-c")
            .arg(&script)
            .output()
            .await
            .map_err(|e| std::format!("Failed to execute Python: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return std::result::Result::Err(std::format!("MLX generation failed: {}", stderr));
        }

        let result = String::from_utf8(output.stdout)
            .map_err(|e| std::format!("Invalid UTF-8 output: {}", e))?;

        std::result::Result::Ok(result)
    }
}

#[async_trait::async_trait]
impl crate::ports::task_enhancement_port::TaskEnhancementPort for MlxSubprocessAdapter {
    async fn generate_enhancement(
        &self,
        task: &task_manager::domain::task::Task,
    ) -> std::result::Result<task_manager::domain::enhancement::Enhancement, std::string::String> {
        // Build enhancement prompt
        let prompt = std::format!(
            r#"Enhance the following task with additional details and acceptance criteria.

Task: {}

Provide a JSON response with:
{{
  "enhancement_type": "clarify",
  "content": "Enhanced task description with details..."
}}

JSON Response:"#,
            task.title
        );

        // Generate enhancement via MLX
        let response = self.generate_text(&prompt, 256).await?;

        // Parse JSON response (use same pattern as OllamaEnhancementAdapter)
        let parsed = Self::parse_enhancement_from_response(&response)?;

        std::result::Result::Ok(task_manager::domain::enhancement::Enhancement {
            enhancement_id: uuid::Uuid::new_v4().to_string(),
            task_id: task.id.clone(),
            timestamp: chrono::Utc::now(),
            enhancement_type: parsed.enhancement_type,
            content: parsed.content,
        })
    }
}

#[async_trait::async_trait]
impl crate::ports::task_decomposition_port::TaskDecompositionPort for MlxSubprocessAdapter {
    async fn decompose_task(
        &self,
        task: &task_manager::domain::task::Task,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String> {
        // Build decomposition prompt (Orca-2 style: recall-reason-generate)
        let prompt = std::format!(
            r#"You are a task decomposition expert using the recall-reason-generate approach.

**Recall**: The task to decompose is: "{}"

**Reason**: This task should be broken into 3-5 concrete, actionable subtasks.

**Generate**: Provide a JSON array of subtasks:
[
  {{"title": "Subtask 1 description"}},
  {{"title": "Subtask 2 description"}},
  {{"title": "Subtask 3 description"}}
]

JSON Array:"#,
            task.title
        );

        // Generate decomposition via MLX (Orca-2 optimized)
        let response = self.generate_text(&prompt, 512).await?;

        // Parse JSON array into subtasks (use same pattern as RigTaskDecompositionAdapter)
        let extractions = Self::parse_subtasks_from_response(&response)?;

        // Convert to Task objects with parent linkage
        let subtasks: std::vec::Vec<task_manager::domain::task::Task> = extractions
            .into_iter()
            .map(|extraction| {
                let action = transcript_extractor::domain::action_item::ActionItem {
                    title: extraction.title,
                    assignee: task.assignee.clone(),
                    due_date: task.due_date.clone(),
                };
                let mut subtask = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
                subtask.parent_task_id = std::option::Option::Some(task.id.clone());
                subtask
            })
            .collect();

        std::result::Result::Ok(subtasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlx_adapter_creation() {
        // Test: Validates adapter instantiation.
        let adapter = MlxSubprocessAdapter::new(String::from("mlx-community/Phi-3-mini-4k-instruct"));
        std::assert!(!adapter.model_name.is_empty());
        std::assert!(!adapter.python_path.is_empty());
    }

    #[test]
    fn test_is_available_detects_platform() {
        // Test: Validates platform detection logic.
        // Justification: Should return false on non-macOS or non-aarch64.
        let available = MlxSubprocessAdapter::is_available();

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            // On macOS Apple Silicon, availability depends on mlx-lm installation
            // (could be true or false)
            std::println!("MLX available on macOS: {}", available);
        }

        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            // On other platforms, should always be false
            std::assert!(!available, "MLX should not be available on non-Apple Silicon");
        }
    }

    #[tokio::test]
    #[ignore] // Requires MLX-LM installation
    async fn test_generate_text_integration() {
        // Test: Validates text generation via MLX subprocess.
        // Justification: End-to-end test of MLX integration.
        if !MlxSubprocessAdapter::is_available() {
            std::println!("Skipping: MLX not available");
            return;
        }

        let adapter = MlxSubprocessAdapter::new(String::from("mlx-community/Phi-3-mini-4k-instruct"));
        let result = adapter.generate_text("Explain quantum computing in one sentence:", 50).await;

        std::assert!(result.is_ok(), "Generation should succeed: {:?}", result.err());
        let text = result.unwrap();
        std::assert!(!text.is_empty(), "Should generate non-empty text");
        std::println!("Generated: {}", text);
    }
}
