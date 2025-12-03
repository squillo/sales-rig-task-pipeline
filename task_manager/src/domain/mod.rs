//! Domain layer for task management.
//!
//! This module contains pure business entities for task management,
//! including Task (the main entity), TaskStatus (lifecycle states),
//! TaskRevision (audit history), ChecklistItem (sub-tasks), and
//! sorting/ordering utilities.
//!
//! Revision History
//! - 2025-11-30T18:30:00Z @AI: Add scan_config module for artifact generator directory scanning configuration.
//! - 2025-11-28T19:00:00Z @AI: Add artifact module for RAG knowledge storage entity.
//! - 2025-11-26T07:10:00Z @AI: Add agent_tool and persona modules for Phase 1 persona management with agent tool configuration.
//! - 2025-11-24T05:00:00Z @AI: Add project module for Phase 1 TUI project architecture (top-level organizational entity).
//! - 2025-11-23T15:45:00Z @AI: Add services module for Phase 2 Sprint 5 domain intelligence services.
//! - 2025-11-22T16:20:00Z @AI: Add project_context module for Rigger Phase 0 codebase analysis.
//! - 2025-11-22T16:00:00Z @AI: Add prd module for Rigger Phase 0 PRD entity.
//! - 2025-11-12T20:28:00Z @AI: Declare enhancement and comprehension_test modules for Phase 1 orchestration support.
//! - 2025-11-06T19:16:00Z @AI: Initial domain module created from transcript_processor split.

pub mod task;
pub mod task_status;
pub mod task_revision;
pub mod checklist_item;
pub mod task_sort_key;
pub mod sort_order;
pub mod enhancement;
pub mod comprehension_test;
pub mod prd;
pub mod project;
pub mod project_context;
pub mod services;
pub mod agent_tool;
pub mod persona;
pub mod artifact;
pub mod scan_config;
