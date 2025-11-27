//! Graph-flow integration skeleton for rs-graph-llm (unified).
//!
//! This module documents the intended API surface for integrating a concrete
//! graph runtime. The project unifies the `graph_flow` runtime by default, so
//! this module and its helpers are always available. In later steps, the
//! placeholders here will be replaced with concrete StateGraph wiring.
//!
//! Revision History
//! - 2025-11-13T00:20:00Z @AI: Update docs to reflect unified graph_flow (no feature gate) and clarify purpose.
//! - 2025-11-12T17:46:00Z @AI: Add FlowIntegrationInfo::new_flow_builder behind graph_flow feature.
//! - 2025-11-12T16:32:00Z @AI: Create feature-gated flow_integration skeleton with docs and unit test stub.

/// Placeholder information type for the future graph-flow integration.
///
/// This struct exists to assert the intended API shape while the concrete
/// runtime wiring is implemented progressively. A later step will replace
/// usages with concrete types from rs-graph-llm (aka graph-flow).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FlowIntegrationInfo {
    /// Human-readable description of the API shape we intend to wire.
    desc: String,
}

impl FlowIntegrationInfo {
    /// Constructs a new FlowIntegrationInfo instance with a static description.
    pub fn new() -> Self {
        FlowIntegrationInfo { desc: std::string::String::from("StateGraph<GraphState> with conditional edges and persistence") }
    }

    /// Returns the descriptive API shape string.
    pub fn api_shape(&self) -> &str { self.desc.as_str() }

    /// Creates a new graph_flow::GraphBuilder with the given name.
    pub fn new_flow_builder(name: &str) -> graph_flow::GraphBuilder {
        graph_flow::GraphBuilder::new(name)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_info_api_shape_contains_keywords() {
        let info = super::FlowIntegrationInfo::new();
        let s = super::FlowIntegrationInfo::api_shape(&info);
        std::assert!(s.contains("StateGraph"));
        std::assert!(s.contains("GraphState"));
    }
}
