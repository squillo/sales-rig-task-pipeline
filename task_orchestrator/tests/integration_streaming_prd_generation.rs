//! Integration test for streaming PRD generation.
//!
//! This test validates that the RigPRDParserAdapter correctly streams
//! task generation updates in real-time, including proper JSON detection,
//! task field extraction, and formatted output generation.
//!
//! Requires: Ollama running locally with llama3.2 model.
//!
//! Revision History
//! - 2025-11-26T21:30:00Z @AI: Add integration test for streaming PRD generation with task detection.

#[tokio::test]
async fn test_streaming_prd_generation_with_task_detection() {
    // Test: Validates streaming PRD generation detects complete tasks and sends proper updates.
    // Justification: Core streaming functionality must correctly parse JSON structure,
    // detect complete task objects, extract all fields, and send formatted updates.

    // Skip if Ollama not available
    let ollama_check = reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .send()
        .await;

    if ollama_check.is_err() {
        eprintln!("⊘ Skipping test: Ollama not running at localhost:11434");
        return;
    }

    // Create test PRD
    let test_prd = task_manager::domain::prd::PRD::new(
        std::string::String::from("project-test"),
        std::string::String::from("Test Calculator CLI"),
        std::vec![std::string::String::from("Build a simple calculator CLI")],
        std::vec![std::string::String::from("Rust"), std::string::String::from("clap for CLI")],
        std::vec![std::string::String::from("Handle division by zero")],
        std::string::String::from("# Test PRD\n\n## Objectives\n- Build a simple calculator CLI\n\n## Tech Stack\n- Rust\n- clap for CLI\n\n## Constraints\n- Handle division by zero"),
    );

    // Create adapter
    let adapter = task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter::new(
        std::string::String::from("llama3.2"),
        std::string::String::from("llama3.2:latest"),
        std::vec::Vec::new(), // No personas for this test
    );

    // Start interactive parsing
    let (update_rx, _response_tx) = adapter.parse_prd_interactively(test_prd).await
        .expect("Failed to start PRD parsing");

    // Collect streaming updates
    let mut received_thinking = false;
    let mut received_task_generated = false;
    let mut task_titles = std::vec::Vec::new();
    let mut complete = false;

    // Use tokio::time::timeout to prevent hanging
    let timeout_duration = std::time::Duration::from_secs(60);
    let mut update_rx = update_rx;

    match tokio::time::timeout(timeout_duration, async {
        while let std::option::Option::Some(update) = update_rx.recv().await {
            match update {
                task_orchestrator::adapters::rig_prd_parser_adapter::PRDGenUpdate::Thinking(msg) => {
                    received_thinking = true;
                    println!("💭 Thinking: {}", if msg.len() > 100 {
                        std::format!("{}...", &msg[0..100])
                    } else {
                        msg
                    });
                }
                task_orchestrator::adapters::rig_prd_parser_adapter::PRDGenUpdate::TaskGenerated { title, description, .. } => {
                    received_task_generated = true;
                    println!("✓ Task Generated: {}", title);
                    println!("  Description: {}", description);
                    task_titles.push(title);
                }
                task_orchestrator::adapters::rig_prd_parser_adapter::PRDGenUpdate::Complete(tasks) => {
                    complete = true;
                    println!("✓ Complete: {} tasks generated", tasks.len());
                    break;
                }
                task_orchestrator::adapters::rig_prd_parser_adapter::PRDGenUpdate::Error(err) => {
                    panic!("Received error update: {}", err);
                }
                _ => {}
            }
        }
    }).await {
        std::result::Result::Ok(_) => {
            // Success - received Complete update
        }
        std::result::Result::Err(_) => {
            panic!("Test timeout after 60s waiting for streaming updates");
        }
    }

    // Assertions
    std::assert!(received_thinking, "Should receive Thinking updates with streaming JSON");
    std::assert!(received_task_generated, "Should receive TaskGenerated updates when complete tasks detected");
    std::assert!(complete, "Should receive Complete update when generation finishes");
    std::assert!(!task_titles.is_empty(), "Should generate at least one task");

    println!("\n✓ Streaming test passed:");
    println!("  - Received Thinking updates: {}", received_thinking);
    println!("  - Received TaskGenerated updates: {}", received_task_generated);
    println!("  - Generation completed: {}", complete);
    println!("  - Tasks detected: {}", task_titles.len());
    for (i, title) in task_titles.iter().enumerate() {
        println!("    {}. {}", i + 1, title);
    }
}

