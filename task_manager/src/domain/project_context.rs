//! Defines the ProjectContext domain entity for codebase analysis.
//!
//! ProjectContext represents a synthesized analysis of a software project's
//! codebase, including detected technologies, key files, architectural patterns,
//! and dependencies. This context is used by Rigger to generate informed
//! task breakdowns that align with the existing project structure.
//!
//! Revision History
//! - 2025-11-23 @AI: Add recent decisions tracking and file relevance scoring (Phase 4 Sprint 9 Task 4.9).
//! - 2025-11-22T16:20:00Z @AI: Initial ProjectContext entity creation for Rigger Phase 0.

/// Represents synthesized context about a software project's codebase.
///
/// ProjectContext is generated through static analysis and file inspection
/// to provide Rigger with an understanding of the project's current state,
/// architecture, and technology stack. This enables context-aware task generation
/// that respects existing patterns and conventions.
///
/// # Fields
///
/// * `id` - Unique identifier (UUID) for this context snapshot.
/// * `project_root` - Absolute path to the project root directory.
/// * `detected_languages` - Programming languages found in the codebase.
/// * `detected_frameworks` - Frameworks and libraries detected (e.g., "React", "Django", "Rig").
/// * `key_directories` - Important directories and their purposes (e.g., "src: source code").
/// * `key_files` - Critical files and their roles (e.g., "Cargo.toml: Rust manifest").
/// * `architectural_patterns` - Detected patterns (e.g., "Hexagonal Architecture", "MVC").
/// * `entry_points` - Main entry points to the application (e.g., "src/main.rs", "index.html").
/// * `created_at` - UTC timestamp when this context was created.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::project_context::ProjectContext;
/// let context = ProjectContext::new(
///     std::string::String::from("/Users/dev/myproject"),
///     std::vec![std::string::String::from("Rust"), std::string::String::from("TypeScript")],
///     std::vec![std::string::String::from("Rig"), std::string::String::from("React")],
///     std::vec![
///         std::string::String::from("src: Source code"),
///         std::string::String::from("tests: Integration tests"),
///     ],
///     std::vec![std::string::String::from("Cargo.toml: Rust workspace manifest")],
///     std::vec![std::string::String::from("Hexagonal Architecture (HEXSER)")],
///     std::vec![std::string::String::from("src/main.rs")],
/// );
///
/// std::assert_eq!(context.detected_languages.len(), 2);
/// std::assert_eq!(context.architectural_patterns.len(), 1);
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProjectContext {
    /// Unique identifier for this context snapshot (UUID v4).
    pub id: String,

    /// Absolute path to the project root directory.
    pub project_root: String,

    /// Programming languages detected in the codebase.
    pub detected_languages: std::vec::Vec<String>,

    /// Frameworks and libraries identified (e.g., "Rig", "React", "SQLite").
    pub detected_frameworks: std::vec::Vec<String>,

    /// Important directories with their purposes.
    pub key_directories: std::vec::Vec<String>,

    /// Critical files with their roles.
    pub key_files: std::vec::Vec<String>,

    /// Architectural patterns detected in the codebase.
    pub architectural_patterns: std::vec::Vec<String>,

    /// Main application entry points.
    pub entry_points: std::vec::Vec<String>,

    /// Recent architectural or implementation decisions (max 50).
    ///
    /// Stores decisions made during development to provide context continuity.
    /// Each decision is timestamped and limited to the most recent 50 entries.
    pub recent_decisions: std::vec::Vec<DecisionEntry>,

    /// UTC timestamp when this context was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Represents a decision made during project development.
///
/// Decisions are tracked to maintain context continuity across task enhancements
/// and provide agents with recent architectural or implementation choices.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DecisionEntry {
    /// The decision description
    pub decision: String,
    /// When this decision was made
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Internal helper for file relevance scoring.
#[derive(Debug)]
struct FileEntry {
    path: String,
    modified: std::time::SystemTime,
    relevance_score: i32,
}

impl ProjectContext {
    /// Creates a new ProjectContext with generated UUID and current timestamp.
    ///
    /// This constructor is the primary way to create a ProjectContext after
    /// analyzing a codebase. It automatically generates a unique ID and
    /// captures the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `project_root` - Absolute path to the project root.
    /// * `detected_languages` - List of programming languages found.
    /// * `detected_frameworks` - List of frameworks/libraries detected.
    /// * `key_directories` - List of important directories with descriptions.
    /// * `key_files` - List of critical files with descriptions.
    /// * `architectural_patterns` - List of detected architectural patterns.
    /// * `entry_points` - List of application entry points.
    ///
    /// # Returns
    ///
    /// A new ProjectContext with generated UUID and current timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::project_context::ProjectContext;
    /// let context = ProjectContext::new(
    ///     std::string::String::from("/path/to/project"),
    ///     std::vec![std::string::String::from("Rust")],
    ///     std::vec![std::string::String::from("Rig")],
    ///     std::vec![std::string::String::from("src: Source code")],
    ///     std::vec![std::string::String::from("Cargo.toml: Manifest")],
    ///     std::vec![std::string::String::from("Hexagonal Architecture")],
    ///     std::vec![std::string::String::from("src/main.rs")],
    /// );
    ///
    /// std::assert!(!context.id.is_empty());
    /// std::assert_eq!(context.project_root, "/path/to/project");
    /// ```
    pub fn new(
        project_root: String,
        detected_languages: std::vec::Vec<String>,
        detected_frameworks: std::vec::Vec<String>,
        key_directories: std::vec::Vec<String>,
        key_files: std::vec::Vec<String>,
        architectural_patterns: std::vec::Vec<String>,
        entry_points: std::vec::Vec<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_root,
            detected_languages,
            detected_frameworks,
            key_directories,
            key_files,
            architectural_patterns,
            entry_points,
            recent_decisions: std::vec::Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Adds a recent decision to the context.
    ///
    /// Decisions are limited to the most recent 50 entries. When adding a new
    /// decision, if the list exceeds 50, the oldest decision is removed.
    ///
    /// # Arguments
    ///
    /// * `decision` - Description of the architectural or implementation decision
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::project_context::ProjectContext;
    /// let mut context = ProjectContext::new(
    ///     std::string::String::from("/project"),
    ///     std::vec![],
    ///     std::vec![],
    ///     std::vec![],
    ///     std::vec![],
    ///     std::vec![],
    ///     std::vec![],
    /// );
    ///
    /// context.add_recent_decision(std::string::String::from("Use SQLite for persistence"));
    /// std::assert_eq!(context.recent_decisions.len(), 1);
    /// ```
    pub fn add_recent_decision(&mut self, decision: String) {
        let entry = DecisionEntry {
            decision,
            timestamp: chrono::Utc::now(),
        };

        self.recent_decisions.push(entry);

        // Limit to 50 most recent decisions
        if self.recent_decisions.len() > 50 {
            self.recent_decisions.remove(0);
        }
    }

    /// Gets relevant files for a given task based on file modification times.
    ///
    /// Scans the project directory for files matching the task's context (by name/path hints)
    /// and returns recently modified files prioritized by modification time.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to find relevant files for
    ///
    /// # Returns
    ///
    /// A vector of relative file paths, sorted by relevance (most recent first),
    /// limited to 10 files.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::domain::project_context::ProjectContext;
    /// # use task_manager::domain::task::Task;
    /// let context = ProjectContext::new(
    ///     std::string::String::from("/project"),
    ///     std::vec![],
    ///     std::vec![],
    ///     std::vec![],
    ///     std::vec![],
    ///     std::vec![],
    ///     std::vec![],
    /// );
    ///
    /// let action_item = transcript_extractor::domain::action_item::ActionItem {
    ///     title: std::string::String::from("Fix authentication bug"),
    ///     assignee: std::option::Option::None,
    ///     due_date: std::option::Option::None,
    /// };
    /// let task = Task::from_action_item(&action_item, std::option::Option::None);
    /// let files = context.get_relevant_files_for_task(&task);
    /// ```
    pub fn get_relevant_files_for_task(&self, task: &crate::domain::task::Task) -> std::vec::Vec<String> {
        // Extract keywords from task title for matching
        let keywords: std::vec::Vec<String> = task.title
            .to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 3) // Filter short words
            .map(|s| s.to_string())
            .collect();

        let project_path = std::path::Path::new(&self.project_root);

        // Collect all files with metadata
        let mut file_entries: std::vec::Vec<FileEntry> = std::vec::Vec::new();

        // Walk the directory tree
        if let std::result::Result::Ok(entries) = std::fs::read_dir(project_path) {
            for entry in entries.flatten() {
                self.collect_files_recursively(&entry.path(), &keywords, &mut file_entries);
            }
        }

        // Sort by relevance score (highest first), then by modification time (newest first)
        file_entries.sort_by(|a, b| {
            b.relevance_score
                .cmp(&a.relevance_score)
                .then_with(|| b.modified.cmp(&a.modified))
        });

        // Return top 10 files as relative paths
        file_entries
            .into_iter()
            .take(10)
            .map(|entry| entry.path)
            .collect()
    }

    /// Helper to recursively collect files with relevance scoring.
    fn collect_files_recursively(
        &self,
        path: &std::path::Path,
        keywords: &[String],
        file_entries: &mut std::vec::Vec<FileEntry>,
    ) {
        // Skip hidden files/directories
        if let std::option::Option::Some(name) = path.file_name() {
            if let std::option::Option::Some(name_str) = name.to_str() {
                if name_str.starts_with('.') {
                    return;
                }
            }
        }

        if path.is_file() {
            // Get file metadata
            if let std::result::Result::Ok(metadata) = std::fs::metadata(path) {
                if let std::result::Result::Ok(modified) = metadata.modified() {
                    // Calculate relevance score
                    let path_str = path.to_string_lossy().to_lowercase();
                    let mut score = 0;

                    for keyword in keywords {
                        if path_str.contains(keyword) {
                            score += 10;
                        }
                    }

                    // Bonus for common important file types
                    if path_str.ends_with(".rs") || path_str.ends_with(".ts") || path_str.ends_with(".js") {
                        score += 5;
                    }

                    if path_str.contains("test") {
                        score += 3;
                    }

                    // Convert to relative path
                    let relative_path = path
                        .strip_prefix(&self.project_root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();

                    file_entries.push(FileEntry {
                        path: relative_path,
                        modified,
                        relevance_score: score,
                    });
                }
            }
        } else if path.is_dir() {
            // Recurse into subdirectories
            if let std::result::Result::Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    self.collect_files_recursively(&entry.path(), keywords, file_entries);
                }
            }
        }
    }

    /// Saves the context to `.rigger/context.json`.
    ///
    /// # Arguments
    ///
    /// * `rigger_dir` - Path to the `.rigger` directory (usually `{project_root}/.rigger`)
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error message.
    pub fn save_to_rigger_dir(&self, rigger_dir: &str) -> std::result::Result<(), std::string::String> {
        let context_path = std::path::Path::new(rigger_dir).join("context.json");

        let json = match serde_json::to_string_pretty(self) {
            std::result::Result::Ok(j) => j,
            std::result::Result::Err(e) => return std::result::Result::Err(std::format!("Serialization error: {}", e)),
        };

        match std::fs::write(&context_path, json) {
            std::result::Result::Ok(_) => std::result::Result::Ok(()),
            std::result::Result::Err(e) => std::result::Result::Err(std::format!("Write error: {}", e)),
        }
    }

    /// Loads context from `.rigger/context.json`.
    ///
    /// # Arguments
    ///
    /// * `rigger_dir` - Path to the `.rigger` directory
    ///
    /// # Returns
    ///
    /// The loaded ProjectContext, or an error if the file doesn't exist or is invalid.
    pub fn load_from_rigger_dir(rigger_dir: &str) -> std::result::Result<Self, std::string::String> {
        let context_path = std::path::Path::new(rigger_dir).join("context.json");

        let content = match std::fs::read_to_string(&context_path) {
            std::result::Result::Ok(c) => c,
            std::result::Result::Err(e) => return std::result::Result::Err(std::format!("Read error: {}", e)),
        };

        match serde_json::from_str(&content) {
            std::result::Result::Ok(ctx) => std::result::Result::Ok(ctx),
            std::result::Result::Err(e) => std::result::Result::Err(std::format!("Deserialization error: {}", e)),
        }
    }

    /// Synthesizes codebase context by analyzing the project directory.
    ///
    /// This method performs static analysis of the codebase to detect languages,
    /// frameworks, architectural patterns, and key files. It provides an automated
    /// way to generate ProjectContext without manual specification.
    ///
    /// # Current Implementation
    ///
    /// This is a placeholder implementation that returns minimal context.
    /// Future implementations will include:
    /// - File extension analysis for language detection
    /// - Manifest file parsing (Cargo.toml, package.json, requirements.txt)
    /// - Pattern recognition for architectural styles
    /// - Dependency graph analysis
    ///
    /// # Arguments
    ///
    /// * `project_root` - Absolute path to the project root directory.
    ///
    /// # Returns
    ///
    /// A Result containing the synthesized ProjectContext or an error message.
    ///
    /// # Errors
    ///
    /// Returns an error if the project_root does not exist or is not accessible.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_manager::domain::project_context::ProjectContext;
    /// let context = ProjectContext::synthesize_context(
    ///     std::string::String::from("/path/to/project")
    /// ).unwrap();
    ///
    /// std::assert!(!context.detected_languages.is_empty());
    /// ```
    pub fn synthesize_context(project_root: String) -> std::result::Result<Self, std::string::String> {
        // Validate project root exists
        let path = std::path::Path::new(&project_root);
        if !path.exists() {
            return std::result::Result::Err(std::format!("Project root does not exist: {}", project_root));
        }
        if !path.is_dir() {
            return std::result::Result::Err(std::format!("Project root is not a directory: {}", project_root));
        }

        // Placeholder implementation - returns minimal context
        // TODO: Implement full static analysis in future sprint
        std::result::Result::Ok(Self::new(
            project_root,
            std::vec![std::string::String::from("Unknown")],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        ))
    }
}

