//! Transcript processor library for extracting and managing action items.
//!
//! This crate implements a Hexagonal Architecture (Ports and Adapters) for
//! processing meeting transcripts using LLM-powered extraction. It converts
//! unstructured transcript text into structured, trackable tasks with full
//! lifecycle management and audit history.
//!
//! The architecture is organized into three main layers:
//! - Domain: Pure business logic and entities (infrastructure-agnostic)
//! - Application: Use cases and port definitions (orchestration layer)
//! - Adapters: Infrastructure implementations (LLM clients, storage, etc.)
//!
//! Revision History
//! - 2025-11-06T18:00:00Z @AI: Added adapters layer (Ollama and in-memory implementations).
//! - 2025-11-06T17:41:00Z @AI: Added application layer (ports and use cases).
//! - 2025-11-06T17:41:00Z @AI: Initial library structure with domain layer.

pub mod domain;
pub mod application;
pub mod adapters;
