//! build_graph_flow helper for unified graph-flow integration.
//!
//! This module provides a minimal helper function that constructs a
//! `graph_flow::GraphBuilder` by name. The graph-flow runtime is unified in
//! this project (no feature gate) and will be used to assemble a concrete
//! state graph over `crate::graph::state::GraphState` by wiring nodes and
//! conditional edges in later steps.
//!
//! Revision History
//! - 2025-11-13T21:22:00Z @AI: Update docs to reflect unified graph_flow (no feature gate).
//! - 2025-11-12T23:55:00Z @AI: Introduce feature-gated build_graph_flow(name) -> graph_flow::GraphBuilder.

/// Returns a new graph_flow::GraphBuilder with the given name.
///
/// This function returns a builder to begin constructing a flow using the
/// graph runtime without forcing callers to depend on specific node wiring.
pub fn build_graph_flow(name: &str) -> graph_flow::GraphBuilder {
    graph_flow::GraphBuilder::new(name)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_build_graph_flow_constructs_builder() {
        let _b = super::build_graph_flow("orchestrator");
        // Intentionally minimal: ensure constructor path compiles.
    }
}
