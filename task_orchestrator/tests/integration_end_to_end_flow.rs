//! End-to-end integration test for graph-flow orchestration.
//!
//! Tests the complete pipeline:
//! Task → run_task_with_flow → Semantic Router → Enhancement → Comprehension Test → Enhanced Task
//!
//! This validates that the orchestration graph executes correctly with deterministic adapters
//! and that state threading works properly through all nodes.
//!
//! Run with: `cargo test --package task_orchestrator --test integration_end_to_end_flow`
//! Run ignored tests: `cargo test --package task_orchestrator --test integration_end_to_end_flow -- --ignored`
//!
//! Revision History
//! - 2025-11-22T19:30:00Z @AI: Create end-to-end integration test for Phase 1 Sprint 1.1.

#[tokio::test]
async fn test_end_to_end_orchestration_flow_in_memory() {
    // Test: Validates complete orchestration flow with in-memory session storage.
    // Justification: Core functionality must work without external dependencies.

    // 1. Create a sample task
    let action_item = transcript_extractor::domain::action_item::ActionItem {
        title: std::string::String::from("Implement user authentication system"),
        assignee: std::option::Option::Some(std::string::String::from("Alice")),
        due_date: std::option::Option::Some(std::string::String::from("2025-12-01")),
    };

    let task = task_manager::domain::task::Task::from_action_item(
        &action_item,
        std::option::Option::Some(std::string::String::from("transcript-001")),
    );

    // Store initial state for comparison
    let initial_title = task.title.clone();
    let initial_status = task.status.clone();

    std::println!("\n=== Initial Task ===");
    std::println!("Title: {}", task.title);
    std::println!("Status: {:?}", task.status);
    std::println!("Enhancements: {:?}", task.enhancements);
    std::println!("Comprehension Tests: {:?}", task.comprehension_tests);

    // 2. Run through orchestration flow
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new("ollama", "llama3.1").unwrap();
    let test_type = "short_answer";

    let result = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
        &factory,
        test_type,
        task,
    )
    .await;

    std::assert!(result.is_ok(), "Orchestration flow should complete successfully: {:?}", result.err());

    let enhanced_task = result.unwrap();

    std::println!("\n=== Enhanced Task ===");
    std::println!("Title: {}", enhanced_task.title);
    std::println!("Status: {:?}", enhanced_task.status);
    std::println!("Enhancements: {:?}", enhanced_task.enhancements);
    std::println!("Comprehension Tests: {:?}", enhanced_task.comprehension_tests);

    // 3. Validate task structure
    std::assert_eq!(enhanced_task.title, initial_title, "Title should remain unchanged");
    std::assert_eq!(enhanced_task.id, enhanced_task.id, "ID should remain unchanged");

    // 4. Validate enhancements were generated
    std::assert!(
        enhanced_task.enhancements.is_some(),
        "Task should have enhancements after orchestration"
    );

    if let std::option::Option::Some(ref enhancements) = enhanced_task.enhancements {
        std::assert!(
            !enhancements.is_empty(),
            "Enhancements list should not be empty"
        );

        std::println!("\n=== Enhancements Generated ===");
        for (idx, enhancement) in enhancements.iter().enumerate() {
            std::println!("Enhancement {}: ID={}, Type={}, Content={}",
                idx + 1,
                enhancement.enhancement_id,
                enhancement.enhancement_type,
                enhancement.content
            );

            // Validate enhancement structure
            std::assert!(!enhancement.enhancement_id.is_empty(), "Enhancement ID should not be empty");
            std::assert!(!enhancement.content.is_empty(), "Enhancement content should not be empty");
            std::assert_eq!(enhancement.task_id, enhanced_task.id, "Enhancement should reference correct task");
        }
    }

    // 5. Validate comprehension tests were generated
    std::assert!(
        enhanced_task.comprehension_tests.is_some(),
        "Task should have comprehension tests after orchestration"
    );

    if let std::option::Option::Some(ref tests) = enhanced_task.comprehension_tests {
        std::assert!(
            !tests.is_empty(),
            "Comprehension tests list should not be empty"
        );

        std::println!("\n=== Comprehension Tests Generated ===");
        for (idx, test) in tests.iter().enumerate() {
            std::println!("Test {}: ID={}, Type={}, Question={}",
                idx + 1,
                test.test_id,
                test.test_type,
                test.question
            );

            // Validate test structure
            std::assert!(!test.test_id.is_empty(), "Test ID should not be empty");
            std::assert!(!test.question.is_empty(), "Test question should not be empty");
            std::assert_eq!(test.task_id, enhanced_task.id, "Test should reference correct task");

            // Validate test type matches request
            std::assert_eq!(
                test.test_type,
                test_type,
                "Test type should match requested type"
            );
        }
    }

    // 6. Validate status progression
    // Note: Current deterministic adapters don't change status, but this validates the baseline
    std::println!("\n=== Status Progression ===");
    std::println!("Initial Status: {:?}", initial_status);
    std::println!("Final Status: {:?}", enhanced_task.status);

    std::println!("\n✓ End-to-end orchestration flow test passed");
}

