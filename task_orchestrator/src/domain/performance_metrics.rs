//! Performance metrics domain entity for benchmarking LLM operations.
//!
//! This module defines the core domain entity for tracking inference performance
//! across different LLM providers (Ollama, MLX, etc.) and model roles (Router,
//! Decomposer, etc.). Metrics enable performance comparison and optimization of
//! the heterogeneous agent pipeline.
//!
//! Revision History
//! - 2025-11-24T00:45:00Z @AI: Create performance metrics domain entity for Phase 5 Sprint 12 Task 5.10.

/// Performance metrics for a single LLM inference operation.
///
/// Captures timing, throughput, and context information for benchmarking
/// and comparing different LLM backends and models.
///
/// # Examples
///
/// ```
/// use task_orchestrator::domain::performance_metrics::InferenceMetrics;
/// use task_orchestrator::domain::model_role::ModelRole;
///
/// let metrics = InferenceMetrics::new(
///     "enhancement".to_string(),
///     "ollama".to_string(),
///     "llama3.1".to_string(),
///     std::option::Option::Some(ModelRole::Enhancer),
/// );
/// // Later, after inference completes:
/// // metrics.record_completion(duration, input_tokens, output_tokens);
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InferenceMetrics {
    /// Unique identifier for this metrics record.
    pub id: String,

    /// Timestamp when the operation started (UTC).
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Type of operation (e.g., "enhancement", "decomposition", "comprehension_test").
    pub operation_type: String,

    /// Provider/backend used (e.g., "ollama", "mlx", "openai").
    pub provider: String,

    /// Model identifier (e.g., "llama3.1", "mlx-community/Phi-3-mini-4k-instruct").
    pub model: String,

    /// Optional role in heterogeneous pipeline (e.g., Router, Decomposer).
    pub role: std::option::Option<crate::domain::model_role::ModelRole>,

    /// Inference duration in milliseconds.
    pub duration_ms: std::option::Option<u64>,

    /// Number of input tokens (prompt).
    pub input_tokens: std::option::Option<usize>,

    /// Number of output tokens (generated).
    pub output_tokens: std::option::Option<usize>,

    /// Tokens per second (calculated from duration and output_tokens).
    pub tokens_per_second: std::option::Option<f64>,

    /// Success status of the operation.
    pub success: bool,

    /// Optional error message if operation failed.
    pub error: std::option::Option<String>,
}

