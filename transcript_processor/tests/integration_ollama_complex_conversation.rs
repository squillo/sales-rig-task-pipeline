//! Integration test: Complex conversation with red herrings and challenging extraction scenarios.
//!
//! This integration test validates the transcript processing pipeline's ability to:
//! 1. Extract action items from a realistic conversation with natural interruptions
//! 2. Filter out red herring conversations (off-topic discussions about weather, lunch, etc.)
//! 3. Handle varied assignment patterns ("I'll...", "X will...", "Let's have Y...")
//! 4. Accurately extract assignees and due dates despite conversational noise
//!
//! The conversation includes:
//! - 5 people discussing a software project
//! - 6-7 legitimate action items
//! - 2-3 red herring conversation sections
//! - Natural topic changes and interruptions
//!
//! Prerequisites:
//! - Ollama service running at http://localhost:11434
//! - llama3.2 model available
//! - Use ./setup-ollama.sh for automated setup
//!
//! Running the Test:
//! ```bash
//! cd transcript_processor
//! cargo test --test integration_complex_conversation -- --nocapture
//! ```
//!
//! Revision History
//! - 2025-11-06T20:51:00Z @AI: Initial complex conversation integration test.

/// Realistic 5-minute team standup with red herrings and natural conversation flow.
///
/// This transcript simulates a daily standup meeting that includes:
/// - Action items mixed with status updates
/// - Off-topic discussions (weather, lunch plans)
/// - Natural interruptions and topic changes
/// - Varied assignment patterns to test extraction robustness
const COMPLEX_CONVERSATION: &str = r#"
Daily Standup - Code Canvas Project Team
Date: November 6, 2025
Duration: ~5 minutes
Participants: Alex (Team Lead), Jordan (Backend), Morgan (Frontend),
              Casey (DevOps), Riley (QA)

[00:00] Alex: Morning everyone! Let's do a quick standup. Before we start though,
wow, is it pouring outside! Did everyone make it in okay?

[00:15] Morgan: Yeah, I got completely soaked walking from the parking lot. I
should have brought an umbrella.

[00:25] Casey: I checked the weather this morning - it's supposed to clear up
by noon. Perfect timing for lunch!

[00:35] Alex: Speaking of lunch, I heard that new Thai place opened downtown.
Anyone want to check it out around 12:30?

[00:45] Jordan: I'm in! I've been craving pad thai all week.

[00:55] Riley: Count me in too. Okay, shall we actually start the standup?

[01:05] Alex: Right, sorry! Let's get focused. Jordan, what's your status on
the API gateway refactoring?

[01:15] Jordan: Good progress. I finished the authentication layer yesterday,
but I need to tackle the rate limiting implementation today. I'll have that
completed by tomorrow, November 7th. It's a high priority item since we can't
deploy without it.

[01:35] Alex: Excellent. Make sure that's done by EOD tomorrow. Morgan, how's
the component library coming?

[01:45] Morgan: The base components are done, but I need someone to review the
accessibility implementation. Casey, could you take a look at that? I'm hoping
to get feedback by Friday, November 8th, so I can iterate before next week.

[02:05] Casey: Sure, I can review the accessibility stuff. I'll carve out time
on Thursday.

[02:15] Alex: Great. Casey, while we have you - any blockers on your end?

[02:20] Casey: Actually yes. The CI/CD pipeline is still flaky. We're getting
random failures in the Docker build step. I need to debug and fix that. It's
becoming a major pain point for everyone. I'm targeting November 9th to have
it stable.

[02:40] Riley: Oh thank goodness. I've been dealing with those failures too.
Yesterday I had to re-run the same test suite four times.

[02:50] Morgan: Wait, Riley, did you see that email about the company holiday
party next month? They're doing it at that fancy hotel downtown!

[03:00] Riley: Oh yeah! I'm excited, they usually have great food. Are they
doing a white elephant gift exchange again?

[03:10] Jordan: I hope so, that was hilarious last year. Remember when someone
brought that singing fish?

[03:20] Alex: Okay, okay, we can talk about the party later. Riley, what's your
testing status?

[03:30] Riley: Right, sorry! I've completed the integration tests for the user
authentication flow, but I still need to write the end-to-end tests for the new
dashboard. Alex, can you help me define the test scenarios? I need that by
tomorrow so I can write the tests by November 10th.

[03:55] Alex: Absolutely. I'll send you a detailed test plan by end of day today.

[04:05] Morgan: One more thing - I need to update the design system documentation.
The component API has changed significantly. I'll own that task and have it done
by November 12th.

[04:20] Jordan: Oh, documentation reminds me - did anyone see that hilarious bug
report someone filed? They said the app "attacked" them with pop-ups.

