//! Application layer module declarations for use cases and ports.
//!
//! This module contains the application orchestration logic, defining the ports
//! (interfaces) that the domain depends on and the use cases that coordinate
//! domain logic with external adapters. The application layer sits between the
//! pure domain logic and the infrastructure adapters in the Hexagonal Architecture.
//!
//! Revision History
//! - 2025-11-06T17:41:00Z @AI: Initial application layer structure created.

pub mod ports;
pub mod use_cases;
