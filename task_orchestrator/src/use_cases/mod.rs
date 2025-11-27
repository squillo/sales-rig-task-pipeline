//! Application layer (use cases) for orchestrating task flows.
//!
//! This module hosts application-level orchestration that wires together the
//! Brain (graph/state/nodes) and Muscle (ports/adapters). In Phase 6 this will
//! delegate to a concrete graph runtime. For now, we provide a minimal runner
//! that executes nodes sequentially to keep the system verifiable.
//!
//! Revision History
//! - 2025-11-14T15:44:00Z @AI: Export FlowRunner module to back run_task_with_flow.
//! - 2025-11-13T08:31:00Z @AI: Export run_task_with_ports helper to enable DI of ports.
//! - 2025-11-13T21:45:00Z @AI: Export Orchestrator facade; unify API for running flows.
//! - 2025-11-13T00:33:00Z @AI: Export run_task_with_flow helper (default no-feature variant returns explanatory error).
//! - 2025-11-12T23:28:00Z @AI: Export run_task_with_ollama helper alongside TaskGraphRunner.
//! - 2025-11-12T22:20:00Z @AI: Create use_cases module and declare task_graph_runner (Phase 7 scaffold).

pub mod task_graph_runner;
pub mod run_task_with_ollama;
pub mod run_task_with_flow;
pub mod orchestrator;
pub mod run_task_with_ports;
pub mod flow_runner;
