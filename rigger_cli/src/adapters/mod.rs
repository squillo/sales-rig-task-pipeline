//! Adapters (concrete implementations) for rigger_cli.
//!
//! This module contains concrete adapter implementations that bridge our
//! application to external systems (clipboard, file system, network, etc.)
//! following hexagonal architecture principles.
//!
//! Revision History
//! - 2025-11-24T00:30:00Z @AI: Create adapters module for clipboard operations.

pub mod arboard_clipboard_adapter;