impl InferenceMetrics {
    /// Creates a new InferenceMetrics record for an operation that is about to start.
    ///
    /// # Arguments
    ///
    /// * `operation_type` - Type of operation (e.g., "enhancement", "decomposition")
    /// * `provider` - Provider/backend name (e.g., "ollama", "mlx")
    /// * `model` - Model identifier
    /// * `role` - Optional role in heterogeneous pipeline
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::performance_metrics::InferenceMetrics;
    /// use task_orchestrator::domain::model_role::ModelRole;
    ///
    /// let metrics = InferenceMetrics::new(
    ///     "enhancement".to_string(),
    ///     "mlx".to_string(),
    ///     "mlx-community/Phi-3-mini-4k-instruct".to_string(),
    ///     std::option::Option::Some(ModelRole::Router),
    /// );
    /// std::assert_eq!(metrics.provider, "mlx");
    /// std::assert!(metrics.duration_ms.is_none()); // Not yet completed
    /// ```
    pub fn new(
        operation_type: String,
        provider: String,
        model: String,
        role: std::option::Option<crate::domain::model_role::ModelRole>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            operation_type,
            provider,
            model,
            role,
            duration_ms: std::option::Option::None,
            input_tokens: std::option::Option::None,
            output_tokens: std::option::Option::None,
            tokens_per_second: std::option::Option::None,
            success: false,
            error: std::option::Option::None,
        }
    }

    /// Records successful completion of the operation with timing and token counts.
    ///
    /// Automatically calculates tokens per second from duration and output tokens.
    ///
    /// # Arguments
    ///
    /// * `duration` - Time taken for the operation
    /// * `input_tokens` - Number of input tokens (prompt)
    /// * `output_tokens` - Number of generated tokens
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::performance_metrics::InferenceMetrics;
    /// use std::time::Duration;
    ///
    /// let mut metrics = InferenceMetrics::new(
    ///     "enhancement".to_string(),
    ///     "ollama".to_string(),
    ///     "llama3.1".to_string(),
    ///     std::option::Option::None,
    /// );
    ///
    /// metrics.record_completion(
    ///     std::time::Duration::from_millis(2000),
    ///     100,
    ///     200,
    /// );
    ///
    /// std::assert_eq!(metrics.duration_ms, std::option::Option::Some(2000));
    /// std::assert_eq!(metrics.input_tokens, std::option::Option::Some(100));
    /// std::assert_eq!(metrics.output_tokens, std::option::Option::Some(200));
    /// std::assert!(metrics.success);
    /// // tokens_per_second = 200 tokens / 2 seconds = 100 t/s
    /// std::assert_eq!(metrics.tokens_per_second, std::option::Option::Some(100.0));
    /// ```
    pub fn record_completion(
        &mut self,
        duration: std::time::Duration,
        input_tokens: usize,
        output_tokens: usize,
    ) {
        let duration_ms = duration.as_millis() as u64;
        self.duration_ms = std::option::Option::Some(duration_ms);
        self.input_tokens = std::option::Option::Some(input_tokens);
        self.output_tokens = std::option::Option::Some(output_tokens);
        self.success = true;

        // Calculate tokens per second (output tokens / duration in seconds)
        if duration_ms > 0 {
            let duration_seconds = duration_ms as f64 / 1000.0;
            self.tokens_per_second = std::option::Option::Some(output_tokens as f64 / duration_seconds);
        }
    }

    /// Records failure of the operation with an error message.
    ///
    /// # Arguments
    ///
    /// * `error` - Error message describing the failure
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::performance_metrics::InferenceMetrics;
    ///
    /// let mut metrics = InferenceMetrics::new(
    ///     "enhancement".to_string(),
    ///     "ollama".to_string(),
    ///     "llama3.1".to_string(),
    ///     std::option::Option::None,
    /// );
    ///
    /// metrics.record_failure("Connection timeout".to_string());
    ///
    /// std::assert!(!metrics.success);
    /// std::assert_eq!(metrics.error, std::option::Option::Some("Connection timeout".to_string()));
    /// ```
    pub fn record_failure(&mut self, error: String) {
        self.success = false;
        self.error = std::option::Option::Some(error);
    }

    /// Calculates speed improvement percentage compared to another metrics record.
    ///
    /// Returns positive percentage if this metric is faster, negative if slower.
    ///
    /// # Arguments
    ///
    /// * `baseline` - Baseline metrics to compare against
    ///
    /// # Returns
    ///
    /// * `Some(percentage)` - Speed improvement percentage (-100.0 to +infinity)
    /// * `None` - If either metric lacks tokens_per_second data
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::performance_metrics::InferenceMetrics;
    /// use std::time::Duration;
    ///
    /// let mut ollama_metrics = InferenceMetrics::new(
    ///     "enhancement".to_string(),
    ///     "ollama".to_string(),
    ///     "llama3.1".to_string(),
    ///     std::option::Option::None,
    /// );
    /// ollama_metrics.record_completion(Duration::from_millis(2000), 100, 100);
    /// // 100 tokens / 2 seconds = 50 t/s
    ///
    /// let mut mlx_metrics = InferenceMetrics::new(
    ///     "enhancement".to_string(),
    ///     "mlx".to_string(),
    ///     "mlx-community/Phi-3-mini-4k-instruct".to_string(),
    ///     std::option::Option::None,
    /// );
    /// mlx_metrics.record_completion(Duration::from_millis(1333), 100, 100);
    /// // 100 tokens / 1.333 seconds = ~75 t/s
    ///
    /// let improvement = mlx_metrics.speed_improvement_vs(&ollama_metrics).unwrap();
    /// // (75 - 50) / 50 * 100 = 50% faster
    /// std::assert!((improvement - 50.0).abs() < 1.0);
    /// ```
    pub fn speed_improvement_vs(&self, baseline: &InferenceMetrics) -> std::option::Option<f64> {
        match (self.tokens_per_second, baseline.tokens_per_second) {
            (std::option::Option::Some(this_tps), std::option::Option::Some(baseline_tps)) => {
                if baseline_tps == 0.0 {
                    return std::option::Option::None;
                }
                let improvement = ((this_tps - baseline_tps) / baseline_tps) * 100.0;
                std::option::Option::Some(improvement)
            }
            _ => std::option::Option::None,
        }
    }
}

