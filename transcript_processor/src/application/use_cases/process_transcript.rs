//! Defines the ProcessTranscriptUseCase for orchestrating transcript analysis.
//!
//! This use case coordinates the extraction of action items from transcripts and
//! their conversion into persistent tasks. It demonstrates the Hexagonal Architecture
//! by depending on port interfaces rather than concrete implementations.
//!
//! Revision History
//! - 2025-11-06T18:56:00Z @AI: Update adapter name to OllamaTranscriptExtractorAdapter for clarity.
//! - 2025-11-06T18:30:00Z @AI: Refactor to use generic concrete repository type (HEXSER pattern).
//! - 2025-11-06T17:41:00Z @AI: Initial ProcessTranscriptUseCase implementation.

/// Use case for processing transcripts and creating tasks.
///
/// ProcessTranscriptUseCase orchestrates the workflow of analyzing a transcript,
/// extracting action items, converting them to tasks, and persisting them with
/// full history tracking.
///
/// This use case is generic over the repository type to enable compile-time
/// polymorphism and mutable access to the repository for HEXSER's save() method.
///
/// # Type Parameters
///
/// * `R` - The concrete repository type implementing TaskRepositoryPort.
///
/// # Examples
///
/// ```no_run
/// # use transcript_processor::application::use_cases::process_transcript::ProcessTranscriptUseCase;
/// # use transcript_processor::adapters::ollama_adapter::OllamaTranscriptExtractorAdapter;
/// # use transcript_processor::adapters::in_memory_task_adapter::InMemoryTaskAdapter;
/// # async fn example() {
/// let extractor = std::sync::Arc::new(OllamaTranscriptExtractorAdapter::new(std::string::String::from("llama3.2")));
/// let repo = InMemoryTaskAdapter::new();
/// let mut use_case = ProcessTranscriptUseCase::new(extractor, repo);
/// let transcript = "Meeting notes: Alice will review the PR by Friday.";
/// let tasks = use_case.process(transcript).await.unwrap();
/// assert!(!tasks.is_empty());
/// # }
/// ```
pub struct ProcessTranscriptUseCase<R>
where
    R: crate::application::ports::task_repository_port::TaskRepositoryPort,
{
    extractor: std::sync::Arc<dyn crate::application::ports::transcript_extractor_port::TranscriptExtractorPort>,
    task_repo: R,
}

