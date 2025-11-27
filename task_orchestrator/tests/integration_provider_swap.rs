//! Integration test demonstrating provider swapping with ProviderFactory.
//!
//! This test validates that the ProviderFactory correctly creates adapters for
//! different LLM providers and handles provider-specific configurations.
//!
//! Revision History
//! - 2025-11-23 @AI: Create integration test for provider swapping (Phase 1 Sprint 3 Task 1.12).

#[tokio::test]
async fn test_provider_factory_ollama_works() {
    // 1. Create a factory for Ollama
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new(
        "ollama",
        "llama3.1",
    )
    .unwrap();

    std::assert_eq!(factory.provider(), "ollama");
    std::assert_eq!(factory.model(), "llama3.1");

    // 2. Create adapters using the factory
    let enh_adapter = factory.create_enhancement_adapter();
    std::assert!(enh_adapter.is_ok(), "Should create Ollama enhancement adapter");

    let ct_adapter = factory.create_comprehension_test_adapter();
    std::assert!(ct_adapter.is_ok(), "Should create Ollama comprehension test adapter");

    let prd_adapter = factory.create_prd_parser_adapter();
    std::assert!(prd_adapter.is_ok(), "Should create Ollama PRD parser adapter");

    let decomp_adapter = factory.create_task_decomposition_adapter();
    std::assert!(decomp_adapter.is_ok(), "Should create Ollama task decomposition adapter");
}

#[tokio::test]
async fn test_provider_factory_openai_not_yet_implemented() {
    // 1. Create a factory for OpenAI
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new(
        "openai",
        "gpt-4",
    )
    .unwrap();

    std::assert_eq!(factory.provider(), "openai");
    std::assert_eq!(factory.model(), "gpt-4");

    // 2. Set API key (required for OpenAI)
    unsafe {
        std::env::set_var("OPENAI_API_KEY", "test-key-123");
    }

    // 3. Verify that enhancement adapter returns "not yet implemented" error
    let enh_adapter = factory.create_enhancement_adapter();
    std::assert!(enh_adapter.is_err(), "OpenAI enhancement adapter should return error");
    if let std::result::Result::Err(e) = enh_adapter {
        let err_str = e.to_string();
        std::assert!(
            err_str.contains("not yet implemented"),
            "Error should mention not implemented: {}",
            err_str
        );
    }

    // 4. Clean up
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }
}

#[tokio::test]
async fn test_provider_factory_anthropic_not_yet_implemented() {
    // 1. Create a factory for Anthropic
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new(
        "anthropic",
        "claude-3-5-sonnet-20241022",
    )
    .unwrap();

    std::assert_eq!(factory.provider(), "anthropic");
    std::assert_eq!(factory.model(), "claude-3-5-sonnet-20241022");

    // 2. Set API key (required for Anthropic)
    unsafe {
        std::env::set_var("ANTHROPIC_API_KEY", "test-key-456");
    }

    // 3. Verify that enhancement adapter returns "not yet implemented" error
    let enh_adapter = factory.create_enhancement_adapter();
    std::assert!(enh_adapter.is_err(), "Anthropic enhancement adapter should return error");
    if let std::result::Result::Err(e) = enh_adapter {
        let err_str = e.to_string();
        std::assert!(
            err_str.contains("not yet implemented"),
            "Error should mention not implemented: {}",
            err_str
        );
    }

    // 4. Clean up
    unsafe {
        std::env::remove_var("ANTHROPIC_API_KEY");
    }
}

#[tokio::test]
async fn test_provider_swap_via_orchestrator() {
    // 1. Create Orchestrator with Ollama
    let orch_ollama = task_orchestrator::use_cases::orchestrator::Orchestrator::new(
        "ollama",
        "llama3.1",
        "short_answer",
    )
    .unwrap();

    std::assert_eq!(orch_ollama.provider(), "ollama");
    std::assert_eq!(orch_ollama.model(), "llama3.1");

    // 2. Run a task through Ollama orchestrator
    let action_item = transcript_extractor::domain::action_item::ActionItem {
        title: std::string::String::from("Test provider swap"),
        assignee: std::option::Option::None,
        due_date: std::option::Option::None,
    };
    let task = task_manager::domain::task::Task::from_action_item(&action_item, std::option::Option::None);

    let result = orch_ollama.run(task).await;
    std::assert!(result.is_ok(), "Ollama orchestration should succeed: {:?}", result.err());

    let enhanced_task = result.unwrap();
    std::assert!(enhanced_task.enhancements.is_some(), "Task should have enhancements");
    std::assert!(enhanced_task.comprehension_tests.is_some(), "Task should have comprehension tests");

    // 3. Demonstrate that we can create orchestrators for other providers
    // (They won't work until adapters are implemented, but factory creation should succeed)
    let orch_openai = task_orchestrator::use_cases::orchestrator::Orchestrator::new(
        "openai",
        "gpt-4",
        "multiple_choice",
    );
    std::assert!(orch_openai.is_ok(), "Should be able to create OpenAI orchestrator");
    let orch_openai = orch_openai.unwrap();
    std::assert_eq!(orch_openai.provider(), "openai");
    std::assert_eq!(orch_openai.model(), "gpt-4");

    let orch_anthropic = task_orchestrator::use_cases::orchestrator::Orchestrator::new(
        "anthropic",
        "claude-3-5-sonnet-20241022",
        "true_false",
    );
    std::assert!(orch_anthropic.is_ok(), "Should be able to create Anthropic orchestrator");
    let orch_anthropic = orch_anthropic.unwrap();
    std::assert_eq!(orch_anthropic.provider(), "anthropic");
    std::assert_eq!(orch_anthropic.model(), "claude-3-5-sonnet-20241022");
}

#[tokio::test]
async fn test_provider_factory_from_env() {
    // 1. Set environment variables for Ollama
    unsafe {
        std::env::set_var("TASK_ORCHESTRATOR_PROVIDER", "ollama");
        std::env::set_var("OLLAMA_MODEL", "qwen2.5");
        std::env::set_var("TEST_TYPE", "multiple_choice");
    }

    // 2. Create orchestrator from environment
    let orch = task_orchestrator::use_cases::orchestrator::Orchestrator::from_env();
    std::assert!(orch.is_ok(), "Should create orchestrator from env");

    let orch = orch.unwrap();
    std::assert_eq!(orch.provider(), "ollama");
    std::assert_eq!(orch.model(), "qwen2.5");
    std::assert_eq!(orch.test_type(), "multiple_choice");

    // 3. Clean up
    unsafe {
        std::env::remove_var("TASK_ORCHESTRATOR_PROVIDER");
        std::env::remove_var("OLLAMA_MODEL");
        std::env::remove_var("TEST_TYPE");
    }
}

#[tokio::test]
async fn test_provider_factory_invalid_provider() {
    // Verify that unsupported providers are rejected
    let result = task_orchestrator::adapters::provider_factory::ProviderFactory::new(
        "unsupported-provider",
        "some-model",
    );

    std::assert!(result.is_err(), "Should reject unsupported provider");
    if let std::result::Result::Err(e) = result {
        let err_str = e.to_string();
        std::assert!(
            err_str.contains("Unsupported provider"),
            "Error should mention unsupported provider: {}",
            err_str
        );
    }
}
