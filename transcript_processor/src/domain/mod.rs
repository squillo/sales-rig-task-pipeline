//! Domain layer module declarations for the transcript processor.
//!
//! This module contains the core domain entities and value objects that represent
//! the business logic of the transcript processing system. All domain types are
//! infrastructure-agnostic and follow hexagonal architecture principles.
//!
//! Revision History
//! - 2025-11-06T17:41:00Z @AI: Initial domain module structure created.

pub mod action_item;
pub mod checklist_item;
pub mod transcript_analysis;
pub mod task_status;
pub mod task;
pub mod task_revision;
pub mod task_sort_key;
pub mod sort_order;