impl<R> ProcessTranscriptUseCase<R>
where
    R: crate::application::ports::task_repository_port::TaskRepositoryPort,
{
    /// Creates a new ProcessTranscriptUseCase with the provided ports.
    ///
    /// # Arguments
    ///
    /// * `extractor` - The transcript extraction port implementation (shared via Arc).
    /// * `task_repo` - The concrete task repository implementation (owned).
    ///
    /// # Returns
    ///
    /// A new ProcessTranscriptUseCase instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use transcript_processor::application::use_cases::process_transcript::ProcessTranscriptUseCase;
    /// # use transcript_processor::adapters::ollama_adapter::OllamaTranscriptExtractorAdapter;
    /// # use transcript_processor::adapters::in_memory_task_adapter::InMemoryTaskAdapter;
    /// let extractor = std::sync::Arc::new(OllamaTranscriptExtractorAdapter::new(std::string::String::from("llama3.2")));
    /// let repo = InMemoryTaskAdapter::new();
    /// let use_case = ProcessTranscriptUseCase::new(extractor, repo);
    /// ```
    pub fn new(
        extractor: std::sync::Arc<dyn crate::application::ports::transcript_extractor_port::TranscriptExtractorPort>,
        task_repo: R,
    ) -> Self {
        ProcessTranscriptUseCase {
            extractor,
            task_repo,
        }
    }

    /// Processes a transcript and creates tasks from extracted action items.
    ///
    /// This method orchestrates the complete workflow:
    /// 1. Extracts action items from the transcript via the extractor port
    /// 2. Converts each ActionItem to a Task with generated UUID and timestamps
    /// 3. Persists each task via the repository port using HEXSER's save() method
    ///
    /// # Arguments
    ///
    /// * `transcript` - The raw transcript text to process.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Task>)` - All successfully created and persisted tasks.
    /// * `Err(String)` - Error message if processing fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use transcript_processor::application::use_cases::process_transcript::ProcessTranscriptUseCase;
    /// # use transcript_processor::adapters::ollama_adapter::OllamaTranscriptExtractorAdapter;
    /// # use transcript_processor::adapters::in_memory_task_adapter::InMemoryTaskAdapter;
    /// # async fn example() {
    /// let extractor = std::sync::Arc::new(OllamaTranscriptExtractorAdapter::new(std::string::String::from("llama3.2")));
    /// let repo = InMemoryTaskAdapter::new();
    /// let mut use_case = ProcessTranscriptUseCase::new(extractor, repo);
    /// let transcript = "Alice will complete the design doc by Monday.";
    /// let tasks = use_case.process(transcript).await.unwrap();
    /// println!("Created {} tasks", tasks.len());
    /// # }
    /// ```
    pub async fn process(
        &mut self,
        transcript: &str,
    ) -> std::result::Result<Vec<crate::domain::task::Task>, std::string::String> {
        // Extract action items from the transcript
        let analysis = self.extractor.extract_analysis(transcript).await?;

        let mut created_tasks = Vec::new();

        // Convert each action item to a task and persist it
        for action_item in &analysis.action_items {
            let task = crate::domain::task::Task::from_action_item(action_item, None);

            // Persist the task using HEXSER Repository trait's save() method
            self.task_repo
                .save(task.clone())
                .map_err(|e| std::format!("Failed to save task: {:?}", e))?;

            created_tasks.push(task);
        }

        std::result::Result::Ok(created_tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExtractor;

    #[async_trait::async_trait]
    impl crate::application::ports::transcript_extractor_port::TranscriptExtractorPort for MockExtractor {
        async fn extract_analysis(
            &self,
            _transcript: &str,
        ) -> std::result::Result<crate::domain::transcript_analysis::TranscriptAnalysis, std::string::String> {
            std::result::Result::Ok(crate::domain::transcript_analysis::TranscriptAnalysis {
                action_items: vec![
                    crate::domain::action_item::ActionItem {
                        title: std::string::String::from("Test action"),
                        assignee: None,
                        due_date: None,
                    },
                ],
            })
        }
    }

    struct MockRepo {
        tasks: std::collections::HashMap<String, crate::domain::task::Task>,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                tasks: std::collections::HashMap::new(),
            }
        }
    }

    // Implement HEXSER Repository trait for write operations
    impl hexser::ports::Repository<crate::domain::task::Task> for MockRepo {
        fn save(
            &mut self,
            entity: crate::domain::task::Task,
        ) -> hexser::HexResult<()> {
            self.tasks.insert(entity.id.clone(), entity);
            std::result::Result::Ok(())
        }
    }

    // Implement HEXSER QueryRepository trait for read operations
    impl hexser::ports::repository::QueryRepository<crate::domain::task::Task> for MockRepo {
        type Filter = crate::application::ports::task_repository_port::TaskFilter;
        type SortKey = crate::application::ports::task_repository_port::TaskSortKey;

        fn find_one(
            &self,
            _filter: &Self::Filter,
        ) -> hexser::HexResult<std::option::Option<crate::domain::task::Task>> {
            std::result::Result::Ok(std::option::Option::None)
        }

        fn find(
            &self,
            _filter: &Self::Filter,
            _opts: hexser::ports::repository::FindOptions<Self::SortKey>,
        ) -> hexser::HexResult<std::vec::Vec<crate::domain::task::Task>> {
            std::result::Result::Ok(Vec::new())
        }
    }

    // Implement the marker trait to satisfy TaskRepositoryPort bounds
    impl crate::application::ports::task_repository_port::TaskRepositoryPort for MockRepo {}

    #[tokio::test]
    async fn test_process_transcript_creates_tasks() {
        // Test: Validates that the ProcessTranscriptUseCase correctly orchestrates the full pipeline.
        // Justification: Ensures the use case properly extracts action items via the extractor port,
        // converts them to tasks, and persists them via the repository port - the core workflow.
        let extractor = std::sync::Arc::new(MockExtractor);
        let repo = MockRepo::new();
        let mut use_case = ProcessTranscriptUseCase::new(extractor, repo);

        let result = use_case.process("test transcript").await;
        assert!(result.is_ok());

        let tasks = result.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Test action");
    }
}
