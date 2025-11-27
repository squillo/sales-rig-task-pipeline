//! LLM response parsers for task_orchestrator infrastructure layer.
//!
//! This module contains parsers that handle noisy JSON responses from LLM providers,
//! applying tolerant parsing strategies for comprehension tests and other entities.
//!
//! Revision History
//! - 2025-11-23T21:26:00Z @AI: Create llm_parsers module (HEXSER compliance).

pub mod comprehension_test_parser;