/// Aggregated performance metrics for a set of inference operations.
///
/// Provides summary statistics (mean, median, min, max) for comparing
/// performance across providers, models, or roles.
///
/// # Examples
///
/// ```
/// use task_orchestrator::domain::performance_metrics::{InferenceMetrics, MetricAggregate};
/// use std::time::Duration;
///
/// let mut metrics1 = InferenceMetrics::new(
///     "enhancement".to_string(),
///     "ollama".to_string(),
///     "llama3.1".to_string(),
///     std::option::Option::None,
/// );
/// metrics1.record_completion(Duration::from_millis(2000), 100, 100);
///
/// let mut metrics2 = InferenceMetrics::new(
///     "enhancement".to_string(),
///     "ollama".to_string(),
///     "llama3.1".to_string(),
///     std::option::Option::None,
/// );
/// metrics2.record_completion(Duration::from_millis(3000), 100, 150);
///
/// let aggregate = MetricAggregate::from_metrics(std::vec![metrics1, metrics2]);
/// std::assert_eq!(aggregate.count, 2);
/// std::assert_eq!(aggregate.success_count, 2);
/// std::assert!(aggregate.mean_duration_ms.is_some());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricAggregate {
    /// Number of metrics in this aggregate.
    pub count: usize,

    /// Number of successful operations.
    pub success_count: usize,

    /// Number of failed operations.
    pub failure_count: usize,

    /// Mean duration in milliseconds.
    pub mean_duration_ms: std::option::Option<f64>,

    /// Median duration in milliseconds.
    pub median_duration_ms: std::option::Option<f64>,

    /// Minimum duration in milliseconds.
    pub min_duration_ms: std::option::Option<u64>,

    /// Maximum duration in milliseconds.
    pub max_duration_ms: std::option::Option<u64>,

    /// Mean tokens per second.
    pub mean_tokens_per_second: std::option::Option<f64>,

    /// Median tokens per second.
    pub median_tokens_per_second: std::option::Option<f64>,

    /// Minimum tokens per second.
    pub min_tokens_per_second: std::option::Option<f64>,

    /// Maximum tokens per second.
    pub max_tokens_per_second: std::option::Option<f64>,

    /// Total input tokens across all operations.
    pub total_input_tokens: usize,

    /// Total output tokens across all operations.
    pub total_output_tokens: usize,
}

impl MetricAggregate {
    /// Creates an aggregate from a collection of individual metrics.
    ///
    /// Calculates summary statistics (mean, median, min, max) for all
    /// numeric fields.
    ///
    /// # Arguments
    ///
    /// * `metrics` - Vector of individual inference metrics
    ///
    /// # Returns
    ///
    /// MetricAggregate with computed statistics.
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::performance_metrics::{InferenceMetrics, MetricAggregate};
    /// use std::time::Duration;
    ///
    /// let mut m1 = InferenceMetrics::new("test".to_string(), "ollama".to_string(), "llama3.1".to_string(), std::option::Option::None);
    /// m1.record_completion(Duration::from_millis(1000), 50, 50);
    ///
    /// let mut m2 = InferenceMetrics::new("test".to_string(), "ollama".to_string(), "llama3.1".to_string(), std::option::Option::None);
    /// m2.record_completion(Duration::from_millis(2000), 50, 100);
    ///
    /// let aggregate = MetricAggregate::from_metrics(std::vec![m1, m2]);
    ///
    /// std::assert_eq!(aggregate.count, 2);
    /// std::assert_eq!(aggregate.success_count, 2);
    /// std::assert_eq!(aggregate.mean_duration_ms, std::option::Option::Some(1500.0)); // (1000 + 2000) / 2
    /// std::assert_eq!(aggregate.total_output_tokens, 150); // 50 + 100
    /// ```
    pub fn from_metrics(metrics: std::vec::Vec<InferenceMetrics>) -> Self {
        let count = metrics.len();
        let success_count = metrics.iter().filter(|m| m.success).count();
        let failure_count = count - success_count;

        // Collect durations
        let mut durations: std::vec::Vec<u64> = metrics
            .iter()
            .filter_map(|m| m.duration_ms)
            .collect();
        durations.sort_unstable();

        let mean_duration_ms = if !durations.is_empty() {
            let sum: u64 = durations.iter().sum();
            std::option::Option::Some(sum as f64 / durations.len() as f64)
        } else {
            std::option::Option::None
        };

        let median_duration_ms = if !durations.is_empty() {
            let mid = durations.len() / 2;
            if durations.len() % 2 == 0 {
                std::option::Option::Some((durations[mid - 1] + durations[mid]) as f64 / 2.0)
            } else {
                std::option::Option::Some(durations[mid] as f64)
            }
        } else {
            std::option::Option::None
        };

        let min_duration_ms = durations.first().copied();
        let max_duration_ms = durations.last().copied();

        // Collect tokens per second
        let mut tps_values: std::vec::Vec<f64> = metrics
            .iter()
            .filter_map(|m| m.tokens_per_second)
            .collect();
        tps_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mean_tokens_per_second = if !tps_values.is_empty() {
            let sum: f64 = tps_values.iter().sum();
            std::option::Option::Some(sum / tps_values.len() as f64)
        } else {
            std::option::Option::None
        };

        let median_tokens_per_second = if !tps_values.is_empty() {
            let mid = tps_values.len() / 2;
            if tps_values.len() % 2 == 0 {
                std::option::Option::Some((tps_values[mid - 1] + tps_values[mid]) / 2.0)
            } else {
                std::option::Option::Some(tps_values[mid])
            }
        } else {
            std::option::Option::None
        };

        let min_tokens_per_second = tps_values.first().copied();
        let max_tokens_per_second = tps_values.last().copied();

        // Sum token counts
        let total_input_tokens: usize = metrics
            .iter()
            .filter_map(|m| m.input_tokens)
            .sum();
        let total_output_tokens: usize = metrics
            .iter()
            .filter_map(|m| m.output_tokens)
            .sum();

        Self {
            count,
            success_count,
            failure_count,
            mean_duration_ms,
            median_duration_ms,
            min_duration_ms,
            max_duration_ms,
            mean_tokens_per_second,
            median_tokens_per_second,
            min_tokens_per_second,
            max_tokens_per_second,
            total_input_tokens,
            total_output_tokens,
        }
    }

