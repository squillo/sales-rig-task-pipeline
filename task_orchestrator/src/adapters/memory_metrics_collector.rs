//! In-memory metrics collector adapter.
//!
//! Stores performance metrics in memory using Arc<RwLock<Vec<T>>>. Suitable for
//! runtime benchmarking and short-lived sessions. Metrics are lost when the
//! process terminates (use database adapter for persistence).
//!
//! Revision History
//! - 2025-11-24T01:10:00Z @AI: Create in-memory metrics collector for Phase 5 Sprint 12 Task 5.11.

/// In-memory metrics collector adapter.
///
/// Stores metrics in a thread-safe vector protected by RwLock. Provides fast
/// access for runtime benchmarking without persistence overhead.
///
/// # Examples
///
/// ```
/// use task_orchestrator::adapters::memory_metrics_collector::MemoryMetricsCollector;
/// use task_orchestrator::ports::metrics_collector_port::MetricsCollectorPort;
/// use task_orchestrator::domain::performance_metrics::InferenceMetrics;
///
/// #[tokio::main]
/// async fn main() {
///     let collector = MemoryMetricsCollector::new();
///
///     let metrics = InferenceMetrics::new(
///         "enhancement".to_string(),
///         "ollama".to_string(),
///         "llama3.1".to_string(),
///         std::option::Option::None,
///     );
///
///     collector.record_metric(metrics).await.unwrap();
///
///     let all_metrics = collector.get_all_metrics().await.unwrap();
///     std::assert_eq!(all_metrics.len(), 1);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MemoryMetricsCollector {
    metrics: std::sync::Arc<tokio::sync::RwLock<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>>>,
}

impl MemoryMetricsCollector {
    /// Creates a new empty in-memory metrics collector.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::adapters::memory_metrics_collector::MemoryMetricsCollector;
    ///
    /// let collector = MemoryMetricsCollector::new();
    /// ```
    pub fn new() -> Self {
        Self {
            metrics: std::sync::Arc::new(tokio::sync::RwLock::new(std::vec::Vec::new())),
        }
    }

    /// Returns the current number of stored metrics.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::adapters::memory_metrics_collector::MemoryMetricsCollector;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let collector = MemoryMetricsCollector::new();
    ///     std::assert_eq!(collector.count().await, 0);
    /// }
    /// ```
    pub async fn count(&self) -> usize {
        let metrics = self.metrics.read().await;
        metrics.len()
    }
}

impl std::default::Default for MemoryMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl crate::ports::metrics_collector_port::MetricsCollectorPort for MemoryMetricsCollector {
    async fn record_metric(
        &self,
        metric: crate::domain::performance_metrics::InferenceMetrics,
    ) -> std::result::Result<(), String> {
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        std::result::Result::Ok(())
    }

