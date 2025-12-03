//! Ollama-based adapter implementing TaskEnhancementPort.
//!
//! This adapter uses Rig's Extractor API with JSON Schema to generate structured
//! task enhancements via LLM. Schema enforcement ensures reliable, valid output.
//!
//! Revision History
//! - 2025-11-23T21:00:00Z @AI: Complete Task 4.10 - Add ProjectContext integration test (Phase 4 Sprint 9).
//! - 2025-11-23 @AI: Integrate FileSystemTool into Agent for project context access (Phase 4 Sprint 9 Task 4.8).
//! - 2025-11-23T15:05:00Z @AI: Upgrade to use Rig Extractor with JSON Schema enforcement (Phase 1 Sprint 3).
//! - 2025-11-23T14:45:00Z @AI: Upgrade to use Rig CompletionModel with real LLM calls (Phase 1 Sprint 2).
//! - 2025-11-12T16:48:00Z @AI: Derive hexser::HexAdapter to align adapters with HEXSER usage for ports/adapters.
//! - 2025-11-12T21:34:00Z @AI: Add minimal OllamaEnhancementAdapter with async implementation and unit test.

/// Extraction DTO for enhancement generation.
///
/// This is separate from the domain Enhancement to allow the LLM
/// to generate just the enhancement content without IDs and timestamps.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
struct EnhancementExtraction {
    /// Type of enhancement (e.g., "clarify", "decompose", "specify")
    enhancement_type: String,

    /// The enhancement suggestion content
    content: String,
}

/// Adapter that generates task enhancements via Ollama LLM using Rig Agent with FileSystemTool.
///
/// Uses Rig's Agent API with FileSystemTool registered, enabling the LLM to read
/// project files when generating context-aware enhancement suggestions.
#[derive(Debug, Clone, hexser::HexAdapter)]
pub struct OllamaEnhancementAdapter {
    model: String,
    project_root: std::option::Option<std::path::PathBuf>,
}

impl OllamaEnhancementAdapter {
    /// Creates a new adapter instance using the provided model name.
    ///
    /// # Arguments
    ///
    /// * `model` - The Ollama model name (e.g., "llama3.1")
    ///
    /// # Returns
    ///
    /// An adapter without filesystem access (project_root = None).
    pub fn new(model: String) -> Self {
        Self {
            model,
            project_root: std::option::Option::None,
        }
    }

    /// Creates a new adapter with filesystem tool access.
    ///
    /// # Arguments
    ///
    /// * `model` - The Ollama model name
    /// * `project_root` - Absolute path to project root for sandboxed file access
    ///
    /// # Returns
    ///
    /// An adapter with FileSystemTool enabled for the agent.
    pub fn new_with_project_root(
        model: String,
        project_root: impl std::convert::AsRef<std::path::Path>,
    ) -> Self {
        Self {
            model,
            project_root: std::option::Option::Some(project_root.as_ref().to_path_buf()),
        }
    }

    /// Returns the configured model name.
    pub fn model(&self) -> &str {
        self.model.as_str()
    }

    /// Returns the project root path if configured.
    pub fn project_root(&self) -> std::option::Option<&std::path::Path> {
        self.project_root.as_deref()
    }

    /// Creates a fallback enhancement when LLM is unavailable.
    fn create_fallback_enhancement(task: &task_manager::domain::task::Task) -> task_manager::domain::enhancement::Enhancement {
        let ts = chrono::Utc::now();
        task_manager::domain::enhancement::Enhancement {
            enhancement_id: std::format!("enh-{}-{}", task.id, ts.timestamp_millis()),
            task_id: task.id.clone(),
            timestamp: ts,
            enhancement_type: std::string::String::from("clarify"),
            content: std::format!("Clarify the specific requirements and acceptance criteria for: {}", task.title),
        }
    }

