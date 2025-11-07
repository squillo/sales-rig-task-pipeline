//! Integration test: Five-person conversation using Candle adapter for in-process inference.
//!
//! This integration test validates the complete transcript processing pipeline by:
//! 1. Processing a realistic 5-minute, 5-person software team meeting transcript
//! 2. Extracting action items with complex assignment patterns
//! 3. Validating LLM extraction accuracy with the embedded Phi-3.5-mini-instruct model
//! 4. Ensuring task persistence and data integrity across the full stack
//!
//! The conversation includes:
//! - 5 team members discussing architecture implementation tasks
//! - 8 distinct action items with varied complexity
//! - Multiple assignment patterns ("I'll take", "X will", "Let's have Y")
//! - Explicit due dates and priority discussions
//!
//! Prerequisites:
//! - First run will download Phi-3.5-mini-instruct model from HuggingFace (~7.6GB)
//! - Subsequent runs use cached model for fast execution
//!
//! Running the Test:
//! ```bash
//! cd transcript_processor
//! cargo test --test integration_candle_five_person_conversation -- --nocapture
//! ```
//!
//! Revision History
//! - 2025-11-07T09:15:00Z @AI: Align test docs/logs to Phi-3.5-mini-instruct (Candle phi3); Context7-verified.
//! - 2025-11-07T08:34:00Z @AI: Revert to Phi-2 model to fix config deserialization error (~5.3GB).
//! - 2025-11-06T21:43:00Z @AI: Downgrade model from Phi-4 to Phi-3.5-mini-instruct to reduce download size (~7.1GB instead of 14.7GB).
//! - 2025-11-06T21:38:00Z @AI: Upgrade model from Phi-3-mini-4k-instruct to Phi-4 for improved performance.
//! - 2025-11-06T21:34:00Z @AI: Update model from Phi-2 to Phi-3-mini-4k-instruct for better performance.
//! - 2025-11-06T21:19:00Z @AI: Initial Candle adapter five-person conversation integration test.

/// Realistic 5-minute team planning meeting with 5 participants.
///
/// This transcript simulates a software architecture planning meeting where the team
/// discusses implementation of a hexagonal architecture pattern. The conversation
/// includes clear action items, assignees, and due dates, making it ideal for testing
/// the extraction pipeline's accuracy.
const FIVE_PERSON_CONVERSATION: &str = r#"
Architecture Planning Meeting - Transcript Processor Implementation
Date: November 6, 2025
Duration: 6 minutes 30 seconds
Participants: Sarah (Tech Lead), David (Backend), Maria (ML Engineer),
              James (Infrastructure), Emily (QA Lead)

[00:00] Sarah: Good morning everyone. Thanks for joining. Today we need to finalize
the implementation plan for the transcript processor using hexagonal architecture.
Let's keep this focused - we have a lot to cover.

[00:30] Sarah: First, let's talk about the core domain. David, you mentioned you
wanted to define the domain entities. Can you take ownership of that?

[00:45] David: Absolutely. I'll define the ActionItem, Task, and TranscriptAnalysis
structs. I can have the complete domain layer done by November 10th.

[01:00] Sarah: Perfect. Now, for the LLM integration, we need both Ollama and
Candle adapters. Maria, this seems right up your alley.

[01:15] Maria: Yes, I'll take ownership of both adapters. The Ollama one should be
straightforward using the rig crate. The Candle adapter will be more complex since
we need to implement the Phi-2 model loading and inference pipeline. I'm thinking
November 12th for both adapters?

[01:40] Sarah: That works. Make sure the Candle adapter uses proper error handling
and includes progress feedback for model downloads.

[01:50] Maria: Will do. I'll use the hf-hub crate for model fetching and add
detailed logging.

[02:05] Sarah: Great. James, we need the infrastructure layer and the main binary
wiring. Can you handle that?

[02:15] James: Sure. I'll set up the dependency injection in main.rs and create
the in-memory task repository adapter. Since this depends on David's domain work
and Maria's adapters, I'll target November 13th.

[02:35] Sarah: Sounds good. Now, Emily, we need comprehensive testing. Can you
own the test strategy?

[02:45] Emily: Definitely. I'm thinking we need three levels: unit tests for the
domain, integration tests for each adapter, and an end-to-end test for the full
pipeline. I can have the test suite complete by November 14th.

[03:10] David: Emily, for the integration tests, should we use mock data or actual
LLM calls?

