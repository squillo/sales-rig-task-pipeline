//! Ports (abstract interfaces) for rigger_cli.
//!
//! This module contains trait definitions that define the boundaries of our
//! application following hexagonal architecture. Ports allow us to swap
//! implementations without changing business logic.
//!
//! Revision History
//! - 2025-11-24T00:30:00Z @AI: Create ports module for clipboard operations.

pub mod clipboard_port;