// Export DecisionEntry for public use
pub use DecisionEntry as Decision;

#[cfg(test)]
mod tests {
    #[test]
    fn test_project_context_creation() {
        // Test: Validates ProjectContext constructor generates valid UUID and timestamp.
        // Justification: Ensures ProjectContext entities are created with required metadata.
        let context = super::ProjectContext::new(
            std::string::String::from("/test/project"),
            std::vec![std::string::String::from("Rust"), std::string::String::from("Python")],
            std::vec![std::string::String::from("Rig"), std::string::String::from("FastAPI")],
            std::vec![std::string::String::from("src: Source code")],
            std::vec![std::string::String::from("Cargo.toml: Rust manifest")],
            std::vec![std::string::String::from("Hexagonal Architecture")],
            std::vec![std::string::String::from("src/main.rs")],
        );

        std::assert!(!context.id.is_empty());
        std::assert_eq!(context.project_root, "/test/project");
        std::assert_eq!(context.detected_languages.len(), 2);
        std::assert_eq!(context.detected_frameworks.len(), 2);
        std::assert_eq!(context.architectural_patterns.len(), 1);
    }

    #[test]
    fn test_project_context_serialization() {
        // Test: Validates ProjectContext can be serialized to JSON and deserialized.
        // Justification: Ensures ProjectContext persistence and API compatibility.
        let context = super::ProjectContext::new(
            std::string::String::from("/test/serialize"),
            std::vec![std::string::String::from("TypeScript")],
            std::vec![std::string::String::from("React")],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        );

        let json = serde_json::to_string(&context).unwrap();
        let deserialized: super::ProjectContext = serde_json::from_str(&json).unwrap();

        std::assert_eq!(deserialized.id, context.id);
        std::assert_eq!(deserialized.project_root, context.project_root);
        std::assert_eq!(deserialized.detected_languages, context.detected_languages);
    }

