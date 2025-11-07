//! Domain layer for transcript extraction.
//!
//! This module contains pure business entities for transcript analysis,
//! including ActionItem (the DTO extracted from transcripts) and
//! TranscriptAnalysis (the aggregate result of extraction).
//!
//! Revision History
//! - 2025-11-06T19:16:00Z @AI: Initial domain module created from transcript_processor split.

pub mod action_item;
pub mod transcript_analysis;
