//! Domain services for intelligent task analysis.
//!
//! This module provides domain services that implement business logic for
//! task complexity analysis, dependency management, and triage classification.
//! These services are stateless and operate on Task entities.
//!
//! Revision History
//! - 2025-11-23T15:35:00Z @AI: Create services module for Phase 2 Sprint 5.

pub mod complexity_scorer;
pub mod dependency_graph;
pub mod triage_service;