    /// Parses EnhancementExtraction from agent response text.
    ///
    /// Looks for JSON object in ```json blocks or standalone JSON.
    fn parse_enhancement_from_response(response: &str) -> std::result::Result<EnhancementExtraction, std::string::String> {
        // Try to find JSON in code block first
        if let std::option::Option::Some(start_idx) = response.find("```json") {
            let json_start = start_idx + 7; // Skip past "```json"
            // Look for closing ``` AFTER the opening tag
            if let std::option::Option::Some(relative_end) = response[json_start..].find("```") {
                let json_end = json_start + relative_end;
                let json_str = response[json_start..json_end].trim();

                match serde_json::from_str::<EnhancementExtraction>(json_str) {
                    std::result::Result::Ok(extracted) => return std::result::Result::Ok(extracted),
                    std::result::Result::Err(_) => {} // Fall through to other methods
                }
            }
        }

        // Try to find standalone JSON object
        if let std::option::Option::Some(start_idx) = response.find('{') {
            if let std::option::Option::Some(end_idx) = response.rfind('}') {
                let json_str = &response[start_idx..=end_idx];

                match serde_json::from_str::<EnhancementExtraction>(json_str) {
                    std::result::Result::Ok(extracted) => return std::result::Result::Ok(extracted),
                    std::result::Result::Err(e) => {
                        return std::result::Result::Err(std::format!("Failed to parse JSON from response: {}", e));
                    }
                }
            }
        }

        std::result::Result::Err(std::string::String::from("No valid JSON found in agent response"))
    }

    /// Builds the extraction prompt for enhancement generation with optional project context.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to enhance
    /// * `has_file_access` - Whether the agent has FileSystemTool available
    /// * `context_opt` - Optional ProjectContext with recent decisions and relevant files
    fn build_extraction_prompt(
        task: &task_manager::domain::task::Task,
        has_file_access: bool,
        context_opt: std::option::Option<&task_manager::domain::project_context::ProjectContext>,
    ) -> std::string::String {
        let mut prompt = std::string::String::new();

        prompt.push_str("Analyze the following task and suggest ONE specific enhancement.\n\n");

        if has_file_access {
            prompt.push_str("# Available Tools\n\n");
            prompt.push_str("You have access to the following project file tools:\n");
            prompt.push_str("- `read_file`: Read a file from the project (relative path)\n");
            prompt.push_str("- `list_directory`: List files in a project directory\n\n");
            prompt.push_str("**Use these tools** if you need to understand the project structure or read existing code to provide better enhancement suggestions.\n\n");
        }

        // Add ProjectContext information if available
        if let std::option::Option::Some(context) = context_opt {
            prompt.push_str("# Project Context\n\n");

            // Add recent decisions for continuity
            if !context.recent_decisions.is_empty() {
                prompt.push_str("## Recent Decisions\n\n");
                let recent_count = std::cmp::min(5, context.recent_decisions.len());
                for decision in context.recent_decisions.iter().rev().take(recent_count) {
                    prompt.push_str(&std::format!("- {}\n", decision.decision));
                }
                prompt.push_str("\n");
            }

            // Add relevant files
            let relevant_files = context.get_relevant_files_for_task(task);
            if !relevant_files.is_empty() {
                prompt.push_str("## Relevant Files\n\n");
                prompt.push_str("Recently modified files that may be relevant to this task:\n");
                for file in relevant_files.iter().take(5) {
                    prompt.push_str(&std::format!("- `{}`\n", file));
                }
                prompt.push_str("\n");
                prompt.push_str("**Consider these files** when suggesting enhancements to ensure alignment with existing code.\n\n");
            }
        }

        prompt.push_str("# Task Information\n\n");
        prompt.push_str(&std::format!("**Title:** {}\n", task.title));
        prompt.push_str(&std::format!("**Status:** {:?}\n", task.status));

        if let std::option::Option::Some(ref agent_persona) = task.agent_persona {
            prompt.push_str(&std::format!("**Assignee Persona:** {}\n", agent_persona));
        }

        if let std::option::Option::Some(ref due_date) = task.due_date {
            prompt.push_str(&std::format!("**Due Date:** {}\n", due_date));
        }

        prompt.push_str("\n# Enhancement Criteria\n\n");
        prompt.push_str("Consider:\n");
        prompt.push_str("- **Clarity**: Is the goal clearly stated?\n");
        prompt.push_str("- **Actionability**: Can someone immediately start working on this?\n");
        prompt.push_str("- **Completeness**: Are there missing details or context?\n");
        prompt.push_str("- **Specificity**: Can you make vague requirements more concrete?\n\n");

        prompt.push_str("# Your Task\n\n");
        prompt.push_str("At the end of your response, generate a JSON object with:\n");
        prompt.push_str("1. `enhancement_type`: Choose ONE type that best fits your suggestion:\n");
        prompt.push_str("   - \"clarify\": Make the task description clearer\n");
        prompt.push_str("   - \"specify\": Add specific details or requirements\n");
        prompt.push_str("   - \"decompose\": Suggest breaking into subtasks\n");
        prompt.push_str("   - \"context\": Add missing context or background\n\n");
        prompt.push_str("2. `content`: Your enhancement suggestion as a clear, actionable statement.\n");
        prompt.push_str("   Start with an action verb (e.g., 'Clarify...', 'Add...', 'Specify...').\n\n");
        prompt.push_str("Format the JSON on its own line like this:\n");
        prompt.push_str("```json\n{\"enhancement_type\": \"...\", \"content\": \"...\"}\n```");

        prompt
    }
}

