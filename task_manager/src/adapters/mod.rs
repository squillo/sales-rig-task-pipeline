//! Adapters layer for task management.
//!
//! This module contains infrastructure implementations of the task repository
//! port, providing concrete storage solutions following HEXSER patterns.
//!
//! Revision History
//! - 2025-11-26T08:05:00Z @AI: Add sqlite_agent_tool_adapter and sqlite_persona_adapter for Phase 3 persona management.
//! - 2025-11-24T05:00:00Z @AI: Add sqlite_project_adapter for Phase 1 TUI project architecture.
//! - 2025-11-14T16:22:00Z @AI: Export sqlite_task_adapter for SQLite persistence via sqlx.
//! - 2025-11-06T19:16:00Z @AI: Initial adapters module created from transcript_processor split.

pub mod in_memory_task_adapter;
pub mod sqlite_task_adapter;
pub mod sqlite_project_adapter;
pub mod sqlite_agent_tool_adapter;
pub mod sqlite_persona_adapter;