    #[test]
    fn test_project_context_with_empty_fields() {
        // Test: Validates ProjectContext handles empty analysis results.
        // Justification: Some projects may have minimal detectable features.
        let context = super::ProjectContext::new(
            std::string::String::from("/minimal/project"),
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        );

        std::assert!(context.detected_languages.is_empty());
        std::assert!(context.detected_frameworks.is_empty());
        std::assert!(context.key_directories.is_empty());
    }

    #[test]
    fn test_project_context_uuid_uniqueness() {
        // Test: Validates each ProjectContext gets a unique UUID.
        // Justification: Critical for context versioning and tracking.
        let ctx1 = super::ProjectContext::new(
            std::string::String::from("/project1"),
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        );

        let ctx2 = super::ProjectContext::new(
            std::string::String::from("/project2"),
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        );

        std::assert_ne!(ctx1.id, ctx2.id);
    }

    #[test]
    fn test_synthesize_context_nonexistent_path() {
        // Test: Validates synthesize_context returns error for nonexistent paths.
        // Justification: Must fail gracefully when given invalid project root.
        let result = super::ProjectContext::synthesize_context(
            std::string::String::from("/this/path/does/not/exist/surely")
        );

        std::assert!(result.is_err());
    }

    #[test]
    fn test_synthesize_context_placeholder() {
        // Test: Validates synthesize_context returns minimal context for valid path.
        // Justification: Current placeholder implementation should work with existing directories.
        let result = super::ProjectContext::synthesize_context(
            std::string::String::from(".")
        );

        std::assert!(result.is_ok());
        let context = result.unwrap();
        std::assert_eq!(context.project_root, ".");
        std::assert_eq!(context.detected_languages.len(), 1);
        std::assert_eq!(context.detected_languages[0], "Unknown");
    }

