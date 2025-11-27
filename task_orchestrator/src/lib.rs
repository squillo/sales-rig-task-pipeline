//! Task Orchestrator crate root: graph-based coordination of task processing.
//!
//! This crate will host the Brain (graph/state/nodes) and Muscle (ports/adapters)
//! for orchestrating task enhancements and comprehension tests. It integrates
//! schema-enforced extraction via rig-core and, later, a stateful graph runtime.
//!
//! Revision History
//! - 2025-11-23T22:05:00Z @AI: Export domain module for ModelRole and heterogeneous pipeline (Phase 5 Sprint 10).
//! - 2025-11-23T21:32:00Z @AI: Remove utils export; all moved to infrastructure (HEXSER compliance).
//! - 2025-11-23 @AI: Export tools module for FileSystemTool and Rig agent capabilities (Phase 4 Sprint 9 Task 4.7).
//! - 2025-11-23 @AI: Export utils module for tolerant parsing helpers (Phase 1 Sprint 3 Task 1.8).
//! - 2025-11-18T11:22:45Z @AI: Export infrastructure module for SQLite session storage adapter.
//! - 2025-11-12T17:20:00Z @AI: Export architecture module for Hexser visibility.
//! - 2025-11-12T22:24:00Z @AI: Export use_cases module and add TaskGraphRunner scaffold.
//! - 2025-11-12T21:36:00Z @AI: Export adapters module; extend crate for Phase 4.
//! - 2025-11-12T21:15:00Z @AI: Export ports module; keep minimal API for compilation.
//! - 2025-11-12T20:51:00Z @AI: Initialize crate root with minimal API and documentation.

pub mod domain;
pub mod ports;
pub mod adapters;
pub mod graph;
pub mod use_cases;
pub mod architecture;
pub mod infrastructure;
pub mod tools;

/// Returns the crate semantic version at compile time.
///
/// This helper is primarily for smoke tests and basic introspection and keeps the
/// crate compilable while the full orchestrator is implemented over multiple phases.
///
/// # Examples
///
/// ```
/// let v = task_orchestrator::crate_version();
/// assert!(v.len() > 0);
/// ```
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_version_non_empty() {
        let v = super::crate_version();
        std::assert!(!v.is_empty());
    }
}
