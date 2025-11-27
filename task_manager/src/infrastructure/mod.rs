//! Infrastructure layer for task_manager crate.
//!
//! This module contains components that integrate with external systems and handle
//! boundary concerns such as parsing external data formats, generating schemas for
//! external APIs, and defining DTOs for crossing architectural boundaries.
//!
//! Modules:
//! - `llm_parsers`: Parse LLM responses into structured DTOs
//! - `markdown_parsers`: Parse markdown documents into domain entities
//! - `schemas`: Generate JSON schemas for external API configuration
//! - `dtos`: Data Transfer Objects for boundary crossing
//!
//! Revision History
//! - 2025-11-23T21:20:00Z @AI: Create infrastructure layer (HEXSER compliance refactoring).

pub mod llm_parsers;
pub mod markdown_parsers;
pub mod schemas;
pub mod dtos;
