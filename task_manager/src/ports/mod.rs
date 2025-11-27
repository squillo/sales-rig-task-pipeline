//! Ports layer for task management.
//!
//! This module defines the port (interface) for task persistence and querying,
//! following HEXSER Repository patterns with filters and sort keys.
//!
//! Revision History
//! - 2025-11-26T07:25:00Z @AI: Add agent_tool_repository_port and persona_repository_port for Phase 2 persona management.
//! - 2025-11-24T05:00:00Z @AI: Add project_repository_port for Phase 1 TUI project architecture.
//! - 2025-11-06T19:16:00Z @AI: Initial ports module created from transcript_processor split.

pub mod task_repository_port;
pub mod project_repository_port;
pub mod agent_tool_repository_port;
pub mod persona_repository_port;