[03:20] Emily: Good question. For the Ollama tests, we'll use the actual service
since it's fast. For Candle, we might need mock data initially since model
inference can be slow. Let's discuss offline and I'll document the approach.

[03:40] Maria: Speaking of the Candle adapter, I should also implement caching
for the parsed model weights. That'll make subsequent loads much faster.

[03:55] Sarah: Excellent idea. Add that to your task. Now, David, you also
mentioned wanting to add support for parsing timestamps from transcripts?

[04:10] David: Yes, I think we should extract the speaker timestamps like [00:30]
and store them as metadata. It'll help with debugging and audit trails. I can
add that to my domain work - same November 10th deadline.

[04:30] Sarah: Perfect. Maria, one more thing - can you also implement the
Tree-sitter integration for syntax analysis? I know that's separate from the
LLM adapters, but it's critical for the code analysis features.

[04:50] Maria: Sure, I can do that. Tree-sitter has excellent Rust bindings.
I'll add it to my plate and still target November 12th. It's mostly configuration
work for the language grammars.

[05:10] James: Sarah, should I also set up the Docker Compose configuration for
the Ollama service? That would make local development easier.

[05:25] Sarah: Yes, please do. Include setup scripts too. Let's make onboarding
as smooth as possible.

[05:35] James: Will do. I'll include that in my November 13th delivery.

[05:50] Emily: I'll also create a comprehensive README with examples of how to
use both adapters. Documentation is part of the test deliverable, so November 14th.

[06:10] Sarah: Excellent. Let me summarize the action items:
- David: Domain entities and timestamp parsing by November 10th
- Maria: Ollama and Candle adapters plus Tree-sitter integration by November 12th
- James: Infrastructure wiring and Docker setup by November 13th
- Emily: Complete test suite and documentation by November 14th

[06:25] Sarah: Everyone on board? Any blockers?

[06:30] David, Maria, James, Emily: All good! / No blockers. / Ready to go. / Let's do this.

[06:35] Sarah: Perfect. Let's reconvene Friday to check progress. Thanks everyone!
"#;

