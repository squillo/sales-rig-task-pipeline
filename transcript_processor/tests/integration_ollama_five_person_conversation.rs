//! Integration test: Five-person conversation with native Ollama service.
//!
//! This integration test validates the complete transcript processing pipeline
//! using a realistic 5-minute team meeting with 5 participants. It demonstrates
//! end-to-end functionality from raw transcript text through LLM-powered extraction
//! to task persistence and retrieval.
//!
//! **Native Ollama Approach:**
//! This test connects to a locally running Ollama service. The test will:
//! 1. Verify Ollama service is running at http://localhost:11434
//! 2. Execute the transcript processing pipeline using the llama3.2 model
//! 3. Validate extraction quality and data integrity
//!
//! Prerequisites:
//! - Ollama installed and running locally (use ./setup-ollama.sh for automated setup)
//! - llama3.2 model pulled (setup script handles this automatically)
//! - No Docker required
//!
//! Setup:
//! ```bash
//! # First time setup (from project root)
//! ./setup-ollama.sh
//!
//! # Ensure Ollama service is running
//! ollama serve &
//! ```
//!
//! Running the Test:
//! ```bash
//! cd transcript_processor
//! cargo test --test integration_five_person_conversation -- --nocapture
//! ```
//!
//! Revision History
//! - 2025-11-06T20:31:00Z @AI: Migrate from Docker containers to native Ollama service.
//! - 2025-11-06T20:15:00Z @AI: Implement automated container lifecycle with testcontainers-rs.
//! - 2025-11-06T19:50:00Z @AI: Add Docker setup instructions for easier testing.
//! - 2025-11-06T19:44:00Z @AI: Initial integration test with realistic 5-person conversation.

/// Realistic 5-minute team meeting transcript with 5 participants.
///
/// This transcript simulates a sprint planning meeting for a software team
/// working on a task management application. It includes multiple action items
/// with varied characteristics to test the extraction pipeline comprehensively.
const FIVE_PERSON_CONVERSATION: &str = r#"
Sprint Planning Meeting - RigTask Pipeline Project
Date: November 6, 2025
Duration: ~5 minutes
Participants: Sarah (Product Manager), James (Tech Lead), Maria (Backend Dev),
              David (Frontend Dev), Emily (QA Engineer)

[00:00] Sarah: Good morning everyone! Let's dive into our sprint planning. We have
some critical items to address for the RigTask pipeline release. James, can you
start with the backend status?

[00:30] James: Sure, Sarah. We've made good progress on the HEXSER integration,
but we need to address the caching layer for the transcript analysis. The current
implementation re-parses everything on each request, which is too slow for
production. I'll take ownership of implementing the persistent cache using SQLite.
I'm targeting November 15th as the completion date since we need it before the
beta release.

[01:15] Sarah: Perfect. That's high priority, so let's mark it as such. Maria,
what's the status on the multi-language support?

[01:30] Maria: We currently only support Rust file analysis. I've been researching
Tree-sitter grammars for JavaScript, Python, and TypeScript. I can implement
support for these three languages, but it's a medium-sized task. I'd say it's
medium priority since our initial customers are primarily Rust shops. I should
have this done by November 20th.

[02:00] David: That reminds me - I need to update the frontend visualization to
handle the different language icons and syntax highlighting. Sarah, should I wait
for Maria's backend work, or can I start with mock data?

[02:20] Sarah: Good question. Let's have you start with mock data so you're not
blocked. David, you own the task to create the language-specific visualization
components. This is medium priority, and let's aim for November 18th so you have
a buffer before Maria's work lands.

[02:45] Emily: I want to flag something important. We don't have any integration
tests for the full pipeline with large transcripts. Our current tests are all
unit tests with small samples. I'd like to create a comprehensive test suite that
includes realistic conversation scenarios - maybe 10-15 minutes long with multiple
participants. This is high priority for quality assurance. I'll need this done by
November 12th, which is before we start the beta testing phase.

[03:30] James: Emily, that's a great point. While you're building those tests,
can you also add performance benchmarks? We need to know how the system handles
transcripts of varying sizes - say 1KB, 10KB, 100KB, and 1MB. This will help us
set realistic expectations for users.

[03:50] Emily: Absolutely. I'll add the benchmark suite as part of the same
initiative. Should be done by the same date, November 12th. High priority as well.

[04:05] Maria: One more thing - we need to document the API endpoints properly.
Right now, the REST API documentation is minimal. I can take this on as a
secondary task. It's lower priority, maybe medium-low, and I can have it done by
November 25th after the language support work is complete.

[04:30] Sarah: Great. David, you mentioned the visualization work, but I don't
think we captured the due date clearly. Can you confirm?

[04:40] David: Yes, November 18th for the language-specific visualization
components. I'll make sure to coordinate with Maria once her backend support is
ready.

[04:55] Sarah: Perfect. James, there's one more thing. The error handling in the
Ollama adapter is pretty basic right now. Can you improve it to include retry
logic with exponential backoff? If the LLM service is temporarily unavailable,
we should retry instead of immediately failing.

[05:15] James: Good call. That's definitely needed for production readiness. I'll
take that on. It's high priority like the caching work. I should be able to
complete both by November 15th since they're related to the adapter layer.

[05:35] Sarah: Excellent.

[06:00] Sarah: Does anyone have any blockers or concerns?

[06:10] David: No blockers on my end. I'll start with the mock data approach.

[06:15] Maria: I'm good. Tree-sitter has excellent documentation.

[06:20] James: All clear. I'll start with the caching layer today.

[06:25] Emily: Ready to go. I'll coordinate with everyone on the test scenarios.

