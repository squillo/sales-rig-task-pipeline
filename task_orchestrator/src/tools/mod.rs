//! Tool definitions for LLM agent integration.
//!
//! This module provides Rig-compatible tool implementations that allow the LLM agent
//! to interact with the Rigger system. Tools enable the agent to search tasks, query
//! artifacts semantically, and access project knowledge.
//!
//! Revision History
//! - 2025-12-05T00:00:00Z @AI: Export all Rig tools for LLM agent integration (fixed module names).
//! - 2025-12-04T00:00:00Z @AI: Initial tools module for LLM agent tool calling support.

pub mod search_artifacts_tool;
pub mod search_tasks_tool;
pub mod get_task_details_tool;
pub mod file_system_tool;
pub mod get_prd_summary_tool;
pub mod list_project_artifacts_tool;

pub use search_artifacts_tool::SearchArtifactsTool;
pub use search_tasks_tool::SearchTasksTool;
pub use get_task_details_tool::GetTaskDetailsTool;
pub use file_system_tool::FileSystemTool;
pub use get_prd_summary_tool::GetPRDSummaryTool;
pub use list_project_artifacts_tool::ListProjectArtifactsTool;
