//! Defines the PRDParserPort output port for parsing PRDs into tasks.
//!
//! This port represents the interface for converting Product Requirements Documents
//! into actionable tasks using LLM-based task decomposition.
//!
//! Revision History
//! - 2025-11-22T17:00:00Z @AI: Initial PRDParserPort for Rigger Phase 0 Sprint 0.3.

/// Port (interface) for parsing PRDs into task lists.
///
/// PRDParserPort defines the contract for adapters that can analyze a PRD
/// and generate a list of actionable tasks. Implementations typically use
/// LLMs to intelligently break down objectives into discrete work items.
///
/// # Object Safety
///
/// This trait is object-safe and uses async_trait to support async methods
/// in trait objects. All methods require Send + Sync for concurrent usage.
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::ports::prd_parser_port::PRDParserPort;
/// # use task_manager::domain::prd::PRD;
/// # async fn example<P: PRDParserPort>(parser: &P, prd: &PRD) {
/// let tasks = parser.parse_prd_to_tasks(prd).await.unwrap();
/// println!("Generated {} tasks from PRD", tasks.len());
/// # }
/// ```
#[async_trait::async_trait]
pub trait PRDParserPort: std::marker::Send + std::marker::Sync {
    /// Parses a PRD and generates a list of actionable tasks.
    ///
    /// This method analyzes the PRD's objectives, tech stack, and constraints
    /// to generate a comprehensive list of tasks required to complete the project.
    /// Tasks are returned with:
    /// - `source_prd_id` set to the PRD's ID for traceability
    /// - `title` derived from PRD objectives
    /// - `status` initialized to Todo
    ///
    /// # Arguments
    ///
    /// * `prd` - The Product Requirements Document to parse
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of generated tasks, or an error if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - LLM request fails
    /// - Response parsing fails
    /// - PRD content is invalid or empty
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::ports::prd_parser_port::PRDParserPort;
    /// # use task_manager::domain::prd::PRD;
    /// # async fn example<P: PRDParserPort>(parser: &P) {
    /// let prd = PRD::new(
    ///     "Build Rigger".to_string(),
    ///     vec!["Enable task decomposition".to_string()],
    ///     vec!["Rust".to_string()],
    ///     vec![],
    ///     "# Rigger PRD".to_string(),
    /// );
    ///
    /// let tasks = parser.parse_prd_to_tasks(&prd).await.unwrap();
    /// assert!(!tasks.is_empty());
    /// # }
    /// ```
    async fn parse_prd_to_tasks(
        &self,
        prd: &task_manager::domain::prd::PRD,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String>;
}