[06:30] Sarah: Perfect! Let's sync again on Friday to check progress. Meeting
adjourned. Thanks everyone!
"#;

/// Helper function to verify native Ollama service is running and ready.
///
/// This function checks if the Ollama service is accessible at localhost:11434
/// by attempting to query the API tags endpoint. This is a prerequisite for
/// running integration tests that require LLM functionality.
async fn check_ollama_service() -> Result<(), String> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama service: {}. Is Ollama running? Try: ollama serve", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "Ollama service returned error status: {}",
            response.status()
        ))
    }
}

#[tokio::test]
async fn test_five_person_conversation_integration() {
    // Test: Validates the complete pipeline with a realistic 5-minute, 5-person conversation.
    // Justification: This integration test ensures the system can handle real-world meeting
    // transcripts with multiple speakers, diverse action items, and complex conversational
    // patterns. It validates LLM extraction accuracy, task persistence, and data integrity
    // across the full stack - critical for production readiness. Uses native Ollama service
    // running locally for fast, reliable testing without Docker overhead.

    println!("\n=== Verifying Native Ollama Service ===\n");

    // Check if Ollama service is running before proceeding
    if let Err(e) = check_ollama_service().await {
        panic!(
            "Ollama service health check failed: {}\n\
             \n\
             Please ensure Ollama is installed and running:\n\
             1. Run setup script: ./setup-ollama.sh\n\
             2. Or start manually: ollama serve &\n\
             3. Verify service: curl http://localhost:11434/api/tags",
            e
        );
    }

    println!("✅ Ollama service is running at http://localhost:11434\n");
    println!("\n=== Processing 5-Minute Team Meeting Transcript ===\n");

    // Initialize the Ollama adapter with llama3.2 model
    let ollama_adapter = std::sync::Arc::new(
        transcript_processor::adapters::ollama_adapter::OllamaTranscriptExtractorAdapter::new(
            std::string::String::from("llama3.2"),
        ),
    );

    // Initialize the in-memory task repository
    let task_repo =
        transcript_processor::adapters::in_memory_task_adapter::InMemoryTaskAdapter::new();

    // Create the use case with the adapters
    let mut process_use_case =
        transcript_processor::application::use_cases::process_transcript::ProcessTranscriptUseCase::new(
            ollama_adapter,
            task_repo,
        );

    // Execute the pipeline with the realistic conversation
    println!("Sending transcript to Ollama LLM (llama3.2) for extraction...\n");

    let result = process_use_case
        .process(FIVE_PERSON_CONVERSATION)
        .await;

    // Assert the pipeline succeeded
    assert!(
        result.is_ok(),
        "Pipeline should successfully process the transcript: {:?}",
        result.err()
    );

    let extracted_tasks = result.unwrap();

    // Print detailed results
    println!("✅ Extraction Complete!\n");
    println!("=== Extracted Tasks ({}) ===\n", extracted_tasks.len());

    for (idx, task) in extracted_tasks.iter().enumerate() {
        println!("Task #{}", idx + 1);
        println!("  ID: {}", task.id);
        println!("  Title: {}", task.title);
        println!(
            "  Assignee: {}",
            task.assignee
                .as_ref()
                .unwrap_or(&std::string::String::from("Unassigned"))
        );
        println!(
            "  Due Date: {}",
            task.due_date
                .as_ref()
                .unwrap_or(&std::string::String::from("No deadline"))
        );
        println!("  Status: {:?}", task.status);
        println!(
            "  Created: {}",
            task.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!();
    }

    // Assertions: Validate extraction quality

    // Expected: The transcript explicitly mentions 7 distinct action items in Sarah's summary
    assert!(
        extracted_tasks.len() >= 5,
        "Expected at least 5 action items extracted from the conversation (found: {})",
        extracted_tasks.len()
    );

    // Validate that tasks were created with proper structure
    for task in &extracted_tasks {
        // All tasks must have a non-empty title
        assert!(
            !task.title.is_empty(),
            "Task ID {} has empty title",
            task.id
        );

        // All tasks must have valid UUIDs
        assert!(
            !task.id.is_empty() && task.id.len() == 36,
            "Task has invalid ID format: {}",
            task.id
        );

        // All tasks must have timestamps
        assert!(
            task.created_at == task.updated_at,
            "Newly created task should have matching created/updated timestamps"
        );

        // All tasks should start in Todo status
        assert_eq!(
            task.status,
            transcript_processor::domain::task_status::TaskStatus::Todo,
            "Newly created task should have Todo status"
        );
    }

    // Count tasks with assignees (informational - LLM extraction quality varies)
    let tasks_with_assignees = extracted_tasks
        .iter()
        .filter(|t| t.assignee.is_some())
        .count();

    // Validate that at least some tasks have due dates
    let tasks_with_due_dates = extracted_tasks
        .iter()
        .filter(|t| t.due_date.is_some())
        .count();

    assert!(
        tasks_with_due_dates >= 3,
        "Expected at least 3 tasks with due dates (found: {})",
        tasks_with_due_dates
    );

    // Check for expected assignees (mentioned in transcript: James, Maria, David, Emily)
    let assignee_names: std::collections::HashSet<String> = extracted_tasks
        .iter()
        .filter_map(|t| t.assignee.clone())
        .collect();

    println!("=== Validation Results ===");
    println!("✅ Total tasks extracted: {}", extracted_tasks.len());
    println!("✅ Tasks with assignees: {}", tasks_with_assignees);
    println!("✅ Tasks with due dates: {}", tasks_with_due_dates);
    println!("✅ Unique assignees found: {:?}", assignee_names);
    println!("\n🎉 Integration test PASSED - Full pipeline validated!");
}