    async fn get_all_metrics(
        &self,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String> {
        let metrics = self.metrics.read().await;
        std::result::Result::Ok(metrics.clone())
    }

    async fn get_metrics_by_provider(
        &self,
        provider: &str,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String> {
        let metrics = self.metrics.read().await;
        let filtered: std::vec::Vec<_> = metrics
            .iter()
            .filter(|m| m.provider == provider)
            .cloned()
            .collect();
        std::result::Result::Ok(filtered)
    }

    async fn get_metrics_by_operation(
        &self,
        operation_type: &str,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String> {
        let metrics = self.metrics.read().await;
        let filtered: std::vec::Vec<_> = metrics
            .iter()
            .filter(|m| m.operation_type == operation_type)
            .cloned()
            .collect();
        std::result::Result::Ok(filtered)
    }

    async fn get_metrics_by_role(
        &self,
        role: crate::domain::model_role::ModelRole,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String> {
        let metrics = self.metrics.read().await;
        let filtered: std::vec::Vec<_> = metrics
            .iter()
            .filter(|m| m.role == std::option::Option::Some(role))
            .cloned()
            .collect();
        std::result::Result::Ok(filtered)
    }

    async fn clear_metrics(&self) -> std::result::Result<(), String> {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
        std::result::Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::metrics_collector_port::MetricsCollectorPort;

    #[tokio::test]
    async fn test_memory_collector_creation() {
        // Test: Validates MemoryMetricsCollector instantiation.
        // Justification: Ensures the collector starts empty.
        let collector = MemoryMetricsCollector::new();
        std::assert_eq!(collector.count().await, 0);
    }

    #[tokio::test]
    async fn test_record_and_retrieve_metric() {
        // Test: Validates recording and retrieval of a single metric.
        // Justification: Core functionality for metrics tracking.
        let collector = MemoryMetricsCollector::new();

        let metrics = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );

        collector.record_metric(metrics.clone()).await.unwrap();

        let all_metrics = collector.get_all_metrics().await.unwrap();
        std::assert_eq!(all_metrics.len(), 1);
        std::assert_eq!(all_metrics[0].operation_type, "enhancement");
        std::assert_eq!(all_metrics[0].provider, "ollama");
    }

    #[tokio::test]
    async fn test_filter_by_provider() {
        // Test: Validates provider-based filtering.
        // Justification: Enables comparison between Ollama and MLX.
        let collector = MemoryMetricsCollector::new();

        let ollama_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );

        let mlx_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "mlx".to_string(),
            "phi3".to_string(),
            std::option::Option::None,
        );

        collector.record_metric(ollama_metric).await.unwrap();
        collector.record_metric(mlx_metric).await.unwrap();

        let ollama_metrics = collector.get_metrics_by_provider("ollama").await.unwrap();
        std::assert_eq!(ollama_metrics.len(), 1);
        std::assert_eq!(ollama_metrics[0].provider, "ollama");

        let mlx_metrics = collector.get_metrics_by_provider("mlx").await.unwrap();
        std::assert_eq!(mlx_metrics.len(), 1);
        std::assert_eq!(mlx_metrics[0].provider, "mlx");
    }

    #[tokio::test]
    async fn test_filter_by_operation() {
        // Test: Validates operation type filtering.
        // Justification: Enables analysis of specific operation performance.
        let collector = MemoryMetricsCollector::new();

        let enhancement_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );

        let decomposition_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "decomposition".to_string(),
            "ollama".to_string(),
            "orca2".to_string(),
            std::option::Option::None,
        );

        collector.record_metric(enhancement_metric).await.unwrap();
        collector.record_metric(decomposition_metric).await.unwrap();

        let enhancement_metrics = collector.get_metrics_by_operation("enhancement").await.unwrap();
        std::assert_eq!(enhancement_metrics.len(), 1);
        std::assert_eq!(enhancement_metrics[0].operation_type, "enhancement");
    }

    #[tokio::test]
    async fn test_filter_by_role() {
        // Test: Validates role-based filtering for heterogeneous pipeline.
        // Justification: Enables comparison of Router vs Decomposer performance.
        let collector = MemoryMetricsCollector::new();

        let router_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "phi3".to_string(),
            std::option::Option::Some(crate::domain::model_role::ModelRole::Router),
        );

        let decomposer_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "decomposition".to_string(),
            "ollama".to_string(),
            "orca2".to_string(),
            std::option::Option::Some(crate::domain::model_role::ModelRole::Decomposer),
        );

        collector.record_metric(router_metric).await.unwrap();
        collector.record_metric(decomposer_metric).await.unwrap();

        let router_metrics = collector
            .get_metrics_by_role(crate::domain::model_role::ModelRole::Router)
            .await
            .unwrap();
        std::assert_eq!(router_metrics.len(), 1);
        std::assert_eq!(router_metrics[0].role, std::option::Option::Some(crate::domain::model_role::ModelRole::Router));
    }

    #[tokio::test]
    async fn test_clear_metrics() {
        // Test: Validates clearing of all metrics.
        // Justification: Required for resetting between benchmark runs.
        let collector = MemoryMetricsCollector::new();

        let metrics = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );

        collector.record_metric(metrics).await.unwrap();
        std::assert_eq!(collector.count().await, 1);

        collector.clear_metrics().await.unwrap();
        std::assert_eq!(collector.count().await, 0);
    }

    #[tokio::test]
    async fn test_concurrent_writes() {
        // Test: Validates thread-safe concurrent metric recording.
        // Justification: Metrics may be recorded from multiple async tasks.
        let collector = std::sync::Arc::new(MemoryMetricsCollector::new());

        let handles: std::vec::Vec<_> = (0..10)
            .map(|i| {
                let collector = collector.clone();
                tokio::spawn(async move {
                    let metrics = crate::domain::performance_metrics::InferenceMetrics::new(
                        std::format!("operation_{}", i),
                        "ollama".to_string(),
                        "llama3.1".to_string(),
                        std::option::Option::None,
                    );
                    collector.record_metric(metrics).await.unwrap();
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        std::assert_eq!(collector.count().await, 10);
    }
}
