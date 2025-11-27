//! LLM response parsers for infrastructure layer.
//!
//! This module contains parsers that handle noisy JSON responses from LLM providers,
//! applying tolerant parsing strategies with alias mapping and field normalization.
//!
//! Revision History
//! - 2025-11-23T21:21:00Z @AI: Create llm_parsers module (HEXSER compliance).

pub mod action_item_parser;
