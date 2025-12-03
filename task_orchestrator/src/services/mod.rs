//! Services layer for artifact management and RAG operations.
//!
//! This module contains domain services that coordinate complex operations
//! involving multiple domain entities and ports. Services orchestrate the
//! business logic for artifact ingestion, chunking, embedding generation,
//! and retrieval.
//!
//! Revision History
//! - 2025-11-30T21:00:00Z @AI: Add artifact_generator_service for Phase 4 artifact generator.
//! - 2025-11-30T11:50:00Z @AI: Add vision_service for Phase 3 media processing implementation.
//! - 2025-11-28T20:15:00Z @AI: Create services module for Phase 3 RAG artifact management (Task 4.1).

pub mod artifact_service;
pub mod vision_service;
pub mod artifact_generator_service;
