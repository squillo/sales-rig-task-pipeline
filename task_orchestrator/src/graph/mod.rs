//! Graph module: state and (future) nodes for orchestrator.
//!
//! This module re-exports the graph components. Currently it provides the
//! GraphState type and placeholder node implementations. The build_graph
//! function remains a stub until graph framework dependencies are finalized
//! (Phase 6).
//!
//! Revision History
//! - 2025-11-15T10:36:00Z @AI: Declare assemble_orchestrator_flow module for graph assembly wiring.
//! - 2025-11-13T21:06:00Z @AI: Unify features; expose flow_integration and build_graph_flow unconditionally.
//! - 2025-11-13T09:32:00Z @AI: Declare flow_shims module for task shims delegating to nodes.
//! - 2025-11-12T23:50:00Z @AI: Declare feature-gated build_graph_flow module for graph_flow integration.
//! - 2025-11-12T16:32:00Z @AI: Add feature-gated flow_integration module declaration.
//! - 2025-11-12T21:55:00Z @AI: Re-export nodes and add build_graph module stub.
//! - 2025-11-12T21:49:00Z @AI: Create graph module and re-export state (Phase 5 bootstrap).

pub mod state;
pub mod nodes;
pub mod build_graph;
pub mod orchestrator_graph;

// Unified features: graph-flow integration modules are always built
pub mod flow_integration;
pub mod build_graph_flow;
pub mod flow_shims;
pub mod assemble_orchestrator_flow;