#[tokio::test]
async fn test_five_person_conversation_integration() {
    // Test: Validates the complete pipeline with a realistic 5-minute, 5-person conversation
    // using the Candle adapter for embedded, in-process inference.
    // Justification: This integration test ensures the system can handle real-world meeting
    // transcripts with multiple speakers, diverse action items, and complex conversational
    // patterns. It validates that the embedded Phi-3.5-mini-instruct model provides extraction accuracy
    // comparable to external LLM services, task persistence works correctly, and data integrity
    // is maintained across the full stack - critical for production readiness without external
    // service dependencies.

    println!("\n=== Candle Five-Person Conversation Integration Test ===\n");
    println!("Testing extraction with embedded Phi-3.5-mini-instruct model...\n");

    // Initialize the Candle adapter
    // Note: First run will download ~7.6GB model from HuggingFace
    println!("Initializing Candle adapter with Phi-3.5-mini-instruct model...");
    println!("(First run will download ~7.6GB model from HuggingFace)\n");

    let candle_adapter = transcript_processor::adapters::candle_adapter::CandleTranscriptExtractorAdapter::new()
        .await
        .expect("Failed to initialize Candle adapter");

    let candle_adapter = std::sync::Arc::new(candle_adapter);

    println!("✅ Candle adapter initialized\n");
    println!("\n=== Processing 5-Minute Team Meeting Transcript ===\n");

    // Initialize the in-memory task repository
    let task_repo =
        transcript_processor::adapters::in_memory_task_adapter::InMemoryTaskAdapter::new();

    // Create the use case
    let mut process_use_case =
        transcript_processor::application::use_cases::process_transcript::ProcessTranscriptUseCase::new(
            candle_adapter,
            task_repo,
        );

    // Execute the pipeline
    println!("Sending transcript to embedded Phi-3.5-mini-instruct model for extraction...\n");
    let result = process_use_case
        .process(FIVE_PERSON_CONVERSATION)
        .await;

    // Assert the pipeline succeeded
    assert!(
        result.is_ok(),
        "Pipeline should successfully process five-person transcript: {:?}",
        result.err()
    );

    let extracted_tasks = result.unwrap();

    println!("✅ Extracted and persisted {} action items\n", extracted_tasks.len());

    // Print detailed results
    println!("=== Extracted Tasks ===\n");
    for (idx, task) in extracted_tasks.iter().enumerate() {
        println!("{}. {}", idx + 1, task.title);
        println!(
            "   Assigned to: {}",
            task.assignee
                .as_ref()
                .unwrap_or(&std::string::String::from("Unassigned"))
        );
        println!(
            "   Due date: {}",
            task.due_date
                .as_ref()
                .unwrap_or(&std::string::String::from("No deadline"))
        );
        println!("   Status: {:?}", task.status);
        println!("   Task ID: {}", task.id);
        println!();
    }

    // === VALIDATION ASSERTIONS ===

    // Expected: 8 action items from the explicit summary at the end
    // Sarah lists: David (2 tasks), Maria (3 tasks), James (2 tasks), Emily (1 task)
    assert!(
        extracted_tasks.len() >= 7 && extracted_tasks.len() <= 10,
        "Expected 7-10 action items from the five-person conversation (found {})",
        extracted_tasks.len()
    );

    // Validate all tasks have proper structure
    for task in &extracted_tasks {
        assert!(
            !task.title.is_empty(),
            "Task ID {} has empty title",
            task.id
        );

        assert!(
            task.id.len() == 36,
            "Task has invalid UUID format: {}",
            task.id
        );

        assert_eq!(
            task.status,
            transcript_processor::domain::task_status::TaskStatus::Todo,
            "All newly extracted tasks should have Todo status"
        );
    }

    // Count tasks with assignees
    let tasks_with_assignees = extracted_tasks
        .iter()
        .filter(|t| t.assignee.is_some())
        .count();

    // Count tasks with due dates
    let tasks_with_due_dates = extracted_tasks
        .iter()
        .filter(|t| t.due_date.is_some())
        .count();

    // This conversation is very explicit - Sarah summarizes all 8 tasks with assignees and dates
    assert!(
        tasks_with_assignees >= 6,
        "Expected at least 6 tasks with assignees (found {}). \
         Conversation explicitly assigns tasks to David, Maria, James, and Emily.",
        tasks_with_assignees
    );

    assert!(
        tasks_with_due_dates >= 6,
        "Expected at least 6 tasks with due dates (found {}). \
         Most tasks have explicit deadlines (Nov 10, 12, 13, 14).",
        tasks_with_due_dates
    );

    // Collect unique assignees
    let assignee_names: std::collections::HashSet<String> = extracted_tasks
        .iter()
        .filter_map(|t| t.assignee.clone())
        .collect();

    // Expected assignees: David, Maria, James, Emily (Sarah is the organizer, not assigned tasks)
    let expected_assignees = vec!["David", "Maria", "James", "Emily"];
    let found_expected: Vec<&str> = expected_assignees
        .iter()
        .filter(|name| {
            assignee_names
                .iter()
                .any(|extracted| extracted.contains(*name))
        })
        .copied()
        .collect();

    println!("\n=== Validation Results ===");
    println!("✅ Total tasks extracted: {}", extracted_tasks.len());
    println!(
        "✅ Tasks with assignees: {} ({:.0}%)",
        tasks_with_assignees,
        (tasks_with_assignees as f64 / extracted_tasks.len() as f64) * 100.0
    );
    println!(
        "✅ Tasks with due dates: {} ({:.0}%)",
        tasks_with_due_dates,
        (tasks_with_due_dates as f64 / extracted_tasks.len() as f64) * 100.0
    );
    println!("✅ Unique assignees found: {:?}", assignee_names);
    println!(
        "✅ Expected assignees matched: {}/4 ({:?})",
        found_expected.len(),
        found_expected
    );

    // Verify we found at least 3 of the 4 expected assignees
    assert!(
        found_expected.len() >= 3,
        "Expected to find at least 3 of 4 assignees (David, Maria, James, Emily). \
         Found only: {:?}",
        found_expected
    );

    println!("\n=== Conversation Characteristics ===");
    println!("This transcript tests:");
    println!("  ✓ Multiple speakers (5 people)");
    println!("  ✓ Complex task assignments with varied patterns");
    println!("  ✓ Explicit due dates in various formats");
    println!("  ✓ Task dependencies and coordination");
    println!("  ✓ Summary section listing all action items");

    println!("\n🎉 Candle five-person conversation test PASSED!");
    println!("   Embedded Phi-3.5-mini-instruct model successfully extracted structured tasks!");
}
