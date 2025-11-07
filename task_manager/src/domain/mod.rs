//! Domain layer for task management.
//!
//! This module contains pure business entities for task management,
//! including Task (the main entity), TaskStatus (lifecycle states),
//! TaskRevision (audit history), ChecklistItem (sub-tasks), and
//! sorting/ordering utilities.
//!
//! Revision History
//! - 2025-11-06T19:16:00Z @AI: Initial domain module created from transcript_processor split.

pub mod task;
pub mod task_status;
pub mod task_revision;
pub mod checklist_item;
pub mod task_sort_key;
pub mod sort_order;
