//! Port for collecting and storing performance metrics.
//!
//! Defines the interface for metrics collection, allowing different storage
//! backends (in-memory, database, file) to be swapped without changing the
//! domain or application layers.
//!
//! Revision History
//! - 2025-11-24T01:00:00Z @AI: Create MetricsCollectorPort for Phase 5 Sprint 12 Task 5.11.

/// Port for collecting and storing performance metrics.
///
/// This trait defines the interface for recording inference metrics and
/// retrieving them for analysis and benchmarking. Implementations can store
/// metrics in memory, database, or files.
///
/// # Examples
///
/// ```
/// use task_orchestrator::ports::metrics_collector_port::MetricsCollectorPort;
/// use task_orchestrator::domain::performance_metrics::InferenceMetrics;
///
/// async fn record_metric(collector: &dyn MetricsCollectorPort) {
///     let metrics = InferenceMetrics::new(
///         "enhancement".to_string(),
///         "ollama".to_string(),
///         "llama3.1".to_string(),
///         std::option::Option::None,
///     );
///     collector.record_metric(metrics).await.unwrap();
/// }
/// ```
#[async_trait::async_trait]
pub trait MetricsCollectorPort: std::marker::Send + std::marker::Sync {
    /// Records a single inference metric.
    ///
    /// # Arguments
    ///
    /// * `metric` - The inference metric to record
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails.
    async fn record_metric(
        &self,
        metric: crate::domain::performance_metrics::InferenceMetrics,
    ) -> std::result::Result<(), String>;

    /// Retrieves all recorded metrics.
    ///
    /// # Returns
    ///
    /// Vector of all inference metrics, or error if retrieval fails.
    async fn get_all_metrics(
        &self,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String>;

    /// Retrieves metrics filtered by provider.
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name to filter by (e.g., "ollama", "mlx")
    ///
    /// # Returns
    ///
    /// Vector of matching metrics, or error if retrieval fails.
    async fn get_metrics_by_provider(
        &self,
        provider: &str,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String>;

    /// Retrieves metrics filtered by operation type.
    ///
    /// # Arguments
    ///
    /// * `operation_type` - Operation type to filter by (e.g., "enhancement", "decomposition")
    ///
    /// # Returns
    ///
    /// Vector of matching metrics, or error if retrieval fails.
    async fn get_metrics_by_operation(
        &self,
        operation_type: &str,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String>;

    /// Retrieves metrics filtered by model role.
    ///
    /// # Arguments
    ///
    /// * `role` - Model role to filter by
    ///
    /// # Returns
    ///
    /// Vector of matching metrics, or error if retrieval fails.
    async fn get_metrics_by_role(
        &self,
        role: crate::domain::model_role::ModelRole,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String>;

    /// Clears all stored metrics.
    ///
    /// Useful for resetting between benchmark runs.
    ///
    /// # Errors
    ///
    /// Returns error if clear operation fails.
    async fn clear_metrics(&self) -> std::result::Result<(), String>;
}
