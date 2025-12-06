//! Rigger Core - Shared configuration and types for Rigger.
//!
//! This crate provides the canonical configuration structure for Rigger v3.0,
//! including support for multiple LLM providers, API key management, task slots,
//! and automatic migration from legacy config formats.
//!
//! Revision History
//! - 2025-12-03T07:45:00Z @AI: Initial rigger_core crate for unified configuration system (Phase 2 of CONFIG-MODERN-20251203).

pub mod config;

pub use config::RiggerConfig;
