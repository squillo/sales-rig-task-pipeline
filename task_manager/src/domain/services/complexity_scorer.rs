//! ComplexityScorer domain service for analyzing task complexity.
//!
//! This service implements a heuristic algorithm to score task complexity on a
//! 1-10 scale. Higher scores indicate more complex tasks that may require
//! decomposition into subtasks. The scoring is based on task characteristics
//! like title length, architectural keywords, ownership clarity, and reasoning depth.
//!
//! Revision History
//! - 2025-11-23T15:40:00Z @AI: Create ComplexityScorer for Phase 2 Sprint 5 Task 2.3.

/// Stateless domain service for scoring task complexity.
///
/// ComplexityScorer analyzes a Task and returns a complexity score from 1-10,
/// where higher scores indicate more complex tasks. The scoring algorithm uses
/// heuristics based on task characteristics to estimate complexity.
///
/// # Scoring Algorithm
///
/// - **Base score**: 3 (default for all tasks)
/// - **+1**: Title length > 50 characters (indicates detailed requirements)
/// - **+2**: Title contains architectural keywords (refactor, migrate, redesign, rewrite, architect)
/// - **+1**: No assignee specified (unclear ownership increases risk)
/// - **+1**: No due date specified (unclear timeline increases uncertainty)
/// - **+2**: Reasoning field > 200 characters (extensive explanation indicates complexity)
/// - **Maximum**: 10 (capped to maintain scale)
///
/// # Examples
///
/// ```
/// # use task_manager::domain::services::complexity_scorer::ComplexityScorer;
/// # use task_manager::domain::task::Task;
/// # use transcript_extractor::domain::action_item::ActionItem;
/// let scorer = ComplexityScorer::new();
///
/// // Simple task: base score 3
/// let simple_action = ActionItem {
///     title: std::string::String::from("Fix typo"),
///     assignee: Some(std::string::String::from("Alice")),
///     due_date: Some(std::string::String::from("2025-12-01")),
/// };
/// let simple_task = Task::from_action_item(&simple_action, None);
/// assert_eq!(scorer.score_task(&simple_task), 3);
///
/// // Complex task: base 3 + long title (+1) + keyword (+2) = 6
/// let complex_action = ActionItem {
///     title: std::string::String::from("Refactor the entire authentication system to use OAuth2 with multiple providers"),
///     assignee: None,  // +1
///     due_date: None,  // +1
/// };
/// let complex_task = Task::from_action_item(&complex_action, None);
/// assert_eq!(scorer.score_task(&complex_task), 8);  // 3 + 1 + 2 + 1 + 1 = 8
/// ```
#[derive(Debug, Clone)]
pub struct ComplexityScorer;

impl ComplexityScorer {
    /// Creates a new ComplexityScorer instance.
    ///
    /// ComplexityScorer is stateless, so this constructor simply returns a new instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::services::complexity_scorer::ComplexityScorer;
    /// let scorer = ComplexityScorer::new();
    /// ```
    pub fn new() -> Self {
        ComplexityScorer
    }