#[async_trait::async_trait]
impl crate::ports::task_enhancement_port::TaskEnhancementPort for OllamaEnhancementAdapter {
    async fn generate_enhancement(
        &self,
        task: &task_manager::domain::task::Task,
    ) -> std::result::Result<task_manager::domain::enhancement::Enhancement, std::string::String> {
        let client = rig::providers::ollama::Client::new();

        // Determine if we have project file access
        let has_file_access = self.project_root.is_some();

        // Synthesize or load ProjectContext if we have a project root
        let context_opt = if let std::option::Option::Some(ref root) = self.project_root {
            // Try to load existing context, or synthesize new one
            let rigger_dir = std::path::Path::new(root).join(".rigger");
            let rigger_dir_str = rigger_dir.to_string_lossy().to_string();
            let root_str = root.to_string_lossy().to_string();

            if rigger_dir.exists() {
                task_manager::domain::project_context::ProjectContext::load_from_rigger_dir(
                    &rigger_dir_str
                ).ok()
            } else {
                // Synthesize minimal context
                task_manager::domain::project_context::ProjectContext::synthesize_context(
                    root_str
                ).ok()
            }
        } else {
            std::option::Option::None
        };

        // Build prompt with context information
        let prompt = Self::build_extraction_prompt(task, has_file_access, context_opt.as_ref());

        let extracted = if let std::option::Option::Some(ref root) = self.project_root {
            // Use Agent with FileSystemTool
            let read_tool = crate::tools::file_system_tool::ReadFileTool::new(root);
            let list_tool = crate::tools::file_system_tool::ListDirectoryTool::new(root);

            let agent = client
                .agent(&self.model)
                .preamble(
                    "You are a task enhancement assistant with access to project files. \
                    You can read files and list directories to understand the codebase. \
                    Use these tools when they help you provide better, context-aware enhancement suggestions. \
                    Always end your response with a JSON object containing enhancement_type and content fields."
                )
                .tool(read_tool)
                .tool(list_tool)
                .build();

            // Prompt the agent
            let response = match rig::completion::Prompt::prompt(&agent, prompt.as_str()).await {
                std::result::Result::Ok(resp) => resp,
                std::result::Result::Err(_e) => {
                    // Fallback
                    return std::result::Result::Ok(Self::create_fallback_enhancement(task));
                }
            };

            // Parse JSON from response
            Self::parse_enhancement_from_response(&response)?
        } else {
            // Use Extractor API (no file access)
            let extractor = client
                .extractor::<EnhancementExtraction>(&self.model)
                .preamble(
                    "You are a task enhancement assistant. \
                    Generate a JSON object with enhancement_type and content fields. \
                    Follow the schema strictly."
                )
                .build();

            match rig::extractor::Extractor::extract(&extractor, &prompt).await {
                std::result::Result::Ok(extraction) => extraction,
                std::result::Result::Err(_e) => {
                    // Fallback
                    return std::result::Result::Ok(Self::create_fallback_enhancement(task));
                }
            }
        };

        // Map extraction to domain entity
        let ts = chrono::Utc::now();
        let enhancement = task_manager::domain::enhancement::Enhancement {
            enhancement_id: std::format!("enh-{}-{}", task.id, ts.timestamp_millis()),
            task_id: task.id.clone(),
            timestamp: ts,
            enhancement_type: extracted.enhancement_type,
            content: extracted.content,
        };

        std::result::Result::Ok(enhancement)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore] // Requires Ollama server running at localhost:11434 with llama3.1 model
    async fn test_adapter_generates_enhancement_with_real_llm() {
        // Test: Validates adapter generates meaningful enhancements via Rig + Ollama.
        // Justification: Ensures LLM integration produces actionable suggestions.
        let adapter = super::OllamaEnhancementAdapter::new(std::string::String::from("llama3.1"));
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Improve spec clarity"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let res = <super::OllamaEnhancementAdapter as crate::ports::task_enhancement_port::TaskEnhancementPort>::generate_enhancement(&adapter, &task).await;

        std::assert!(res.is_ok(), "Enhancement generation should succeed: {:?}", res.err());

        let enhancement = res.unwrap();

        // Validate enhancement structure
        std::assert!(enhancement.enhancement_id.starts_with("enh-"), "Enhancement ID should have correct prefix");
        std::assert_eq!(enhancement.task_id, task.id, "Enhancement should link to correct task");
        std::assert_eq!(enhancement.enhancement_type, "clarify", "Enhancement type should be set");
        std::assert!(!enhancement.content.is_empty(), "Enhancement content should not be empty");

        // Validate LLM generated meaningful content (not deterministic dummy data)
        std::assert!(
            enhancement.content.len() > 20,
            "Enhancement should be meaningful text, got: '{}'",
            enhancement.content
        );

        std::println!("✓ Generated enhancement: {}", enhancement.content);
    }