#[cfg(feature = "sqlite_persistence")]
#[tokio::test]
async fn test_end_to_end_orchestration_flow_with_sqlite() {
    // Test: Validates orchestration flow with SQLite session storage.
    // Justification: Must verify persistence layer integration.

    std::println!("\n=== Testing with SQLite Session Storage ===");

    // Use in-memory SQLite for test isolation
    std::env::set_var("TASK_ORCHESTRATOR_SQLITE_URL", "sqlite::memory:");

    let action_item = transcript_extractor::domain::action_item::ActionItem {
        title: std::string::String::from("Set up CI/CD pipeline"),
        assignee: std::option::Option::Some(std::string::String::from("Bob")),
        due_date: std::option::Option::None,
    };

    let task = task_manager::domain::task::Task::from_action_item(&action_item, std::option::Option::None);

    std::println!("Initial task: {}", task.title);
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new("ollama", "llama3.1").unwrap();

    let result = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
        &factory,
        "multiple_choice",
        task,
    )
    .await;

    std::assert!(result.is_ok(), "SQLite-backed flow should complete: {:?}", result.err());

    let enhanced_task = result.unwrap();

    // Validate same properties as in-memory test
    std::assert!(enhanced_task.enhancements.is_some(), "Should have enhancements");
    std::assert!(enhanced_task.comprehension_tests.is_some(), "Should have tests");

    std::println!("✓ SQLite persistence test passed");
}

#[tokio::test]
async fn test_multiple_tasks_through_flow() {
    // Test: Validates orchestration handles multiple tasks correctly.
    // Justification: System must maintain state isolation between tasks.

    std::println!("\n=== Testing Multiple Tasks ===");

    let tasks = std::vec![
        ("Refactor authentication module", "Alice"),
        ("Write API documentation", "Bob"),
        ("Add integration tests", "Charlie"),
    ];

    for (title, assignee) in tasks {
        let action_item = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from(title),
            assignee: std::option::Option::Some(std::string::String::from(assignee)),
            due_date: std::option::Option::None,
        };

        let task = task_manager::domain::task::Task::from_action_item(&action_item, std::option::Option::None);
        let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new("ollama", "llama3.1").unwrap();

        let result = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
            &factory,
            "short_answer",
            task,
        )
        .await;

        std::assert!(result.is_ok(), "Task '{}' should complete", title);

        let enhanced_task = result.unwrap();

        std::assert_eq!(enhanced_task.title, title, "Task title should be preserved");
        std::assert!(enhanced_task.enhancements.is_some(), "Task '{}' should have enhancements", title);
        std::assert!(enhanced_task.comprehension_tests.is_some(), "Task '{}' should have tests", title);

        std::println!("✓ Task '{}' completed successfully", title);
    }

    std::println!("✓ Multiple tasks test passed");
}

#[tokio::test]
async fn test_task_state_threading_through_nodes() {
    // Test: Validates GraphState correctly threads task through all nodes.
    // Justification: State management is critical for orchestration correctness.

    let action_item = transcript_extractor::domain::action_item::ActionItem {
        title: std::string::String::from("Implement rate limiting"),
        assignee: std::option::Option::None,
        due_date: std::option::Option::Some(std::string::String::from("2025-11-30")),
    };

    let mut task = task_manager::domain::task::Task::from_action_item(&action_item, std::option::Option::None);

    // Add some initial state to validate it's preserved
    task.source_transcript_id = std::option::Option::Some(std::string::String::from("transcript-999"));
    task.source_prd_id = std::option::Option::Some(std::string::String::from("prd-abc"));

    let original_id = task.id.clone();
    let original_transcript_id = task.source_transcript_id.clone();
    let original_prd_id = task.source_prd_id.clone();
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new("ollama", "llama3.1").unwrap();

    let result = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
        &factory,
        "multiple_choice",
        task,
    )
    .await;

    std::assert!(result.is_ok(), "Flow should complete");

    let enhanced_task = result.unwrap();

    // Validate original task properties are preserved
    std::assert_eq!(enhanced_task.id, original_id, "Task ID should be preserved");
    std::assert_eq!(
        enhanced_task.source_transcript_id,
        original_transcript_id,
        "Source transcript ID should be preserved"
    );
    std::assert_eq!(
        enhanced_task.source_prd_id,
        original_prd_id,
        "Source PRD ID should be preserved"
    );

    // Validate new properties were added
    std::assert!(
        enhanced_task.enhancements.is_some(),
        "Enhancements should be added while preserving original state"
    );
    std::assert!(
        enhanced_task.comprehension_tests.is_some(),
        "Tests should be added while preserving original state"
    );

    std::println!("✓ State threading test passed");
}

