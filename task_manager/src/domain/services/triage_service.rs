//! TriageService domain service for intelligent task classification.
//!
//! This service uses ComplexityScorer to analyze tasks and make routing decisions
//! for the orchestration pipeline. High-complexity tasks (score >= 7) are routed
//! to decomposition for breakdown into subtasks, while simpler tasks go through
//! the standard enhancement flow.
//!
//! Revision History
//! - 2025-11-23T16:00:00Z @AI: Create TriageService for Phase 2 Sprint 5 Task 2.5.

/// Routing decision for task orchestration.
///
/// TriageDecision determines which orchestration path a task should follow
/// based on its complexity score.
///
/// # Variants
///
/// * `Enhance` - Task should follow standard enhancement flow (score < 7)
/// * `Decompose` - Task should be decomposed into subtasks (score >= 7)
///
/// # Examples
///
/// ```
/// # use task_manager::domain::services::triage_service::TriageDecision;
/// let decision = TriageDecision::Enhance;
/// assert_eq!(decision, TriageDecision::Enhance);
///
/// let complex_decision = TriageDecision::Decompose;
/// assert_eq!(complex_decision, TriageDecision::Decompose);
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TriageDecision {
    /// Task should follow standard enhancement flow.
    Enhance,

    /// Task should be decomposed into subtasks.
    Decompose,
}

/// Domain service for intelligent task triage and routing.
///
/// TriageService analyzes tasks using ComplexityScorer and makes routing
/// decisions for the orchestration pipeline. This enables intelligent workflow
/// branching based on task complexity.
///
/// # Classification Logic
///
/// - **Complexity score < 7**: Route to Enhance (standard flow)
/// - **Complexity score >= 7**: Route to Decompose (breakdown into subtasks)
///
/// # Examples
///
/// ```
/// # use task_manager::domain::services::triage_service::{TriageService, TriageDecision};
/// # use task_manager::domain::services::complexity_scorer::ComplexityScorer;
/// # use task_manager::domain::task::Task;
/// # use transcript_extractor::domain::action_item::ActionItem;
/// let scorer = ComplexityScorer::new();
/// let triage = TriageService::new(scorer);
///
/// // Simple task
/// let simple_action = ActionItem {
///     title: std::string::String::from("Fix typo"),
///     assignee: Some(std::string::String::from("Alice")),
///     due_date: Some(std::string::String::from("2025-12-01")),
/// };
/// let simple_task = Task::from_action_item(&simple_action, None);
/// assert_eq!(triage.classify_task(&simple_task), TriageDecision::Enhance);
///
/// // Complex task
/// let complex_action = ActionItem {
///     title: std::string::String::from("Refactor entire authentication system to support OAuth2 and SAML with multi-region deployment"),
///     assignee: None,
///     due_date: None,
/// };
/// let complex_task = Task::from_action_item(&complex_action, None);
/// assert_eq!(triage.classify_task(&complex_task), TriageDecision::Decompose);
/// ```
#[derive(Debug, Clone)]
pub struct TriageService {
    complexity_scorer: crate::domain::services::complexity_scorer::ComplexityScorer,
}

impl TriageService {
    /// Creates a new TriageService with the provided ComplexityScorer.
    ///
    /// # Arguments
    ///
    /// * `complexity_scorer` - The scorer to use for complexity analysis
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::services::triage_service::TriageService;
    /// # use task_manager::domain::services::complexity_scorer::ComplexityScorer;
    /// let scorer = ComplexityScorer::new();
    /// let triage = TriageService::new(scorer);
    /// ```
    pub fn new(complexity_scorer: crate::domain::services::complexity_scorer::ComplexityScorer) -> Self {
        TriageService { complexity_scorer }
    }