    #[test]
    fn test_add_recent_decision() {
        // Test: Validates adding decisions to context.
        let mut context = super::ProjectContext::new(
            std::string::String::from("/project"),
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        );

        std::assert_eq!(context.recent_decisions.len(), 0);

        context.add_recent_decision(std::string::String::from("Use SQLite for persistence"));
        std::assert_eq!(context.recent_decisions.len(), 1);
        std::assert_eq!(context.recent_decisions[0].decision, "Use SQLite for persistence");

        context.add_recent_decision(std::string::String::from("Use Hexagonal Architecture"));
        std::assert_eq!(context.recent_decisions.len(), 2);
    }

    #[test]
    fn test_add_recent_decision_limits_to_50() {
        // Test: Validates decision list is capped at 50 entries.
        let mut context = super::ProjectContext::new(
            std::string::String::from("/project"),
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        );

        // Add 60 decisions
        for i in 0..60 {
            context.add_recent_decision(std::format!("Decision {}", i));
        }

        std::assert_eq!(context.recent_decisions.len(), 50);
        // First decision (0) should be removed, so oldest is now "Decision 10"
        std::assert_eq!(context.recent_decisions[0].decision, "Decision 10");
        std::assert_eq!(context.recent_decisions[49].decision, "Decision 59");
    }

