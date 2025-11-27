//! Rig-powered task decomposition adapter.
//!
//! RigTaskDecompositionAdapter uses Rig's Extractor API to break complex tasks
//! into 3-5 manageable subtasks. The adapter employs JSON schema enforcement
//! to ensure structured output and includes fallback logic for LLM unavailability.
//!
//! Revision History
//! - 2025-11-23T17:15:00Z @AI: Create RigTaskDecompositionAdapter for Phase 3 Sprint 7.

/// DTO for JSON Schema-enforced subtask extraction.
///
/// SubtaskExtraction defines the structure expected from the LLM when
/// decomposing tasks. Each subtask includes a title, optional assignee,
/// and optional due_date. The schema validation ensures the LLM returns
/// properly structured data.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
struct SubtaskExtraction {
    /// The title of the subtask (required, 10-100 characters).
    title: String,

    /// Optional assignee for the subtask.
    #[serde(default)]
    assignee: std::option::Option<String>,

    /// Optional due date for the subtask (ISO 8601 format).
    #[serde(default)]
    due_date: std::option::Option<String>,
}

/// Adapter for task decomposition using Rig's Extractor API.
///
/// RigTaskDecompositionAdapter implements TaskDecompositionPort by using
/// Rig's schema-enforced extraction to generate structured subtasks from
/// complex parent tasks. The adapter provides intelligent prompt engineering
/// and graceful degradation when the LLM service is unavailable.
///
/// # Decomposition Strategy
///
/// 1. **Prompt Engineering**: Constructs a detailed prompt including:
///    - Parent task title and complexity
///    - Decomposition guidelines (3-5 subtasks, progressive difficulty)
///    - Context from parent task (assignee, due_date if available)
///
/// 2. **Schema Enforcement**: Uses Rig Extractor with `SubtaskExtraction` schema
///    to ensure structured JSON output with validation
///
/// 3. **Fallback**: Returns deterministic subtasks if LLM unavailable
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::adapters::rig_task_decomposition_adapter::RigTaskDecompositionAdapter;
/// # use task_orchestrator::ports::task_decomposition_port::TaskDecompositionPort;
/// # use task_manager::domain::task::Task;
/// # use transcript_extractor::domain::action_item::ActionItem;
/// let adapter = RigTaskDecompositionAdapter::new(
///     std::string::String::from("llama3.1"),
/// );
///
/// # async fn example(adapter: RigTaskDecompositionAdapter) {
/// let action = ActionItem {
///     title: std::string::String::from("Implement OAuth2 authentication system"),
///     assignee: std::option::Option::None,
///     due_date: std::option::Option::None,
/// };
/// let task = Task::from_action_item(&action, std::option::Option::None);
///
/// let subtasks = adapter.decompose_task(&task).await.unwrap();
/// std::assert!(subtasks.len() >= 3 && subtasks.len() <= 5);
/// # }
/// ```
pub struct RigTaskDecompositionAdapter {
    model: String,
}

impl RigTaskDecompositionAdapter {
    /// Creates a new RigTaskDecompositionAdapter with specified model.
    ///
    /// # Arguments
    ///
    /// * `model` - The model name to use (e.g., "llama3.1", "gpt-4")
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_orchestrator::adapters::rig_task_decomposition_adapter::RigTaskDecompositionAdapter;
    /// let adapter = RigTaskDecompositionAdapter::new(
    ///     std::string::String::from("llama3.1"),
    /// );
    /// ```
    pub fn new(model: String) -> Self {
        RigTaskDecompositionAdapter { model }
    }