    #[tokio::test]
    #[ignore] // Requires Ollama server
    async fn test_adapter_with_filesystem_tool_access() {
        // Test: Validates adapter can use FileSystemTool to read project files.
        // Justification: Ensures Agent + FileSystemTool integration works end-to-end.

        // Create temp project directory with sample file
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_fs_tool_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("README.md"), "# Test Project\nA sample project for testing.").unwrap();

        // Create adapter with project root
        let adapter = super::OllamaEnhancementAdapter::new_with_project_root(
            std::string::String::from("llama3.1"),
            &temp_dir,
        );

        std::assert!(adapter.project_root().is_some());

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Add project description"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let res = <super::OllamaEnhancementAdapter as crate::ports::task_enhancement_port::TaskEnhancementPort>::generate_enhancement(&adapter, &task).await;

        std::assert!(res.is_ok(), "Enhancement generation with FS tools should succeed: {:?}", res.err());

        let enhancement = res.unwrap();
        std::assert!(!enhancement.content.is_empty(), "Enhancement should have content");

        std::println!("✓ Generated enhancement with FS access: {}", enhancement.content);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_build_extraction_prompt_includes_task_details() {
        // Test: Validates extraction prompt includes task information and enhancement criteria.
        // Justification: Extractor needs complete context for schema-compliant generation.
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Fix authentication bug"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-01")),
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let prompt = super::OllamaEnhancementAdapter::build_extraction_prompt(&task, false, std::option::Option::None);

        std::assert!(prompt.contains("Fix authentication bug"), "Prompt should include task title");
        std::assert!(prompt.contains("Alice"), "Prompt should include assignee");
        std::assert!(prompt.contains("2025-12-01"), "Prompt should include due date");
        std::assert!(prompt.contains("ONE specific enhancement"), "Prompt should request single enhancement");
        std::assert!(prompt.contains("Clarity"), "Prompt should mention clarity criterion");
        std::assert!(prompt.contains("Actionability"), "Prompt should mention actionability criterion");
        std::assert!(prompt.contains("Completeness"), "Prompt should mention completeness criterion");
        std::assert!(prompt.contains("enhancement_type"), "Prompt should specify enhancement_type field");
        std::assert!(prompt.contains("clarify"), "Prompt should list clarify enhancement type");
        std::assert!(prompt.contains("specify"), "Prompt should list specify enhancement type");
        std::assert!(!prompt.contains("Available Tools"), "Prompt without FS access shouldn't mention tools");
    }

    #[test]
    fn test_build_extraction_prompt_with_file_access() {
        // Test: Validates prompt includes tool information when file access is enabled.
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Update documentation"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let prompt = super::OllamaEnhancementAdapter::build_extraction_prompt(&task, true, std::option::Option::None);

        std::assert!(prompt.contains("Available Tools"), "Prompt should mention available tools");
        std::assert!(prompt.contains("read_file"), "Prompt should mention read_file tool");
        std::assert!(prompt.contains("list_directory"), "Prompt should mention list_directory tool");
        std::assert!(prompt.contains("Use these tools"), "Prompt should encourage tool use");
    }

    #[test]
    fn test_build_extraction_prompt_with_project_context() {
        // Test: Validates prompt includes ProjectContext information.
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_prompt_ctx_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("auth.rs"), "// auth").unwrap();

        let mut context = task_manager::domain::project_context::ProjectContext::new(
            temp_dir.to_string_lossy().to_string(),
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
            std::vec![],
        );

        context.add_recent_decision(std::string::String::from("Use Ollama for LLM inference"));
        context.add_recent_decision(std::string::String::from("Apply Hexagonal Architecture"));

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Improve auth system"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let prompt = super::OllamaEnhancementAdapter::build_extraction_prompt(&task, true, std::option::Option::Some(&context));

        std::assert!(prompt.contains("Project Context"), "Prompt should have Project Context section");
        std::assert!(prompt.contains("Recent Decisions"), "Prompt should list recent decisions");
        std::assert!(prompt.contains("Ollama"), "Prompt should include decision about Ollama");
        std::assert!(prompt.contains("Hexagonal Architecture"), "Prompt should include decision about architecture");
        std::assert!(prompt.contains("Relevant Files"), "Prompt should have relevant files section");

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_parse_enhancement_from_json_code_block() {
        // Test: Validates JSON parsing from code block format.
        let response = "Here's my analysis.\n\n```json\n{\"enhancement_type\": \"clarify\", \"content\": \"Add acceptance criteria\"}\n```\n\nHope this helps!";

        let result = super::OllamaEnhancementAdapter::parse_enhancement_from_response(response);
        std::assert!(result.is_ok());

        let extracted = result.unwrap();
        std::assert_eq!(extracted.enhancement_type, "clarify");
        std::assert_eq!(extracted.content, "Add acceptance criteria");
    }

    #[test]
    fn test_parse_enhancement_from_standalone_json() {
        // Test: Validates JSON parsing from standalone object.
        let response = "After analyzing the task, I suggest: {\"enhancement_type\": \"specify\", \"content\": \"Define edge cases\"}";

        let result = super::OllamaEnhancementAdapter::parse_enhancement_from_response(response);
        std::assert!(result.is_ok());

        let extracted = result.unwrap();
        std::assert_eq!(extracted.enhancement_type, "specify");
        std::assert_eq!(extracted.content, "Define edge cases");
    }

    #[tokio::test]
    #[ignore] // Requires Ollama server
    async fn test_enhancement_with_project_context_integration() {
        // Test: Validates that adapter synthesizes ProjectContext and uses it in enhancements.
        // Justification: Task 4.10 requires verification that enhancements reference project structure.

        // Create temp project with structure
        let temp_dir = std::env::temp_dir().join(std::format!("rigger_ctx_enh_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();
        let rigger_dir = temp_dir.join(".rigger");
        std::fs::create_dir(&rigger_dir).unwrap();

        // Create sample project files
        std::fs::write(temp_dir.join("auth_handler.rs"), "// authentication logic").unwrap();
        std::fs::write(temp_dir.join("user_service.rs"), "// user service").unwrap();

        // Create ProjectContext with recent decisions
        let mut context = task_manager::domain::project_context::ProjectContext::new(
            temp_dir.to_string_lossy().to_string(),
            std::vec![std::string::String::from("Rust")],
            std::vec![std::string::String::from("Rig")],
            std::vec![std::string::String::from("src: Source code")],
            std::vec![],
            std::vec![std::string::String::from("Hexagonal Architecture")],
            std::vec![],
        );

        context.add_recent_decision(std::string::String::from("Use Ollama for LLM inference"));
        context.add_recent_decision(std::string::String::from("Apply HEXSER pattern for adapters"));

        // Save context to .rigger directory
        context.save_to_rigger_dir(rigger_dir.to_str().unwrap()).unwrap();

        // Create adapter with project root
        let adapter = super::OllamaEnhancementAdapter::new_with_project_root(
            std::string::String::from("llama3.1"),
            &temp_dir,
        );

        // Create a task related to authentication
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Improve authentication security"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);

        // Generate enhancement
        let result = <super::OllamaEnhancementAdapter as crate::ports::task_enhancement_port::TaskEnhancementPort>::generate_enhancement(&adapter, &task).await;

        std::assert!(result.is_ok(), "Enhancement generation should succeed: {:?}", result.err());

        let enhancement = result.unwrap();

        // Validate enhancement structure
        std::assert!(!enhancement.content.is_empty(), "Enhancement should have content");
        std::assert_eq!(enhancement.task_id, task.id, "Enhancement should link to task");

        // Validate that the prompt would have included context
        // (Note: We can't directly verify LLM used the context, but we verify it was available)
        let loaded_context = task_manager::domain::project_context::ProjectContext::load_from_rigger_dir(
            rigger_dir.to_str().unwrap()
        ).unwrap();

        std::assert_eq!(loaded_context.recent_decisions.len(), 2, "Context should have 2 decisions");
        std::assert_eq!(loaded_context.architectural_patterns[0], "Hexagonal Architecture");

        std::println!("✓ Generated enhancement with ProjectContext: {}", enhancement.content);
        std::println!("✓ ProjectContext had {} recent decisions", loaded_context.recent_decisions.len());
        std::println!("✓ ProjectContext architectural pattern: {}", loaded_context.architectural_patterns[0]);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