    /// Compares this aggregate with another (baseline) and returns improvement percentage.
    ///
    /// # Arguments
    ///
    /// * `baseline` - Baseline aggregate to compare against
    ///
    /// # Returns
    ///
    /// * `Some(percentage)` - Speed improvement based on mean tokens per second
    /// * `None` - If either aggregate lacks mean_tokens_per_second
    ///
    /// # Examples
    ///
    /// ```
    /// use task_orchestrator::domain::performance_metrics::{InferenceMetrics, MetricAggregate};
    /// use std::time::Duration;
    ///
    /// // Create Ollama baseline
    /// let mut ollama1 = InferenceMetrics::new("test".to_string(), "ollama".to_string(), "llama3.1".to_string(), std::option::Option::None);
    /// ollama1.record_completion(Duration::from_millis(2000), 100, 100); // 50 t/s
    ///
    /// let ollama_aggregate = MetricAggregate::from_metrics(std::vec![ollama1]);
    ///
    /// // Create MLX comparison
    /// let mut mlx1 = InferenceMetrics::new("test".to_string(), "mlx".to_string(), "phi3".to_string(), std::option::Option::None);
    /// mlx1.record_completion(Duration::from_millis(1333), 100, 100); // ~75 t/s
    ///
    /// let mlx_aggregate = MetricAggregate::from_metrics(std::vec![mlx1]);
    ///
    /// let improvement = mlx_aggregate.speed_improvement_vs(&ollama_aggregate).unwrap();
    /// std::assert!((improvement - 50.0).abs() < 1.0); // ~50% faster
    /// ```
    pub fn speed_improvement_vs(&self, baseline: &MetricAggregate) -> std::option::Option<f64> {
        match (self.mean_tokens_per_second, baseline.mean_tokens_per_second) {
            (std::option::Option::Some(this_tps), std::option::Option::Some(baseline_tps)) => {
                if baseline_tps == 0.0 {
                    return std::option::Option::None;
                }
                let improvement = ((this_tps - baseline_tps) / baseline_tps) * 100.0;
                std::option::Option::Some(improvement)
            }
            _ => std::option::Option::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_metrics_creation() {
        // Test: Validates InferenceMetrics instantiation.
        // Justification: Ensures the constructor sets all fields correctly.
        let metrics = InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::Some(crate::domain::model_role::ModelRole::Enhancer),
        );

        std::assert!(!metrics.id.is_empty());
        std::assert_eq!(metrics.operation_type, "enhancement");
        std::assert_eq!(metrics.provider, "ollama");
        std::assert_eq!(metrics.model, "llama3.1");
        std::assert_eq!(metrics.role, std::option::Option::Some(crate::domain::model_role::ModelRole::Enhancer));
        std::assert!(metrics.duration_ms.is_none());
        std::assert!(!metrics.success);
    }

    #[test]
    fn test_record_completion() {
        // Test: Validates recording of successful operation.
        // Justification: Ensures duration, tokens, and calculated t/s are correct.
        let mut metrics = InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );

        metrics.record_completion(
            std::time::Duration::from_millis(2000),
            100,
            200,
        );

        std::assert_eq!(metrics.duration_ms, std::option::Option::Some(2000));
        std::assert_eq!(metrics.input_tokens, std::option::Option::Some(100));
        std::assert_eq!(metrics.output_tokens, std::option::Option::Some(200));
        std::assert_eq!(metrics.tokens_per_second, std::option::Option::Some(100.0)); // 200 tokens / 2 seconds
        std::assert!(metrics.success);
    }

