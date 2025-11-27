//! Task manager library for task persistence and lifecycle management.
//!
//! This crate provides comprehensive task management capabilities including
//! persistence, status tracking, querying with filters and sorting, and
//! revision history. It is designed as an independent, focused library
//! following hexagonal architecture patterns with the HEXSER framework.
//!
//! The architecture consists of five hexagonal layers (DPAI):
//! - Domain: Pure business entities (Task, TaskStatus, PRD, ProjectContext, etc.)
//! - Ports: Interface definitions (TaskRepositoryPort)
//! - Adapters: Port implementations (InMemoryTaskAdapter, SqliteTaskAdapter)
//! - Use Cases: Business logic orchestration (ManageTaskUseCase)
//! - Infrastructure: External system integrations (parsers, schemas, DTOs)
//!
//! # Rigger Integration
//!
//! This crate includes Rigger-compatible entities for AI-driven project management:
//! - PRD: Product Requirements Document parsing and storage
//! - ProjectContext: Codebase analysis and synthesis
//! - Task extensions: Support for task hierarchies and PRD linkage
//!
//! Revision History
//! - 2025-11-23T21:30:00Z @AI: Replace utils with infrastructure module (HEXSER compliance refactoring).
//! - 2025-11-22T16:25:00Z @AI: Add Rigger entities (PRD, ProjectContext) and update documentation for Phase 0.
//! - 2025-11-08T08:39:00Z @AI: Expose utils module with tolerant parser for shared use across crates.
//! - 2025-11-06T19:16:00Z @AI: Initial library created from transcript_processor split.

pub mod domain;
pub mod ports;
pub mod adapters;
pub mod use_cases;
pub mod infrastructure;
