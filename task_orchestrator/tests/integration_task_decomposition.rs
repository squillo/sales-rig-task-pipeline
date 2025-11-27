//! End-to-end integration test for task decomposition flow.
//!
//! This test validates that high-complexity tasks are correctly routed through
//! the decomposition path, generating 3-5 subtasks with proper parent linkage.
//!
//! Revision History
//! - 2025-11-23T19:00:00Z @AI: Create decomposition integration test for Phase 3 Sprint 7 Task 3.9.

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore] // Requires Ollama service running locally
    async fn test_task_decomposition_end_to_end() {
        // Test: Validates end-to-end decomposition flow for high-complexity task.
        // Justification: Ensures routing_decision="decompose" path works correctly
        // with real LLM generating subtasks.

        // Create high-complexity task (score = 9)
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from(
                "Refactor entire authentication system to support OAuth2, SAML, and LDAP with multi-region deployment and zero-downtime migration"
            ),
            assignee: std::option::Option::None,  // +1
            due_date: std::option::Option::None,  // +1
        };
        // Complexity score: 3 (base) + 1 (long title >50) + 2 (keyword "refactor") + 1 (no assignee) + 1 (no due date) + 1 (very long >100) = 9

        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);

        // Verify complexity score
        let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
        let complexity = scorer.score_task(&task);
        std::assert!(complexity >= 7, "Task should be high complexity (got {})", complexity);

        // Run through orchestrator using ProviderFactory
        let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new(
            "ollama",
            "llama3.1",
        ).expect("Failed to create provider factory");

        // Run task with flow
        let result = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
            &factory,
            "short_answer",
            task.clone(),
        ).await;

        std::assert!(result.is_ok(), "Decomposition flow should complete: {:?}", result.err());

        let enhanced_task = result.unwrap();

        // Verify routing decision was "decompose"
        // Note: This would require GraphState to persist routing_decision, which may not be
        // directly observable from the returned task. For now, we verify subtasks exist.

        // Verify subtasks were created
        std::assert!(
            !enhanced_task.subtask_ids.is_empty(),
            "Task should have subtasks after decomposition"
        );
        std::assert!(
            enhanced_task.subtask_ids.len() >= 3 && enhanced_task.subtask_ids.len() <= 5,
            "Should generate 3-5 subtasks, got {}",
            enhanced_task.subtask_ids.len()
        );

        // Verify parent task status
        std::assert_eq!(
            enhanced_task.status,
            task_manager::domain::task_status::TaskStatus::Decomposed,
            "Parent task should have Decomposed status"
        );

        println!("✓ Decomposition test passed:");
        println!("  Parent task: {}", enhanced_task.title);
        println!("  Complexity: {}", complexity);
        println!("  Subtasks generated: {}", enhanced_task.subtask_ids.len());
        for (idx, subtask_id) in enhanced_task.subtask_ids.iter().enumerate() {
            println!("    {}. {}", idx + 1, subtask_id);
        }
    }

    #[tokio::test]
    async fn test_simple_task_does_not_decompose() {
        // Test: Validates that simple tasks skip decomposition path.
        // Justification: Ensures routing_decision="enhance" path for low-complexity tasks.

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Fix typo in README"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-01")),
        };
        // Complexity score: 3 (base) = 3

        let task = task_manager::domain::task::Task::from_action_item(&action, std::option::Option::None);

        // Verify complexity score is low
        let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
        let complexity = scorer.score_task(&task);
        std::assert!(complexity < 7, "Task should be low complexity (got {})", complexity);

        // Run through orchestrator (uses deterministic adapters in test mode)
        let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new(
            "ollama",
            "llama3.1",
        ).expect("Failed to create provider factory");

        let result = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
            &factory,
            "short_answer",
            task.clone(),
        ).await;

        std::assert!(result.is_ok(), "Enhancement flow should complete: {:?}", result.err());

        let enhanced_task = result.unwrap();

        // Verify subtasks were NOT created
        std::assert!(
            enhanced_task.subtask_ids.is_empty(),
            "Simple task should not have subtasks"
        );

        // Verify task went through enhancement path instead
        std::assert!(
            enhanced_task.enhancements.is_some(),
            "Simple task should have enhancements"
        );

        println!("✓ Simple task test passed:");
        println!("  Task: {}", enhanced_task.title);
        println!("  Complexity: {}", complexity);
        println!("  Enhancements: {}", enhanced_task.enhancements.as_ref().map(|e| e.len()).unwrap_or(0));
        println!("  Subtasks: {} (expected 0)", enhanced_task.subtask_ids.len());
    }

    #[tokio::test]
    async fn test_intelligent_routing_with_triage_service() {
        // Test: Validates TriageService integration in routing decisions.
        // Justification: Ensures SemanticRouterNode uses real complexity scoring.

        let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
        let triage = task_manager::domain::services::triage_service::TriageService::new(scorer);

        // Test simple task
        let simple_action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Update docs"),
            assignee: std::option::Option::Some(std::string::String::from("Bob")),
            due_date: std::option::Option::Some(std::string::String::from("2025-11-30")),
        };
        let simple_task = task_manager::domain::task::Task::from_action_item(&simple_action, std::option::Option::None);

        let decision = triage.classify_task(&simple_task);
        std::assert_eq!(
            decision,
            task_manager::domain::services::triage_service::TriageDecision::Enhance,
            "Simple task should route to Enhance"
        );

        // Test complex task
        let complex_action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from(
                "Migrate entire database infrastructure to distributed system with replication and sharding"
            ),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let complex_task = task_manager::domain::task::Task::from_action_item(&complex_action, std::option::Option::None);

        let decision = triage.classify_task(&complex_task);
        std::assert_eq!(
            decision,
            task_manager::domain::services::triage_service::TriageDecision::Decompose,
            "Complex task should route to Decompose"
        );

        println!("✓ Intelligent routing test passed");
    }
}
