//! Task shims bridging graph runtime to existing nodes (module declarations).
//!
//! This module declares shim types that delegate to the Phase 5 nodes' execute()
//! methods. Each shim lives in its own file to comply with the one-item-per-file
//! standard. They provide a stable API surface for future graph runtime wiring.
//!
//! Revision History
//! - 2025-11-23T17:45:00Z @AI: Add task_decomposition_task_shim for Phase 3 Sprint 7.
//! - 2025-11-15T10:31:30Z @AI: Add end_task module declaration for terminal node.
//! - 2025-11-13T09:32:00Z @AI: Create flow_shims module declarations for four task shims.

pub mod semantic_router_task_shim;
pub mod enhancement_task_shim;
pub mod comprehension_test_task_shim;
pub mod check_test_result_task_shim;
pub mod task_decomposition_task_shim;
pub mod end_task;
