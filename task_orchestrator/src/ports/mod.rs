//! Ports (interfaces) for the task_orchestrator crate.
//!
//! This module declares abstract interfaces (Ports) used by the orchestrator's
//! graph nodes to request enhancements and comprehension tests from concrete
//! adapters. Traits here are async and object-safe for use behind Arc<dyn _>.
//!
//! Revision History
//! - 2025-12-03T00:00:00Z @AI: Add llm_agent_port for chain-of-thought chat agent implementation.
//! - 2025-11-30T19:45:00Z @AI: Add web_crawler_port for Phase 3 artifact generator.
//! - 2025-11-30T11:05:00Z @AI: Add vision_port for Phase 5 image processing implementation.
//! - 2025-11-28T19:10:00Z @AI: Add embedding_port for Phase 1 RAG implementation.
//! - 2025-11-24T01:05:00Z @AI: Add metrics_collector_port for Phase 5 Sprint 12 Task 5.11.
//! - 2025-11-23T17:00:00Z @AI: Add task_decomposition_port for Phase 3 Sprint 7.
//! - 2025-11-22T17:00:00Z @AI: Add prd_parser_port for Rigger Phase 0 Sprint 0.3.
//! - 2025-11-12T21:05:00Z @AI: Create ports module and declare submodules for Phase 3.

pub mod task_enhancement_port;
pub mod comprehension_test_port;
pub mod prd_parser_port;
pub mod task_decomposition_port;
pub mod metrics_collector_port;
pub mod embedding_port;
pub mod vision_port;
pub mod web_crawler_port;
pub mod llm_agent_port;