    #[test]
    fn test_save_and_load_context() {
        // Test: Validates save and load from .rigger/context.json.
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_ctx_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let mut context = super::ProjectContext::new(
            std::string::String::from("/test/project"),
            std::vec![std::string::String::from("Rust")],
            std::vec![std::string::String::from("Rig")],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        );

        context.add_recent_decision(std::string::String::from("Use Ollama for LLM"));

        // Save
        let save_result = context.save_to_rigger_dir(temp_dir.to_str().unwrap());
        std::assert!(save_result.is_ok());

        // Load
        let load_result = super::ProjectContext::load_from_rigger_dir(temp_dir.to_str().unwrap());
        std::assert!(load_result.is_ok());

        let loaded = load_result.unwrap();
        std::assert_eq!(loaded.id, context.id);
        std::assert_eq!(loaded.project_root, "/test/project");
        std::assert_eq!(loaded.recent_decisions.len(), 1);
        std::assert_eq!(loaded.recent_decisions[0].decision, "Use Ollama for LLM");

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_get_relevant_files_for_task() {
        // Test: Validates file relevance scoring returns recently modified files.
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_files_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        // Create test files
        std::fs::write(temp_dir.join("auth.rs"), "// auth module").unwrap();
        std::fs::write(temp_dir.join("user.rs"), "// user module").unwrap();
        std::fs::write(temp_dir.join("README.md"), "# Project").unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps

        std::fs::write(temp_dir.join("authentication_test.rs"), "// test").unwrap();

        let context = super::ProjectContext::new(
            temp_dir.to_string_lossy().to_string(),
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        );

        let action_item = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Fix authentication bug"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = crate::domain::task::Task::from_action_item(&action_item, std::option::Option::None);

        let files = context.get_relevant_files_for_task(&task);

        // Should return files, prioritizing those matching "authentication"
        std::assert!(!files.is_empty());
        std::println!("Relevant files: {:?}", files);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