    /// Classifies a task and determines its routing decision.
    ///
    /// Scores the task using ComplexityScorer and applies the triage threshold:
    /// - Score >= 7: Decompose (high complexity)
    /// - Score < 7: Enhance (normal flow)
    ///
    /// # Arguments
    ///
    /// * `task` - The task to classify
    ///
    /// # Returns
    ///
    /// `TriageDecision::Enhance` or `TriageDecision::Decompose`
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::services::triage_service::{TriageService, TriageDecision};
    /// # use task_manager::domain::services::complexity_scorer::ComplexityScorer;
    /// # use task_manager::domain::task::Task;
    /// # use transcript_extractor::domain::action_item::ActionItem;
    /// let scorer = ComplexityScorer::new();
    /// let triage = TriageService::new(scorer);
    ///
    /// let action = ActionItem {
    ///     title: std::string::String::from("Update README"),
    ///     assignee: Some(std::string::String::from("Bob")),
    ///     due_date: Some(std::string::String::from("2025-12-15")),
    /// };
    /// let task = Task::from_action_item(&action, None);
    ///
    /// let decision = triage.classify_task(&task);
    /// assert_eq!(decision, TriageDecision::Enhance);
    /// ```
    pub fn classify_task(&self, task: &crate::domain::task::Task) -> TriageDecision {
        let complexity_score = self.complexity_scorer.score_task(task);

        if complexity_score >= 7 {
            TriageDecision::Decompose
        } else {
            TriageDecision::Enhance
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triage_simple_task_goes_to_enhance() {
        // Test: Validates simple task (score 3) routes to Enhance.
        // Justification: Ensures low-complexity tasks follow standard enhancement flow.
        let scorer = crate::domain::services::complexity_scorer::ComplexityScorer::new();
        let triage = TriageService::new(scorer);

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Fix typo"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-01")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let decision = triage.classify_task(&task);
        std::assert_eq!(decision, TriageDecision::Enhance, "Simple task should route to Enhance");
    }

    #[test]
    fn test_triage_complex_task_goes_to_decompose() {
        // Test: Validates complex task (score >= 7) routes to Decompose.
        // Justification: Ensures high-complexity tasks are broken down for manageability.
        let scorer = crate::domain::services::complexity_scorer::ComplexityScorer::new();
        let triage = TriageService::new(scorer);

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor entire authentication system to support OAuth2 and SAML with multi-region deployment"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let decision = triage.classify_task(&task);
        std::assert_eq!(decision, TriageDecision::Decompose, "Complex task should route to Decompose");
    }

    #[test]
    fn test_triage_boundary_score_6_goes_to_enhance() {
        // Test: Validates boundary case where score=6 (just below threshold) routes to Enhance.
        // Justification: Ensures threshold works correctly (>= 7 required for Decompose).
        let scorer = crate::domain::services::complexity_scorer::ComplexityScorer::new();
        let triage = TriageService::new(scorer);

        // Create task that scores exactly 6: base 3 + long title (+1) + keyword (+2) = 6
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor the user authentication module for better security"),
            assignee: std::option::Option::Some(std::string::String::from("Charlie")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-20")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let decision = triage.classify_task(&task);
        std::assert_eq!(decision, TriageDecision::Enhance, "Score 6 should route to Enhance (< 7 threshold)");
    }

    #[test]
    fn test_triage_boundary_score_7_goes_to_decompose() {
        // Test: Validates boundary case where score=7 (at threshold) routes to Decompose.
        // Justification: Ensures threshold inclusivity (>= 7 includes 7).
        let scorer = crate::domain::services::complexity_scorer::ComplexityScorer::new();
        let triage = TriageService::new(scorer);

        // Create task that scores exactly 7: base 3 + long title (+1) + keyword (+2) + no assignee (+1) = 7
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor the payment processing system for PCI compliance"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::Some(std::string::String::from("2025-12-25")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let decision = triage.classify_task(&task);
        std::assert_eq!(decision, TriageDecision::Decompose, "Score 7 should route to Decompose (>= 7 threshold)");
    }

    #[test]
    fn test_triage_decision_equality() {
        // Test: Validates TriageDecision enum equality and inequality.
        // Justification: Ensures enum comparison works for routing logic.
        std::assert_eq!(TriageDecision::Enhance, TriageDecision::Enhance);
        std::assert_eq!(TriageDecision::Decompose, TriageDecision::Decompose);
        std::assert_ne!(TriageDecision::Enhance, TriageDecision::Decompose);
    }

    #[test]
    fn test_triage_decision_clone() {
        // Test: Validates TriageDecision can be cloned.
        // Justification: Ensures enum can be passed around without ownership issues.
        let decision = TriageDecision::Enhance;
        let cloned = decision.clone();
        std::assert_eq!(decision, cloned);
    }
}
