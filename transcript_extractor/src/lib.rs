//! Transcript extractor library for LLM-powered action item extraction.
//!
//! This crate provides LLM-powered extraction of structured action items from
//! unstructured meeting transcripts. It is designed as an independent, focused
//! library following hexagonal architecture patterns with the HEXSER framework.
//!
//! The architecture consists of three main layers:
//! - Domain: Pure business entities (ActionItem, TranscriptAnalysis)
//! - Ports: Interface definitions (TranscriptExtractorPort)
//! - Adapters: Infrastructure implementations (OllamaTranscriptExtractorAdapter)
//!
//! Revision History
//! - 2025-11-06T19:16:00Z @AI: Initial library created from transcript_processor split.

pub mod domain;
pub mod ports;
pub mod adapters;
