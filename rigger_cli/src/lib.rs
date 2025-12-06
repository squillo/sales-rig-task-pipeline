//! Rigger CLI library.
//!
//! This module exposes command implementations for testing.
//!
//! Revision History
//! - 2025-12-03T09:15:00Z @AI: Add ui module for hierarchical config editor (Phase 3 of CONFIG-MODERN-20251203).
//! - 2025-11-24T00:30:00Z @AI: Add ports, adapters, and services modules for hexagonal architecture.
//! - 2025-11-22T18:05:00Z @AI: Create lib.rs to expose commands for integration tests.

pub mod commands;
pub mod display;
pub mod ports;
pub mod adapters;
pub mod services;
pub mod constants;
pub mod ui;