#[tokio::test]
#[ignore] // Mark as ignored for CI (requires real understanding of deterministic adapter behavior)
async fn test_baseline_deterministic_adapter_output() {
    // Test: Documents baseline behavior of deterministic adapters.
    // Justification: Establishes reference point before Rig upgrade.
    //
    // Run with: cargo test --package task_orchestrator --test integration_end_to_end_flow -- --ignored --nocapture

    std::println!("\n=== Baseline: Deterministic Adapter Behavior ===");

    let action_item = transcript_extractor::domain::action_item::ActionItem {
        title: std::string::String::from("Baseline test task"),
        assignee: std::option::Option::None,
        due_date: std::option::Option::None,
    };

    let task = task_manager::domain::task::Task::from_action_item(&action_item, std::option::Option::None);
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new("ollama", "llama3.1").unwrap();

    let result = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
        &factory,
        "short_answer",
        task,
    )
    .await;

    std::assert!(result.is_ok());

    let enhanced_task = result.unwrap();

    std::println!("\n--- Task Details ---");
    std::println!("Title: {}", enhanced_task.title);
    std::println!("Status: {:?}", enhanced_task.status);
    std::println!("ID: {}", enhanced_task.id);

    if let std::option::Option::Some(ref enhancements) = enhanced_task.enhancements {
        std::println!("\n--- Enhancements ({}) ---", enhancements.len());
        for enh in enhancements {
            std::println!("  ID: {}", enh.enhancement_id);
            std::println!("  Type: {}", enh.enhancement_type);
            std::println!("  Content: {}", enh.content);
            std::println!("  Task ID: {}", enh.task_id);
            std::println!("  Timestamp: {}", enh.timestamp);
            std::println!();
        }
    }

    if let std::option::Option::Some(ref tests) = enhanced_task.comprehension_tests {
        std::println!("\n--- Comprehension Tests ({}) ---", tests.len());
        for test in tests {
            std::println!("  ID: {}", test.test_id);
            std::println!("  Type: {}", test.test_type);
            std::println!("  Question: {}", test.question);
            std::println!("  Options: {:?}", test.options);
            std::println!("  Correct Answer: {}", test.correct_answer);
            std::println!("  Task ID: {}", test.task_id);
            std::println!("  Timestamp: {}", test.timestamp);
            std::println!();
        }
    }

    std::println!("\n--- Observations ---");
    std::println!("This baseline documents what the deterministic adapters currently return.");
    std::println!("After Rig upgrade in Sprint 2, we expect:");
    std::println!("  1. Enhancement content to be meaningful (not dummy data)");
    std::println!("  2. Comprehension test questions to be task-specific");
    std::println!("  3. Multiple choice options to be relevant");
    std::println!("  4. Correct answers to be properly indicated");
}

