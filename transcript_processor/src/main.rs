//! Transcript processor main binary - Hexagonal Architecture demonstration.
//!
//! This binary demonstrates the complete transcript processing pipeline by
//! wiring together all layers of the hexagonal architecture:
//! - Domain: Core business entities (ActionItem, TaskStatus, Priority, etc.)
//! - Application: Use cases (ProcessTranscriptUseCase, ManageTaskUseCase)
//! - Adapters: Infrastructure implementations (Ollama LLM, in-memory storage)
//!
//! The main function serves as the composition root, where concrete adapters
//! are injected into use cases through the port interfaces, demonstrating
//! dependency inversion and the hexagonal pattern.
//!
//! Prerequisites:
//! - For Ollama adapter: Ollama must be installed and running (https://ollama.ai) + Run: ollama pull llama3.2
//! - For Candle adapter: First run will download Phi-3.5-mini-instruct model from HuggingFace (~7.6GB)
//!
//! Usage:
//! - Use Ollama adapter (default): cargo run
//! - Use Candle adapter: EXTRACTOR=candle cargo run
//!
//! Revision History
//! - 2025-11-07T09:15:00Z @AI: Align runtime/docs with Phi-3.5-mini-instruct for Candle; Context7-verified phi3 support in candle-transformers 0.9.2-alpha.1.
//! - 2025-11-07T08:34:00Z @AI: Revert Candle adapter to Phi-2 to fix config deserialization error (~5.3GB).
//! - 2025-11-06T21:43:00Z @AI: Downgrade Candle adapter from Phi-4 to Phi-3.5-mini-instruct to reduce download size (~7.1GB instead of 14.7GB).
//! - 2025-11-06T21:38:00Z @AI: Upgrade Candle adapter from Phi-3-mini-4k-instruct to Phi-4 for improved performance.
//! - 2025-11-06T21:34:00Z @AI: Update Candle adapter to use Phi-3-mini-4k-instruct instead of Phi-2 for better performance.
//! - 2025-11-06T21:00:00Z @AI: Add support for switching between Ollama and Candle adapters via env var.
//! - 2025-11-06T18:56:00Z @AI: Update adapter name to OllamaTranscriptExtractorAdapter for clarity.
//! - 2025-11-06T18:30:00Z @AI: Update to use HEXSER generic patterns with concrete repository types.
//! - 2025-11-06T18:00:00Z @AI: Initial main binary demonstrating hexagonal architecture.

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Transcript Processor - Hexagonal Architecture Demo ===\n");

    // ============================================================================
    // STEP 1: Initialize Adapters (Infrastructure Layer)
    // ============================================================================
    // These are the "driven" adapters - concrete implementations of the ports
    // defined in the application layer. They handle external infrastructure
    // concerns (LLM API calls, data storage).

    println!("Initializing adapters...");

    // Determine which extractor to use based on EXTRACTOR environment variable
    // Default to "ollama" if not set
    let extractor_type = std::env::var("EXTRACTOR").unwrap_or_else(|_| std::string::String::from("ollama"));

    println!("Using extractor: {}", extractor_type);

    // Create the appropriate transcript extractor adapter based on environment variable
    // Both adapters implement TranscriptExtractorPort, demonstrating the port/adapter pattern
    let transcript_extractor: std::sync::Arc<dyn transcript_processor::application::ports::transcript_extractor_port::TranscriptExtractorPort> = match extractor_type.as_str() {
        "candle" => {
            println!("Initializing Candle adapter with Phi-3.5-mini-instruct model...");
            println!("(First run will download ~7.6GB model from HuggingFace)");
            let candle_adapter = transcript_processor::adapters::candle_adapter::CandleTranscriptExtractorAdapter::new()
                .await
                .map_err(|e| std::format!("Failed to initialize Candle adapter: {}", e))?;
            std::sync::Arc::new(candle_adapter)
        },
        "ollama" | _ => {
            println!("Initializing Ollama adapter with llama3.2 model...");
            let ollama_adapter = transcript_processor::adapters::ollama_adapter::OllamaTranscriptExtractorAdapter::new(
                std::string::String::from("llama3.2"),
            );
            std::sync::Arc::new(ollama_adapter)
        }
    };

    // Create the in-memory task repository adapter
    // This implements TaskRepositoryPort using HEXSER patterns
    // Each use case owns its repository (HEXSER generic pattern)
    let task_repo_for_processing =
        transcript_processor::adapters::in_memory_task_adapter::InMemoryTaskAdapter::new();

    println!("✓ Adapters initialized\n");

    // ============================================================================
    // STEP 2: Initialize Use Cases (Application Layer)
    // ============================================================================
    // Use cases orchestrate the business logic using generic concrete types
    // rather than trait objects, following HEXSER patterns.

    println!("Initializing use cases...");

    // Create the transcript processing use case
    // Pass concrete repository by value (owned by use case)
    // Must be mutable because process() requires &mut self for save() operations
    let mut process_transcript_use_case =
        transcript_processor::application::use_cases::process_transcript::ProcessTranscriptUseCase::new(
            transcript_extractor.clone(),
            task_repo_for_processing,
        );

    println!("✓ Use cases initialized\n");

    // ============================================================================
    // STEP 3: Execute the Pipeline - Process Transcript
    // ============================================================================
    // This is where the application's primary workflow executes.

    println!("=== Processing Transcript ===\n");

    // Example transcript from a project meeting
    let transcript = r#"
Team Meeting - November 6, 2025

Action Items:
1. John will complete the API documentation by November 15th. This is high priority.
2. Sarah needs to review the security audit findings. Due date is November 10th. High priority.
3. Mike should update the deployment scripts. Medium priority, no specific deadline.
4. Emily will organize the team retrospective meeting. Low priority.
5. The database migration script needs to be tested by Alex before November 12th. High priority.
"#;

    println!("Input transcript:\n{}\n", transcript);
    println!("Sending to LLM for extraction...\n");

    // Execute the processing pipeline
    // This calls the LLM, parses the response, and persists tasks using HEXSER's save()
    // Note: process() is now synchronous but still calls async extractor internally
    let extracted_tasks = process_transcript_use_case
        .process(transcript)
        .await
        .map_err(|e| std::format!("Failed to process transcript: {}", e))?;

    println!("✓ Extracted and persisted {} action items\n", extracted_tasks.len());

    // ============================================================================
    // STEP 4: Display Extracted and Persisted Tasks
    // ============================================================================

    println!("=== Extracted and Persisted Tasks ===\n");
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
        println!("   Created: {}", task.created_at);
        println!();
    }

    // ============================================================================
    // STEP 5: Architecture Summary
    // ============================================================================

    println!("=== HEXSER Architecture Summary ===\n");
    println!("This application demonstrates Hexagonal Architecture using the HEXSER framework:");
    println!("  • Domain Layer: Pure business logic with HexEntity derives (Task, ActionItem)");
    println!("  • Application Layer: Generic use cases with compile-time polymorphism");
    println!("  • Adapters Layer: HEXSER Repository/QueryRepository implementations");
    println!();
    println!("HEXSER Pattern Benefits:");
    println!("  ✓ Type Safety: Generic concrete types instead of trait objects");
    println!("  ✓ Performance: Compile-time dispatch, no runtime Arc/Mutex overhead for mutations");
    println!("  ✓ Explicitness: save() and find() methods make operations clear");
    println!("  ✓ Testability: Easy to mock with concrete test types");
    println!();
    println!("Successfully extracted, persisted, and displayed {} tasks!", extracted_tasks.len());

    std::result::Result::Ok(())
}
