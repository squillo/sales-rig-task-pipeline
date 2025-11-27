//! Provides a human-readable description of the task_orchestrator architecture.
//!
//! This function summarizes the Ports, Adapters, Graph components, and Use Cases
//! to help developers "see into" the architecture at a glance without external
//! tooling. It complements HEXSER by surfacing where HexEntity/HexAdapter are
//! applied and which boundaries exist.
//!
//! Revision History
//! - 2025-11-13T21:46:00Z @AI: Add Orchestrator facade to description and extend unit test.
//! - 2025-11-13T21:06:00Z @AI: Update description to reflect unified graph_flow and add run_task_with_flow.
//! - 2025-11-12T17:20:00Z @AI: Introduce describe_architecture() with unit tests.

/// Returns a multi-line string describing the orchestrator architecture.
///
/// The description includes:
/// - Ports (async traits)
/// - Adapters (HexAdapter-derived)
/// - Graph (state and nodes)
/// - Use cases (application layer helpers)
///
/// # Examples
///
/// ```
/// let s = task_orchestrator::architecture::describe_architecture::describe_architecture();
/// assert!(s.contains("Ports"));
/// ```
pub fn describe_architecture() -> String {
    let mut s = String::new();

    // Header
    s.push_str("task_orchestrator Architecture\n");
    s.push_str("===============================\n");

    // Ports
    s.push_str("\n[Ports]\n");
    s.push_str("- task_enhancement_port::TaskEnhancementPort (async trait)\n");
    s.push_str("- comprehension_test_port::ComprehensionTestPort (async trait)\n");

    // Adapters
    s.push_str("\n[Adapters]\n");
    s.push_str("- ollama_enhancement_adapter::OllamaEnhancementAdapter (#[derive(hexser::HexAdapter)])\n");
    s.push_str("- ollama_comprehension_test_adapter::OllamaComprehensionTestAdapter (#[derive(hexser::HexAdapter)])\n");

    // Graph
    s.push_str("\n[Graph]\n");
    s.push_str("- state::GraphState { task, routing_decision }\n");
    s.push_str("- nodes::semantic_router_node::SemanticRouterNode\n");
    s.push_str("- nodes::enhancement_node::EnhancementNode\n");
    s.push_str("- nodes::comprehension_test_node::ComprehensionTestNode\n");
    s.push_str("- nodes::check_test_result_node::CheckTestResultNode\n");
    s.push_str("- orchestrator_graph::OrchestratorGraph (placeholder)\n");
    s.push_str("- build_graph::build_graph() (placeholder returns OrchestratorGraph)\n");
    s.push_str("- flow_integration (graph-flow integration skeleton)\n");

    // Use cases
    s.push_str("\n[Use Cases]\n");
    s.push_str("- task_graph_runner::TaskGraphRunner (sequential runner)\n");
    s.push_str("- run_task_with_ollama::run_task_with_ollama (convenience helper)\n");
    s.push_str("- run_task_with_flow::run_task_with_flow (unified runtime helper)\n");
    s.push_str("- orchestrator::Orchestrator (facade for running flows)\n");

    s
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_describe_includes_key_sections() {
        let s = super::describe_architecture();
        std::assert!(s.contains("[Ports]"));
        std::assert!(s.contains("TaskEnhancementPort"));
        std::assert!(s.contains("[Adapters]"));
        std::assert!(s.contains("HexAdapter"));
        std::assert!(s.contains("[Graph]"));
        std::assert!(s.contains("GraphState"));
        std::assert!(s.contains("SemanticRouterNode"));
        std::assert!(s.contains("[Use Cases]"));
        std::assert!(s.contains("TaskGraphRunner"));
        std::assert!(s.contains("run_task_with_flow"));
        std::assert!(s.contains("Orchestrator"));
    }
}