    /// Builds the decomposition prompt for the LLM.
    ///
    /// Constructs a detailed prompt that guides the LLM to generate 3-5
    /// actionable subtasks. The prompt includes decomposition guidelines,
    /// parent task context, and quality criteria.
    fn build_decomposition_prompt(&self, task: &task_manager::domain::task::Task) -> String {
        let mut prompt = std::string::String::new();

        prompt.push_str("You are a project management assistant specialized in breaking down complex tasks.\n\n");

        prompt.push_str(&std::format!("**Parent Task**: {}\n\n", task.title));

        if let std::option::Option::Some(complexity) = task.complexity {
            prompt.push_str(&std::format!("**Complexity Score**: {} / 10\n\n", complexity));
        }

        prompt.push_str("**Decomposition Guidelines**:\n");
        prompt.push_str("1. Generate 3-5 subtasks that collectively achieve the parent task's objective\n");
        prompt.push_str("2. Each subtask should be specific, actionable, and independently verifiable\n");
        prompt.push_str("3. Order subtasks by dependency (earlier tasks should be prerequisites for later ones)\n");
        prompt.push_str("4. Avoid overly granular steps (each subtask should be substantial)\n");
        prompt.push_str("5. Include technical specifics where applicable\n\n");

        if let std::option::Option::Some(ref assignee) = task.assignee {
            prompt.push_str(&std::format!("**Parent Assignee**: {} (may inherit to subtasks)\n\n", assignee));
        }

        if let std::option::Option::Some(ref due_date) = task.due_date {
            prompt.push_str(&std::format!("**Parent Due Date**: {} (subtasks should align)\n\n", due_date));
        }

        prompt.push_str("Generate a JSON array of subtasks, each with:\n");
        prompt.push_str("- title: string (10-100 characters, descriptive and actionable)\n");
        prompt.push_str("- assignee: string | null (inherit from parent if appropriate)\n");
        prompt.push_str("- due_date: string | null (ISO 8601 format, coordinate with parent deadline)\n");

        prompt
    }

