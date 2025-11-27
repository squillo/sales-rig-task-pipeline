//! Domain layer for task orchestration business logic.
//!
//! Contains core domain entities and services for the orchestration pipeline,
//! including model selection strategies for the heterogeneous agent architecture
//! and performance metrics for benchmarking LLM operations.
//!
//! Revision History
//! - 2025-11-24T00:50:00Z @AI: Add performance_metrics module for Phase 5 Sprint 12 Task 5.10.
//! - 2025-11-23T22:05:00Z @AI: Add domain module with model_role for heterogeneous pipeline (Phase 5 Sprint 10 Task 5.1).

pub mod model_role;
pub mod performance_metrics;
