//! SemanticRouterNode classifies tasks to decide routing.
//!
//! This node uses TriageService to analyze task complexity and make intelligent
//! routing decisions. High-complexity tasks (score >= 7) are routed to decomposition,
//! while simpler tasks follow the standard enhancement flow.
//!
//! Revision History
//! - 2025-11-23T23:00:00Z @AI: Document heterogeneous pipeline routing strategy (Phase 5 Sprint 10 Task 5.4).
//! - 2025-11-23T16:15:00Z @AI: Upgrade to use TriageService for intelligent routing (Phase 3 Sprint 6).
//! - 2025-11-12T21:41:00Z @AI: Add SemanticRouterNode with deterministic classifier and tests.
//!
//! # Heterogeneous Pipeline Notes
//!
//! This node uses HEURISTIC-BASED routing (ComplexityScorer) rather than LLM-based
//! classification. This is intentional for maximum speed - routing decisions must be
//! sub-millisecond to avoid becoming a bottleneck.
//!
//! If LLM-based routing is ever needed, it should use ModelRole::Router (Phi-3-mini)
//! which provides 15-30 tokens/second inference for fast classification tasks.

/// Node that sets `routing_decision` based on task complexity analysis.
///
/// SemanticRouterNode uses TriageService to analyze the task and determine
/// the appropriate routing path through the orchestration pipeline.
pub struct SemanticRouterNode {
    triage_service: task_manager::domain::services::triage_service::TriageService,
}

impl SemanticRouterNode {
    /// Creates a new SemanticRouterNode with the provided TriageService.
    ///
    /// # Arguments
    ///
    /// * `triage_service` - The triage service to use for routing decisions
    pub fn new(triage_service: task_manager::domain::services::triage_service::TriageService) -> Self {
        SemanticRouterNode { triage_service }
    }

    /// Executes routing logic using intelligent complexity-based classification.
    ///
    /// Uses TriageService to analyze the task and set the routing_decision:
    /// - "enhance" for simple tasks (complexity score < 7)
    /// - "decompose" for complex tasks (complexity score >= 7)
    pub async fn execute(
        &self,
        mut state: crate::graph::state::GraphState,
    ) -> std::result::Result<crate::graph::state::GraphState, std::string::String> {
        let decision = self.triage_service.classify_task(&state.task);

        let route = match decision {
            task_manager::domain::services::triage_service::TriageDecision::Enhance => "enhance",
            task_manager::domain::services::triage_service::TriageDecision::Decompose => "decompose",
        };

        state.routing_decision = std::option::Option::Some(std::string::String::from(route));
        std::result::Result::Ok(state)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_routing_simple_task_goes_to_enhance() {
        // Test: Validates simple task (low complexity) routes to "enhance".
        // Justification: Ensures TriageService integration works for standard enhancement flow.
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Fix typo"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-01")),
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);

        let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
        let triage = task_manager::domain::services::triage_service::TriageService::new(scorer);
        let node = super::SemanticRouterNode::new(triage);

        let out = super::SemanticRouterNode::execute(&node, state).await.unwrap();
        std::assert_eq!(out.routing_decision, std::option::Option::Some(std::string::String::from("enhance")));
    }

    #[tokio::test]
    async fn test_routing_complex_task_goes_to_decompose() {
        // Test: Validates complex task (high complexity) routes to "decompose".
        // Justification: Ensures TriageService integration works for decomposition flow.
        let ai = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor entire authentication system to support OAuth2 and SAML with multi-region deployment"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = task_manager::domain::task::Task::from_action_item(&ai, std::option::Option::None);
        let state = crate::graph::state::GraphState::new(task);

        let scorer = task_manager::domain::services::complexity_scorer::ComplexityScorer::new();
        let triage = task_manager::domain::services::triage_service::TriageService::new(scorer);
        let node = super::SemanticRouterNode::new(triage);

        let out = super::SemanticRouterNode::execute(&node, state).await.unwrap();
        std::assert_eq!(out.routing_decision, std::option::Option::Some(std::string::String::from("decompose")));
    }
}