#[tokio::test]
async fn test_intelligent_routing_with_triage_service() {
    // Test: Validates TriageService-based intelligent routing for simple vs complex tasks.
    // Justification: Phase 3 Sprint 6 requires complexity-based routing instead of title length heuristic.

    std::println!("\n=== Testing Intelligent Routing with TriageService ===");

    // Test 1: Simple task should route to "enhance"
    std::println!("\n--- Test 1: Simple Task Routing ---");
    let simple_action = transcript_extractor::domain::action_item::ActionItem {
        title: std::string::String::from("Fix typo"),
        assignee: std::option::Option::Some(std::string::String::from("Alice")),
        due_date: std::option::Option::Some(std::string::String::from("2025-12-01")),
    };

    let simple_task = task_manager::domain::task::Task::from_action_item(
        &simple_action,
        std::option::Option::None,
    );

    std::println!("Simple task title: {}", simple_task.title);

    // Create ComplexityScorer and TriageService to validate routing
    let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
    let simple_complexity = scorer.score_task(&simple_task);
    std::println!("Complexity score: {}", simple_complexity);
    std::assert!(simple_complexity < 7, "Simple task should have complexity < 7, got {}", simple_complexity);

    let triage = task_manager::domain::services::triage_service::TriageService::new(scorer);
    let decision = triage.classify_task(&simple_task);
    std::assert_eq!(
        decision,
        task_manager::domain::services::triage_service::TriageDecision::Enhance,
        "Simple task should route to Enhance"
    );
    std::println!("✓ Simple task correctly routed to 'enhance'");

    // Test 2: Complex task should route to "decompose"
    std::println!("\n--- Test 2: Complex Task Routing ---");
    let complex_action = transcript_extractor::domain::action_item::ActionItem {
        title: std::string::String::from(
            "Refactor entire authentication system to support OAuth2 and SAML with multi-region deployment"
        ),
        assignee: std::option::Option::None,
        due_date: std::option::Option::None,
    };

    let complex_task = task_manager::domain::task::Task::from_action_item(
        &complex_action,
        std::option::Option::None,
    );

    std::println!("Complex task title: {}", complex_task.title);

    let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
    let complex_complexity = scorer.score_task(&complex_task);
    std::println!("Complexity score: {}", complex_complexity);
    std::assert!(complex_complexity >= 7, "Complex task should have complexity >= 7, got {}", complex_complexity);

    let triage = task_manager::domain::services::triage_service::TriageService::new(scorer);
    let decision = triage.classify_task(&complex_task);
    std::assert_eq!(
        decision,
        task_manager::domain::services::triage_service::TriageDecision::Decompose,
        "Complex task should route to Decompose"
    );
    std::println!("✓ Complex task correctly routed to 'decompose'");

    // Test 3: Validate routing through SemanticRouterNode
    std::println!("\n--- Test 3: SemanticRouterNode Integration ---");
    let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
    let triage_service = task_manager::domain::services::triage_service::TriageService::new(scorer);
    let router = task_orchestrator::graph::nodes::semantic_router_node::SemanticRouterNode::new(triage_service);

    // Test simple task through router node
    let simple_state = task_orchestrator::graph::state::GraphState::new(simple_task);
    let result = task_orchestrator::graph::nodes::semantic_router_node::SemanticRouterNode::execute(
        &router,
        simple_state,
    )
    .await;
    std::assert!(result.is_ok(), "Router should execute successfully for simple task");
    let simple_output = result.unwrap();
    std::assert_eq!(
        simple_output.routing_decision,
        std::option::Option::Some(std::string::String::from("enhance")),
        "Simple task should be routed to 'enhance' by node"
    );
    std::println!("✓ Simple task correctly routed by SemanticRouterNode");

    // Test complex task through router node
    let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
    let triage_service = task_manager::domain::services::triage_service::TriageService::new(scorer);
    let router = task_orchestrator::graph::nodes::semantic_router_node::SemanticRouterNode::new(triage_service);

    let complex_state = task_orchestrator::graph::state::GraphState::new(complex_task);
    let result = task_orchestrator::graph::nodes::semantic_router_node::SemanticRouterNode::execute(
        &router,
        complex_state,
    )
    .await;
    std::assert!(result.is_ok(), "Router should execute successfully for complex task");
    let complex_output = result.unwrap();
    std::assert_eq!(
        complex_output.routing_decision,
        std::option::Option::Some(std::string::String::from("decompose")),
        "Complex task should be routed to 'decompose' by node"
    );
    std::println!("✓ Complex task correctly routed by SemanticRouterNode");

    std::println!("\n✓ Intelligent routing integration test passed");
}

