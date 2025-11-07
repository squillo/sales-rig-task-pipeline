//! Port trait definitions for external adapter interfaces.
//!
//! This module defines the ports (interfaces) that external adapters must implement
//! to interact with the application layer. Ports represent the boundaries of the
//! application and enable the Hexagonal Architecture's dependency inversion principle.
//! The core application depends on these abstract ports, not on concrete implementations.
//!
//! Revision History
//! - 2025-11-06T17:41:00Z @AI: Initial ports module structure created.

pub mod transcript_extractor_port;
pub mod task_repository_port;