    /// Scores a task's complexity on a 1-10 scale.
    ///
    /// Analyzes the task using a heuristic algorithm based on title length,
    /// keywords, ownership clarity, timeline clarity, and reasoning depth.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to score
    ///
    /// # Returns
    ///
    /// A complexity score from 1-10, where higher scores indicate more complex tasks.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::services::complexity_scorer::ComplexityScorer;
    /// # use task_manager::domain::task::Task;
    /// # use transcript_extractor::domain::action_item::ActionItem;
    /// let scorer = ComplexityScorer::new();
    ///
    /// let action = ActionItem {
    ///     title: std::string::String::from("Update README"),
    ///     assignee: Some(std::string::String::from("Bob")),
    ///     due_date: Some(std::string::String::from("2025-12-15")),
    /// };
    /// let task = Task::from_action_item(&action, None);
    ///
    /// let score = scorer.score_task(&task);
    /// assert!(score >= 1 && score <= 10);
    /// ```
    pub fn score_task(&self, task: &crate::domain::task::Task) -> u8 {
        let mut score: u8 = 3; // Base score

        // +1 if title > 50 chars (detailed)
        if task.title.len() > 50 {
            score += 1;
        }

        // +2 if title contains architectural keywords
        let title_lower = task.title.to_lowercase();
        let architectural_keywords = [
            "refactor",
            "migrate",
            "redesign",
            "rewrite",
            "architect",
        ];
        if architectural_keywords.iter().any(|kw| title_lower.contains(kw)) {
            score += 2;
        }

        // +1 if agent_persona is None (unclear ownership)
        if task.agent_persona.is_none() {
            score += 1;
        }

        // +1 if due_date is None (unclear timeline)
        if task.due_date.is_none() {
            score += 1;
        }

        // +2 if reasoning field > 200 chars (extensive explanation)
        if let std::option::Option::Some(ref reasoning) = task.reasoning {
            if reasoning.len() > 200 {
                score += 2;
            }
        }

        // Cap at 10
        if score > 10 {
            score = 10;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_minimal_task() {
        // Test: Validates base score of 3 for minimal task with all clarity fields set.
        // Justification: Ensures baseline scoring works correctly for well-defined simple tasks.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Fix typo"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-01")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let score = scorer.score_task(&task);
        std::assert_eq!(score, 3, "Minimal task should score base 3");
    }

    #[test]
    fn test_scorer_long_title() {
        // Test: Validates +1 bonus for title > 50 characters.
        // Justification: Long titles indicate detailed requirements, increasing complexity.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Update the user authentication flow to support multi-factor authentication"),
            assignee: std::option::Option::Some(std::string::String::from("Bob")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-15")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let score = scorer.score_task(&task);
        std::assert_eq!(score, 4, "Long title should score 3 + 1 = 4");
    }

    #[test]
    fn test_scorer_architectural_keyword() {
        // Test: Validates +2 bonus for architectural keywords in title.
        // Justification: Architectural work (refactor, migrate, redesign) is inherently complex.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor payment processing"),
            assignee: std::option::Option::Some(std::string::String::from("Charlie")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-20")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let score = scorer.score_task(&task);
        std::assert_eq!(score, 5, "Architectural keyword should score 3 + 2 = 5");
    }

    #[test]
    fn test_scorer_no_assignee() {
        // Test: Validates +1 penalty for missing assignee.
        // Justification: Unclear ownership increases coordination complexity and risk.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Review API design"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::Some(std::string::String::from("2025-12-10")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let score = scorer.score_task(&task);
        std::assert_eq!(score, 4, "Missing assignee should score 3 + 1 = 4");
    }

    #[test]
    fn test_scorer_no_due_date() {
        // Test: Validates +1 penalty for missing due date.
        // Justification: Unclear timeline increases uncertainty and planning complexity.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Research options"),
            assignee: std::option::Option::Some(std::string::String::from("Dana")),
            due_date: std::option::Option::None,
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let score = scorer.score_task(&task);
        std::assert_eq!(score, 4, "Missing due date should score 3 + 1 = 4");
    }

    #[test]
    fn test_scorer_long_reasoning() {
        // Test: Validates +2 bonus for reasoning > 200 characters.
        // Justification: Extensive reasoning indicates complex requirements or constraints.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Implement feature"),
            assignee: std::option::Option::Some(std::string::String::from("Eve")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-25")),
        };
        let mut task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        // Add long reasoning (> 200 chars)
        task.reasoning = std::option::Option::Some(std::string::String::from(
            "This feature requires careful coordination across multiple teams. We need to ensure backward compatibility with the existing API while introducing the new functionality. The implementation must account for edge cases in the current system and provide comprehensive error handling for the new code paths that will be introduced."
        ));

        let score = scorer.score_task(&task);
        std::assert_eq!(score, 5, "Long reasoning should score 3 + 2 = 5");
    }

    #[test]
    fn test_scorer_maximal_complexity() {
        // Test: Validates score caps at 10 for maximally complex tasks.
        // Justification: Ensures scoring scale remains bounded at 10 maximum.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor the entire microservices architecture to support multi-region deployment with automated failover"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        // Add extensive reasoning (> 200 chars)
        task.reasoning = std::option::Option::Some(std::string::String::from(
            "This is a comprehensive architectural overhaul requiring deep coordination across all engineering teams. We must maintain zero-downtime deployment capability while introducing fundamental changes to the system's infrastructure. The complexity includes distributed systems challenges, data consistency concerns, and extensive testing requirements to validate the new architecture under various failure scenarios."
        ));

        let score = scorer.score_task(&task);
        // Base 3 + long title (+1) + keyword (+2) + no assignee (+1) + no due date (+1) + long reasoning (+2) = 10
        std::assert_eq!(score, 10, "Maximal complexity should cap at 10");
    }

    #[test]
    fn test_scorer_edge_case_exactly_50_chars() {
        // Test: Validates edge case where title is exactly 50 characters (should NOT get bonus).
        // Justification: Boundary condition testing ensures > comparison works correctly.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("12345678901234567890123456789012345678901234567890"), // Exactly 50 chars
            assignee: std::option::Option::Some(std::string::String::from("Frank")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-30")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let score = scorer.score_task(&task);
        std::assert_eq!(score, 3, "Exactly 50 chars should NOT get length bonus (> 50 required)");
    }

    #[test]
    fn test_scorer_edge_case_exactly_51_chars() {
        // Test: Validates edge case where title is 51 characters (should get bonus).
        // Justification: Boundary condition testing ensures > 50 comparison works correctly.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("123456789012345678901234567890123456789012345678901"), // 51 chars
            assignee: std::option::Option::Some(std::string::String::from("Grace")),
            due_date: std::option::Option::Some(std::string::String::from("2025-12-31")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let score = scorer.score_task(&task);
        std::assert_eq!(score, 4, "51 chars should get length bonus (> 50)");
    }

    #[test]
    fn test_scorer_multiple_architectural_keywords() {
        // Test: Validates that multiple architectural keywords still only give +2 bonus (not cumulative).
        // Justification: Ensures scoring doesn't double-count architectural complexity.
        let scorer = ComplexityScorer::new();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Refactor and migrate and redesign system"),
            assignee: std::option::Option::Some(std::string::String::from("Henry")),
            due_date: std::option::Option::Some(std::string::String::from("2026-01-01")),
        };
        let task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);

        let score = scorer.score_task(&task);
        std::assert_eq!(score, 5, "Multiple keywords should still only give +2 (not cumulative)");
    }
}