    #[test]
    fn test_record_failure() {
        // Test: Validates recording of failed operation.
        // Justification: Ensures error message is captured and success is false.
        let mut metrics = InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );

        metrics.record_failure("Connection timeout".to_string());

        std::assert!(!metrics.success);
        std::assert_eq!(metrics.error, std::option::Option::Some("Connection timeout".to_string()));
    }

    #[test]
    fn test_speed_improvement_calculation() {
        // Test: Validates speed improvement percentage calculation.
        // Justification: Core metric for comparing MLX vs Ollama performance.
        let mut baseline = InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        baseline.record_completion(std::time::Duration::from_millis(2000), 100, 100);
        // 100 tokens / 2 seconds = 50 t/s

        let mut improved = InferenceMetrics::new(
            "enhancement".to_string(),
            "mlx".to_string(),
            "phi3".to_string(),
            std::option::Option::None,
        );
        improved.record_completion(std::time::Duration::from_millis(1333), 100, 100);
        // 100 tokens / 1.333 seconds = ~75 t/s

        let improvement = improved.speed_improvement_vs(&baseline).unwrap();
        // (75 - 50) / 50 * 100 = 50%
        std::assert!((improvement - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_metric_aggregate_creation() {
        // Test: Validates aggregate statistics calculation.
        // Justification: Ensures mean, median, min, max are calculated correctly.
        let mut m1 = InferenceMetrics::new(
            "test".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        m1.record_completion(std::time::Duration::from_millis(1000), 50, 50);

        let mut m2 = InferenceMetrics::new(
            "test".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        m2.record_completion(std::time::Duration::from_millis(3000), 50, 150);

        let aggregate = MetricAggregate::from_metrics(std::vec![m1, m2]);

        std::assert_eq!(aggregate.count, 2);
        std::assert_eq!(aggregate.success_count, 2);
        std::assert_eq!(aggregate.mean_duration_ms, std::option::Option::Some(2000.0));
        std::assert_eq!(aggregate.median_duration_ms, std::option::Option::Some(2000.0));
        std::assert_eq!(aggregate.min_duration_ms, std::option::Option::Some(1000));
        std::assert_eq!(aggregate.max_duration_ms, std::option::Option::Some(3000));
        std::assert_eq!(aggregate.total_input_tokens, 100);
        std::assert_eq!(aggregate.total_output_tokens, 200);
    }

    #[test]
    fn test_aggregate_speed_improvement() {
        // Test: Validates aggregate comparison between providers.
        // Justification: Used for benchmarking reports (e.g., "MLX is 45% faster than Ollama").
        let mut ollama1 = InferenceMetrics::new(
            "test".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        ollama1.record_completion(std::time::Duration::from_millis(2000), 100, 100);

        let ollama_aggregate = MetricAggregate::from_metrics(std::vec![ollama1]);

        let mut mlx1 = InferenceMetrics::new(
            "test".to_string(),
            "mlx".to_string(),
            "phi3".to_string(),
            std::option::Option::None,
        );
        mlx1.record_completion(std::time::Duration::from_millis(1333), 100, 100);

        let mlx_aggregate = MetricAggregate::from_metrics(std::vec![mlx1]);

        let improvement = mlx_aggregate.speed_improvement_vs(&ollama_aggregate).unwrap();
        std::assert!((improvement - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_aggregate_empty_metrics() {
        // Test: Validates aggregate handles empty input gracefully.
        // Justification: Edge case protection for benchmarking with no data.
        let aggregate = MetricAggregate::from_metrics(std::vec![]);

        std::assert_eq!(aggregate.count, 0);
        std::assert_eq!(aggregate.success_count, 0);
        std::assert!(aggregate.mean_duration_ms.is_none());
        std::assert!(aggregate.mean_tokens_per_second.is_none());
    }

    #[test]
    fn test_aggregate_with_failures() {
        // Test: Validates aggregate counts successes and failures separately.
        // Justification: Reliability metrics for provider comparison.
        let mut m1 = InferenceMetrics::new(
            "test".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        m1.record_completion(std::time::Duration::from_millis(1000), 50, 50);

        let mut m2 = InferenceMetrics::new(
            "test".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        m2.record_failure("Timeout".to_string());

        let aggregate = MetricAggregate::from_metrics(std::vec![m1, m2]);

        std::assert_eq!(aggregate.count, 2);
        std::assert_eq!(aggregate.success_count, 1);
        std::assert_eq!(aggregate.failure_count, 1);
    }
}