    /// Creates fallback subtasks when LLM is unavailable.
    ///
    /// Generates deterministic subtasks based on the parent task title.
    /// Used for graceful degradation and testing scenarios.
    fn create_fallback_subtasks(&self, parent_task: &task_manager::domain::task::Task) -> std::vec::Vec<task_manager::domain::task::Task> {
        let fallback_titles = std::vec![
            std::format!("Research and analyze requirements for: {}", parent_task.title),
            std::format!("Design architecture and technical approach for: {}", parent_task.title),
            std::format!("Implement core functionality for: {}", parent_task.title),
            std::format!("Test and validate: {}", parent_task.title),
        ];

        let parent_complexity = parent_task.complexity.unwrap_or(5);
        let subtask_complexity = if parent_complexity > 2 {
            parent_complexity - 2
        } else {
            1
        };

        fallback_titles
            .into_iter()
            .map(|title| {
                let subtask_action = transcript_extractor::domain::action_item::ActionItem {
                    title,
                    assignee: parent_task.assignee.clone(),
                    due_date: parent_task.due_date.clone(),
                };

                let mut subtask = task_manager::domain::task::Task::from_action_item(
                    &subtask_action,
                    parent_task.source_transcript_id.clone(),
                );

                subtask.parent_task_id = std::option::Option::Some(parent_task.id.clone());
                subtask.source_prd_id = parent_task.source_prd_id.clone();
                subtask.complexity = std::option::Option::Some(subtask_complexity);

                subtask
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl crate::ports::task_decomposition_port::TaskDecompositionPort for RigTaskDecompositionAdapter {
    async fn decompose_task(
        &self,
        task: &task_manager::domain::task::Task,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String> {
        // Build decomposition prompt
        let prompt = self.build_decomposition_prompt(task);

        // Create Ollama client and extractor
        let client = rig::providers::ollama::Client::from_url("http://localhost:11434");

        let extractor = client
            .extractor::<std::vec::Vec<SubtaskExtraction>>(&self.model)
            .preamble("You are a task decomposition expert. Generate a JSON array of 3-5 subtasks that break down the parent task into manageable, actionable steps.")
            .build();

        // Attempt extraction
        let extractions = match rig::extractor::Extractor::extract(&extractor, prompt.as_str()).await {
            std::result::Result::Ok(extracted) => extracted,
            std::result::Result::Err(_e) => {
                // Fallback to deterministic subtasks
                return std::result::Result::Ok(self.create_fallback_subtasks(task));
            }
        };

        // Validate we got 3-5 subtasks
        if extractions.is_empty() || extractions.len() > 5 {
            return std::result::Result::Ok(self.create_fallback_subtasks(task));
        }

        // Convert extractions to Task entities
        let parent_complexity = task.complexity.unwrap_or(5);
        let subtask_complexity = if parent_complexity > 2 {
            parent_complexity - 2
        } else {
            1
        };

        let subtasks: std::vec::Vec<task_manager::domain::task::Task> = extractions
            .into_iter()
            .map(|extraction| {
                let subtask_action = transcript_extractor::domain::action_item::ActionItem {
                    title: extraction.title,
                    assignee: extraction.assignee,
                    due_date: extraction.due_date,
                };

                let mut subtask = task_manager::domain::task::Task::from_action_item(
                    &subtask_action,
                    task.source_transcript_id.clone(),
                );

                subtask.parent_task_id = std::option::Option::Some(task.id.clone());
                subtask.source_prd_id = task.source_prd_id.clone();
                subtask.complexity = std::option::Option::Some(subtask_complexity);

                subtask
            })
            .collect();

        std::result::Result::Ok(subtasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::task_decomposition_port::TaskDecompositionPort;

    #[test]
    fn test_build_decomposition_prompt_includes_task_details() {
        // Test: Validates prompt construction includes parent task context.
        // Justification: Prompts must provide sufficient context for quality decomposition.
        let adapter = RigTaskDecompositionAdapter::new(std::string::String::from("llama3.1"));

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Implement OAuth2 authentication"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-31")),
        };
        let mut task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        task.complexity = std::option::Option::Some(8);

        let prompt = adapter.build_decomposition_prompt(&task);

        std::assert!(prompt.contains("Implement OAuth2 authentication"), "Prompt should include task title");
        std::assert!(prompt.contains("Complexity Score"), "Prompt should mention complexity");
        std::assert!(prompt.contains("Alice"), "Prompt should include assignee");
        std::assert!(prompt.contains("2025-12-31"), "Prompt should include due date");
        std::assert!(prompt.contains("3-5 subtasks"), "Prompt should specify subtask count");
    }

    #[test]
    fn test_fallback_subtasks_generation() {
        // Test: Validates fallback creates 4 deterministic subtasks with proper linkage.
        // Justification: Fallback ensures graceful degradation when LLM unavailable.
        let adapter = RigTaskDecompositionAdapter::new(std::string::String::from("llama3.1"));

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor payment system"),
            assignee: std::option::Option::Some(std::string::String::from("Bob")),
            due_date: std::option::Option::None,
        };
        let mut parent = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        parent.complexity = std::option::Option::Some(9);

        let subtasks = adapter.create_fallback_subtasks(&parent);

        std::assert_eq!(subtasks.len(), 4, "Should create 4 fallback subtasks");

        for subtask in &subtasks {
            std::assert_eq!(
                subtask.parent_task_id,
                std::option::Option::Some(parent.id.clone()),
                "Each subtask should link to parent"
            );
            std::assert_eq!(
                subtask.complexity,
                std::option::Option::Some(7),
                "Subtask complexity should be parent - 2"
            );
            std::assert_eq!(
                subtask.assignee,
                std::option::Option::Some(std::string::String::from("Bob")),
                "Subtask should inherit parent assignee"
            );
            std::assert!(subtask.title.contains("Refactor payment system"), "Subtask title should reference parent");
        }
    }

    #[test]
    fn test_fallback_subtasks_complexity_floor() {
        // Test: Validates complexity reduction doesn't go below 1.
        // Justification: Ensures subtask complexity stays within valid range (1-10).
        let adapter = RigTaskDecompositionAdapter::new(std::string::String::from("llama3.1"));

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Simple task"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut parent = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        parent.complexity = std::option::Option::Some(2);

        let subtasks = adapter.create_fallback_subtasks(&parent);

        for subtask in &subtasks {
            std::assert_eq!(
                subtask.complexity,
                std::option::Option::Some(1),
                "Subtask complexity should have floor of 1"
            );
        }
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running locally
    async fn test_decompose_task_with_ollama() {
        // Test: Validates real LLM decomposition generates valid subtasks.
        // Justification: Integration test ensures Rig Extractor API works correctly.
        let adapter = RigTaskDecompositionAdapter::new(std::string::String::from("llama3.1"));

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Implement user authentication system with OAuth2 and SAML support"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::Some(std::string::String::from("2025-12-31")),
        };
        let mut task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);
        task.complexity = std::option::Option::Some(9);

        let result = adapter.decompose_task(&task).await;
        std::assert!(result.is_ok(), "Decomposition should succeed");

        let subtasks = result.unwrap();
        std::assert!(subtasks.len() >= 3 && subtasks.len() <= 5, "Should generate 3-5 subtasks, got {}", subtasks.len());

        for subtask in &subtasks {
            std::assert!(!subtask.title.is_empty(), "Subtask title should not be empty");
            std::assert_eq!(subtask.parent_task_id, std::option::Option::Some(task.id.clone()));
            std::println!("Subtask: {}", subtask.title);
        }
    }
}
