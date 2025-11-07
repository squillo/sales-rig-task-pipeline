//! Adapters layer: Infrastructure implementations of ports.
//!
//! This module contains all infrastructure-specific implementations that
//! fulfill the port contracts defined in the application layer. These adapters
//! connect the core domain logic to external systems and services.
//!
//! The adapters in this module include:
//! - OllamaTranscriptExtractorAdapter: Implements TranscriptExtractorPort using the Ollama LLM service
//! - CandleTranscriptExtractorAdapter: Implements TranscriptExtractorPort using the Candle ML framework
//! - InMemoryTaskAdapter: Implements TaskRepositoryPort using an in-memory data structure
//!
//! These adapters are the "driven" components in the Hexagonal Architecture,
//! implementing the abstract interfaces (ports) defined by the application layer.
//!
//! Revision History
//! - 2025-11-06T21:00:00Z @AI: Add CandleTranscriptExtractorAdapter for embedded ML inference.
//! - 2025-11-06T18:56:00Z @AI: Update adapter name to OllamaTranscriptExtractorAdapter for clarity.
//! - 2025-11-06T18:00:00Z @AI: Initial adapters module structure.

pub mod ollama_adapter;
pub mod candle_adapter;
pub mod in_memory_task_adapter;