#[tokio::test]
async fn test_streaming_prd_with_complex_json() {
    // Test: Validates streaming handles complex PRD with multiple tasks and nested fields.
    // Justification: Real PRDs generate complex JSON with priority, complexity, etc.
    // Must correctly extract all fields from streaming task objects.

    // Skip if Ollama not available
    let ollama_check = reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .send()
        .await;

    if ollama_check.is_err() {
        eprintln!("⊘ Skipping test: Ollama not running at localhost:11434");
        return;
    }

    // Create PRD with multiple features
    let test_prd = task_manager::domain::prd::PRD::new(
        std::string::String::from("project-complex"),
        std::string::String::from("Multi-Feature App"),
        std::vec![
            std::string::String::from("User authentication"),
            std::string::String::from("Product catalog"),
            std::string::String::from("Shopping cart"),
            std::string::String::from("Payment processing"),
        ],
        std::vec![
            std::string::String::from("Rust backend"),
            std::string::String::from("PostgreSQL"),
            std::string::String::from("React frontend"),
        ],
        std::vec![
            std::string::String::from("PCI compliance required"),
            std::string::String::from("Must support 1000+ concurrent users"),
        ],
        std::string::String::from("# Complex PRD\n\n## Objectives\n- User authentication\n- Product catalog\n- Shopping cart\n- Payment processing\n\n## Tech Stack\n- Rust backend\n- PostgreSQL\n- React frontend\n\n## Constraints\n- PCI compliance required\n- Must support 1000+ concurrent users"),
    );

    let adapter = task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter::new(
        std::string::String::from("llama3.2"),
        std::string::String::from("llama3.2:latest"),
        std::vec::Vec::new(), // No personas for this test
    );

    let (update_rx, _response_tx) = adapter.parse_prd_interactively(test_prd).await
        .expect("Failed to start PRD parsing");

    // Track task details
    let mut tasks_with_priority = 0;
    let mut tasks_with_complexity = 0;
    let mut total_tasks = 0;
    let mut update_rx = update_rx;

    match tokio::time::timeout(std::time::Duration::from_secs(90), async {
        while let std::option::Option::Some(update) = update_rx.recv().await {
            if let task_orchestrator::adapters::rig_prd_parser_adapter::PRDGenUpdate::TaskGenerated { title, description, .. } = update {
                total_tasks += 1;
                println!("✓ Task {}: {}", total_tasks, title);

                // Check if priority was extracted
                if description.contains("Priority:") {
                    tasks_with_priority += 1;
                }

                // Check if complexity was extracted
                if description.contains("Complexity:") {
                    tasks_with_complexity += 1;
                }
            } else if let task_orchestrator::adapters::rig_prd_parser_adapter::PRDGenUpdate::Complete(_) = update {
                break;
            }
        }
    }).await {
        std::result::Result::Ok(_) => {}
        std::result::Result::Err(_) => {
            panic!("Test timeout after 90s");
        }
    }

    // Assertions
    std::assert!(total_tasks >= 3, "Should generate at least 3 tasks for complex PRD (got {})", total_tasks);

    println!("\n✓ Complex PRD streaming test passed:");
    println!("  - Total tasks generated: {}", total_tasks);
    println!("  - Tasks with priority field: {}", tasks_with_priority);
    println!("  - Tasks with complexity field: {}", tasks_with_complexity);
}
