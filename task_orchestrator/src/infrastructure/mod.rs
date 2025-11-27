//! Infrastructure module declarations for task_orchestrator.
//!
//! This module hosts infrastructure components such as persistence backends,
//! LLM response parsers, and other integrations with external tools while keeping
//! domain and application layers free of infrastructure concerns.
//!
//! Modules:
//! - `llm_parsers`: Parse LLM responses into structured domain entities
//! - `sqlite_session_storage`: SQLite-backed session persistence
//!
//! Revision History
//! - 2025-11-23T23:35:00Z @AI: Add config module for configuration management (Phase 5 Sprint 10 Task 5.6).
//! - 2025-11-23T21:27:00Z @AI: Add llm_parsers module (HEXSER compliance refactoring).
//! - 2025-11-18T11:22:00Z @AI: Introduce infrastructure module and declare SQLite session storage adapter.

pub mod config;
pub mod llm_parsers;
pub mod sqlite_session_storage;