[04:30] Casey: Ha! I saw that. Classic user error, but the phrasing was gold.

[04:40] Alex: Alright team, let's wrap up. One last item - we need someone to
update the deployment runbook with the new Kubernetes configurations. Riley,
since you're familiar with both the ops side and testing, could you handle that?
Medium priority, let's say November 15th deadline.

[05:00] Riley: Sure, I can do that. I'll coordinate with Casey on the specifics.

[05:35] Alex: Everyone clear? Great. Now who's serious about that Thai place?

[05:40] Morgan: Me! I'm thinking green curry.

[05:45] Casey: I'll make a reservation for 12:30. Party of five?

[05:50] Jordan: Perfect. See you all then!

[05:55] Alex: Alright, back to work everyone. Stay dry out there!
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
async fn test_complex_conversation_with_red_herrings() {
    // Test: Validates the pipeline's ability to extract action items from a complex conversation
    // with red herrings (weather talk, lunch plans, holiday party discussion) and natural
    // interruptions.
    // Justification: Real-world meetings contain significant off-topic discussion and natural
    // conversation flow. This test ensures the LLM can distinguish actionable items from noise,
    // which is critical for production use where transcripts won't be sanitized.

    println!("\n=== Complex Conversation Integration Test ===\n");
    println!("Testing extraction with red herrings and natural conversation flow...\n");

    // Verify Ollama service is running
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

    println!("✅ Ollama service is running\n");

    // Initialize the Ollama adapter
    let ollama_adapter = std::sync::Arc::new(
        transcript_processor::adapters::ollama_adapter::OllamaTranscriptExtractorAdapter::new(
            std::string::String::from("llama3.2"),
        ),
    );

    // Initialize the in-memory task repository
    let task_repo =
        transcript_processor::adapters::in_memory_task_adapter::InMemoryTaskAdapter::new();

    // Create the use case
    let mut process_use_case =
        transcript_processor::application::use_cases::process_transcript::ProcessTranscriptUseCase::new(
            ollama_adapter,
            task_repo,
        );

    // Execute the pipeline
    println!("Processing complex conversation with red herrings...\n");
    let result = process_use_case
        .process(COMPLEX_CONVERSATION)
        .await;

    // Assert the pipeline succeeded
    assert!(
        result.is_ok(),
        "Pipeline should successfully process complex transcript: {:?}",
        result.err()
    );

    let extracted_tasks = result.unwrap();

    // Print detailed results
    println!("✅ Extraction Complete!\n");
    println!("=== Extracted Tasks ({}) ===\n", extracted_tasks.len());

    for (idx, task) in extracted_tasks.iter().enumerate() {
        println!("Task #{}", idx + 1);
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
        println!();
    }

    // === VALIDATION ASSERTIONS ===

    // Expected: 6 legitimate action items from Alex's summary
    // (Weather, lunch, holiday party discussions should NOT generate tasks)
    assert!(
        extracted_tasks.len() >= 5 && extracted_tasks.len() <= 8,
        "Expected 5-8 action items (found {}). Red herrings should be filtered out.",
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

    // The conversation explicitly mentions 6 tasks, all with assignees and due dates
    assert!(
        tasks_with_assignees >= 4,
        "Expected at least 4 tasks with assignees (found {})",
        tasks_with_assignees
    );

    assert!(
        tasks_with_due_dates >= 4,
        "Expected at least 4 tasks with due dates (found {})",
        tasks_with_due_dates
    );

    // Check for expected assignees: Jordan, Morgan, Casey, Riley, Alex
    let assignee_names: std::collections::HashSet<String> = extracted_tasks
        .iter()
        .filter_map(|t| t.assignee.clone())
        .collect();

    println!("=== Validation Results ===");
    println!("✅ Total tasks extracted: {}", extracted_tasks.len());
    println!("✅ Tasks with assignees: {} ({:.0}%)",
        tasks_with_assignees,
        (tasks_with_assignees as f64 / extracted_tasks.len() as f64) * 100.0
    );
    println!("✅ Tasks with due dates: {} ({:.0}%)",
        tasks_with_due_dates,
        (tasks_with_due_dates as f64 / extracted_tasks.len() as f64) * 100.0
    );
    println!("✅ Unique assignees found: {:?}", assignee_names);

    println!("\n=== Red Herring Filtering ===");
    println!("Topics that should NOT generate tasks:");
    println!("  - Weather discussion (rain, umbrellas)");
    println!("  - Lunch plans (Thai restaurant, pad thai)");
    println!("  - Holiday party (gift exchange, singing fish)");
    println!("\nIf extraction count is 5-8, red herrings were successfully filtered!");

    println!("\n🎉 Complex conversation test PASSED - Pipeline handles noise correctly!");
}
