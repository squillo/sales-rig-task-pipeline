//! Adapters (concrete implementations of orchestrator ports).
//!
//! This module contains concrete adapters that implement the orchestrator ports
//! to interact with external systems (e.g., LLMs). These are the "Muscle" of the
//! architecture and are used by the graph nodes to perform work.
//!
//! Revision History
//! - 2025-12-03T00:00:00Z @AI: Add rig_agent_adapter for chain-of-thought chat agent implementation.
//! - 2025-11-30T20:15:00Z @AI: Add reqwest_web_crawler for Phase 3 artifact generator.
//! - 2025-11-30T11:20:00Z @AI: Add rig_vision_adapter for Phase 5 image processing implementation.
//! - 2025-11-28T19:45:00Z @AI: Add rig_embedding_adapter for Phase 3 RAG embedding generation.
//! - 2025-11-24T01:35:00Z @AI: Add sqlite_metrics_collector for Phase 5 Sprint 12 Task 5.12.
//! - 2025-11-24T01:15:00Z @AI: Add memory_metrics_collector for Phase 5 Sprint 12 Task 5.11.
//! - 2025-11-24T00:10:00Z @AI: Add mlx_subprocess_adapter for macOS optimization (Phase 5 Sprint 11 Task 5.8).
//! - 2025-11-23 @AI: Add provider_factory for vendor-agnostic LLM provider swapping (Phase 1 Sprint 3 Task 1.9).
//! - 2025-11-23T17:15:00Z @AI: Add rig_task_decomposition_adapter for Phase 3 Sprint 7.
//! - 2025-11-22T17:05:00Z @AI: Add rig_prd_parser_adapter for Rigger Phase 0 Sprint 0.3.
//! - 2025-11-12T21:28:00Z @AI: Create adapters module and declare submodules for Phase 4.

pub mod ollama_enhancement_adapter;
pub mod ollama_comprehension_test_adapter;
pub mod rig_prd_parser_adapter;
pub mod rig_task_decomposition_adapter;
pub mod provider_factory;
pub mod mlx_subprocess_adapter;
pub mod memory_metrics_collector;
pub mod sqlite_metrics_collector;
pub mod rig_embedding_adapter;
pub mod rig_vision_adapter;
pub mod reqwest_web_crawler;
pub mod rig_agent_adapter;
