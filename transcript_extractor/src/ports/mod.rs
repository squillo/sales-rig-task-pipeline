//! Ports layer for transcript extraction.
//!
//! This module defines the port (interface) for transcript extraction,
//! which allows the application to remain agnostic of the specific
//! LLM provider or extraction technology being used.
//!
//! Revision History
//! - 2025-11-06T19:16:00Z @AI: Initial ports module created from transcript_processor split.

pub mod transcript_extractor_port;
