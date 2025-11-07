//! Use case implementations for application workflows.
//!
//! This module contains the use cases that orchestrate the application's business
//! workflows. Use cases coordinate domain logic with infrastructure adapters through
//! the port interfaces, implementing the application's core features while remaining
//! independent of external implementation details.
//!
//! Revision History
//! - 2025-11-06T17:41:00Z @AI: Initial use cases module structure created.

pub mod process_transcript;
pub mod manage_task;