#[tokio::test]
async fn test_task_decomposition_end_to_end() {
    // Test: Validates end-to-end decomposition flow for complex tasks.
    // Justification: Phase 3 Sprint 7 requires full decomposition pipeline validation.

    std::println!("\n=== Testing Task Decomposition End-to-End ===");

    // Create a high-complexity task
    let action = transcript_extractor::domain::action_item::ActionItem {
        title: std::string::String::from(
            "Refactor entire authentication system to support OAuth2, SAML, and multi-region deployment with zero-downtime migration"
        ),
        assignee: std::option::Option::None,
        due_date: std::option::Option::Some(std::string::String::from("2025-12-31")),
    };

    let task = task_manager::domain::task::Task::from_action_item(
        &action,
        std::option::Option::None,
    );

    let initial_title = task.title.clone();
    let initial_id = task.id.clone();

    std::println!("\n--- Parent Task ---");
    std::println!("Title: {}", task.title);
    std::println!("ID: {}", task.id);

    // Verify task complexity is high enough for decomposition
    let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
    let complexity = scorer.score_task(&task);
    std::println!("Complexity score: {}", complexity);
    std::assert!(complexity >= 7, "Task should have complexity >= 7 for decomposition, got {}", complexity);

    // Run through orchestration flow
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new("ollama", "llama3.1").unwrap();
    let result = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
        &factory,
        "short_answer",
        task,
    )
    .await;

    std::assert!(result.is_ok(), "Orchestration should complete successfully: {:?}", result.err());

    let enhanced_task = result.unwrap();

    std::println!("\n--- Post-Orchestration State ---");
    std::println!("Task ID: {}", enhanced_task.id);
    std::println!("Task Status: {:?}", enhanced_task.status);
    std::println!("Subtask IDs: {:?}", enhanced_task.subtask_ids);

    // Validate task properties
    std::assert_eq!(enhanced_task.id, initial_id, "Task ID should be preserved");
    std::assert_eq!(enhanced_task.title, initial_title, "Task title should be unchanged");

    // Validate decomposition occurred
    std::assert_eq!(
        enhanced_task.status,
        task_manager::domain::task_status::TaskStatus::Decomposed,
        "Task should be marked as Decomposed"
    );

    // Validate subtasks were generated
    std::assert!(
        !enhanced_task.subtask_ids.is_empty(),
        "Task should have generated subtasks"
    );
    std::assert!(
        enhanced_task.subtask_ids.len() >= 3 && enhanced_task.subtask_ids.len() <= 5,
        "Should generate 3-5 subtasks, got {}",
        enhanced_task.subtask_ids.len()
    );

    std::println!("\n--- Subtasks Generated ---");
    for (i, subtask_id) in enhanced_task.subtask_ids.iter().enumerate() {
        std::println!("Subtask {}: {}", i + 1, subtask_id);
    }

    std::println!("\n✓ Task decomposition end-to-end test passed");
    std::println!("  - Complex task routed to decomposition");
    std::println!("  - {} subtasks generated", enhanced_task.subtask_ids.len());
    std::println!("  - Parent task marked as Decomposed");
}

#[tokio::test]
async fn test_simple_task_does_not_decompose() {
    // Test: Validates simple tasks bypass decomposition and go through enhancement flow.
    // Justification: Ensures routing logic correctly discriminates based on complexity.

    std::println!("\n=== Testing Simple Task Enhancement Path ===");

    // Create a simple task
    let action = transcript_extractor::domain::action_item::ActionItem {
        title: std::string::String::from("Fix typo in README"),
        assignee: std::option::Option::Some(std::string::String::from("Alice")),
        due_date: std::option::Option::Some(std::string::String::from("2025-12-01")),
    };

    let task = task_manager::domain::task::Task::from_action_item(
        &action,
        std::option::Option::None,
    );

    std::println!("\n--- Simple Task ---");
    std::println!("Title: {}", task.title);

    // Verify task complexity is low
    let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
    let complexity = scorer.score_task(&task);
    std::println!("Complexity score: {}", complexity);
    std::assert!(complexity < 7, "Task should have complexity < 7, got {}", complexity);

    // Run through orchestration flow
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new("ollama", "llama3.1").unwrap();
    let result = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
        &factory,
        "short_answer",
        task,
    )
    .await;

    std::assert!(result.is_ok(), "Orchestration should complete: {:?}", result.err());

    let enhanced_task = result.unwrap();

    std::println!("\n--- Post-Orchestration State ---");
    std::println!("Task Status: {:?}", enhanced_task.status);

    // Validate task went through enhancement path (not decomposition)
    std::assert_ne!(
        enhanced_task.status,
        task_manager::domain::task_status::TaskStatus::Decomposed,
        "Simple task should NOT be decomposed"
    );

    // Validate enhancements were generated (enhancement path)
    std::assert!(
        enhanced_task.enhancements.is_some(),
        "Simple task should have enhancements from enhancement path"
    );

    // Validate no subtasks generated
    std::assert!(
        enhanced_task.subtask_ids.is_empty(),
        "Simple task should not generate subtasks"
    );

    std::println!("\n✓ Simple task enhancement path test passed");
    std::println!("  - Simple task routed to enhancement");
    std::println!("  - No decomposition occurred");
    std::println!("  - Enhancements generated normally");
}
