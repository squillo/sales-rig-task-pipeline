//! Task manager library for task persistence and lifecycle management.
//!
//! This crate provides comprehensive task management capabilities including
//! persistence, status tracking, querying with filters and sorting, and
//! revision history. It is designed as an independent, focused library
//! following hexagonal architecture patterns with the HEXSER framework.
//!
//! The architecture consists of four main layers:
//! - Domain: Pure business entities (Task, TaskStatus, TaskRevision, ChecklistItem)
//! - Ports: Interface definitions (TaskRepositoryPort)
//! - Use Cases: Business logic orchestration (ManageTaskUseCase)
//! - Adapters: Infrastructure implementations (InMemoryTaskAdapter)
//!
//! Revision History
//! - 2025-11-06T19:16:00Z @AI: Initial library created from transcript_processor split.

pub mod domain;
pub mod ports;
pub mod use_cases;
pub mod adapters;
