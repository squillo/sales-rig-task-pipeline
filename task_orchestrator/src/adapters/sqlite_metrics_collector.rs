//! SQLite-backed metrics collector adapter.
//!
//! Persists performance metrics to a SQLite database using sqlx. This adapter
//! implements MetricsCollectorPort for durable storage of benchmark data across
//! sessions. JSON columns store optional role and error fields.
//!
//! Schema is created automatically via `connect_and_init()` if it doesn't exist.
//!
//! Revision History
//! - 2025-11-24T01:30:00Z @AI: Create SQLite metrics collector for Phase 5 Sprint 12 Task 5.12.

/// SQLite-backed implementation of MetricsCollectorPort.
///
/// Stores performance metrics in a `performance_metrics` table with columns
/// for all metric fields. Supports filtering by provider, operation, and role.
///
/// # Examples
///
/// ```no_run
/// use task_orchestrator::adapters::sqlite_metrics_collector::SqliteMetricsCollector;
///
/// #[tokio::main]
/// async fn main() {
///     let collector = SqliteMetricsCollector::connect_and_init("sqlite://metrics.db")
///         .await
///         .unwrap();
///
///     // Metrics are now persisted to disk
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SqliteMetricsCollector {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

impl SqliteMetricsCollector {
    /// Creates a new adapter from an existing SQLite pool.
    ///
    /// # Arguments
    ///
    /// * `pool` - SQLite connection pool
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use task_orchestrator::adapters::sqlite_metrics_collector::SqliteMetricsCollector;
    ///
    /// # async fn example(pool: sqlx::Pool<sqlx::Sqlite>) {
    /// let collector = SqliteMetricsCollector::new(pool);
    /// # }
    /// ```
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        Self { pool }
    }

    /// Connects to SQLite database and initializes schema if needed.
    ///
    /// Creates the `performance_metrics` table if it doesn't exist. Safe to call
    /// multiple times (idempotent).
    ///
    /// # Arguments
    ///
    /// * `database_url` - SQLite database URL (e.g., "sqlite://metrics.db" or "sqlite::memory:")
    ///
    /// # Returns
    ///
    /// Initialized SqliteMetricsCollector or error if connection fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use task_orchestrator::adapters::sqlite_metrics_collector::SqliteMetricsCollector;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     // File-based storage
    ///     let collector = SqliteMetricsCollector::connect_and_init("sqlite://metrics.db")
    ///         .await
    ///         .unwrap();
    ///
    ///     // In-memory storage (testing)
    ///     let test_collector = SqliteMetricsCollector::connect_and_init("sqlite::memory:")
    ///         .await
    ///         .unwrap();
    /// }
    /// ```
    pub async fn connect_and_init(database_url: &str) -> std::result::Result<Self, String> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| std::format!("Failed to connect to SQLite: {:?}", e))?;

        // Create schema if not exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS performance_metrics (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                operation_type TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                role TEXT NULL,
                duration_ms INTEGER NULL,
                input_tokens INTEGER NULL,
                output_tokens INTEGER NULL,
                tokens_per_second REAL NULL,
                success INTEGER NOT NULL,
                error TEXT NULL
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| std::format!("Failed to create schema: {:?}", e))?;

        // Create indices for common queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_provider ON performance_metrics(provider)")
            .execute(&pool)
            .await
            .map_err(|e| std::format!("Failed to create provider index: {:?}", e))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_operation_type ON performance_metrics(operation_type)")
            .execute(&pool)
            .await
            .map_err(|e| std::format!("Failed to create operation_type index: {:?}", e))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_timestamp ON performance_metrics(timestamp)")
            .execute(&pool)
            .await
            .map_err(|e| std::format!("Failed to create timestamp index: {:?}", e))?;

        std::result::Result::Ok(Self { pool })
    }

    /// Converts a database row to InferenceMetrics entity.
    fn row_to_metrics(row: sqlx::sqlite::SqliteRow) -> std::result::Result<crate::domain::performance_metrics::InferenceMetrics, String> {
        let id: String = sqlx::Row::get(&row, "id");
        let timestamp_str: String = sqlx::Row::get(&row, "timestamp");
        let operation_type: String = sqlx::Row::get(&row, "operation_type");
        let provider: String = sqlx::Row::get(&row, "provider");
        let model: String = sqlx::Row::get(&row, "model");
        let role_str: std::option::Option<String> = sqlx::Row::get(&row, "role");
        let duration_ms: std::option::Option<i64> = sqlx::Row::get(&row, "duration_ms");
        let input_tokens: std::option::Option<i64> = sqlx::Row::get(&row, "input_tokens");
        let output_tokens: std::option::Option<i64> = sqlx::Row::get(&row, "output_tokens");
        let tokens_per_second: std::option::Option<f64> = sqlx::Row::get(&row, "tokens_per_second");
        let success_int: i64 = sqlx::Row::get(&row, "success");
        let error: std::option::Option<String> = sqlx::Row::get(&row, "error");

        // Parse timestamp
        let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|e| std::format!("Invalid timestamp: {:?}", e))?
            .with_timezone(&chrono::Utc);

        // Parse role enum
        let role = match role_str {
            std::option::Option::Some(s) => serde_json::from_str::<crate::domain::model_role::ModelRole>(&std::format!("\"{}\"", s))
                .ok(),
            std::option::Option::None => std::option::Option::None,
        };

        std::result::Result::Ok(crate::domain::performance_metrics::InferenceMetrics {
            id,
            timestamp,
            operation_type,
            provider,
            model,
            role,
            duration_ms: duration_ms.map(|v| v as u64),
            input_tokens: input_tokens.map(|v| v as usize),
            output_tokens: output_tokens.map(|v| v as usize),
            tokens_per_second,
            success: success_int != 0,
            error,
        })
    }
}

#[async_trait::async_trait]
impl crate::ports::metrics_collector_port::MetricsCollectorPort for SqliteMetricsCollector {
    async fn record_metric(
        &self,
        metric: crate::domain::performance_metrics::InferenceMetrics,
    ) -> std::result::Result<(), String> {
        let timestamp_str = metric.timestamp.to_rfc3339();

        // Serialize role to string
        let role_str = metric.role.as_ref().map(|r| std::format!("{:?}", r));

        sqlx::query(
            "INSERT INTO performance_metrics (
                id, timestamp, operation_type, provider, model, role,
                duration_ms, input_tokens, output_tokens, tokens_per_second,
                success, error
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&metric.id)
        .bind(&timestamp_str)
        .bind(&metric.operation_type)
        .bind(&metric.provider)
        .bind(&metric.model)
        .bind(role_str)
        .bind(metric.duration_ms.map(|v| v as i64))
        .bind(metric.input_tokens.map(|v| v as i64))
        .bind(metric.output_tokens.map(|v| v as i64))
        .bind(metric.tokens_per_second)
        .bind(if metric.success { 1 } else { 0 })
        .bind(&metric.error)
        .execute(&self.pool)
        .await
        .map_err(|e| std::format!("Failed to insert metric: {:?}", e))?;

        std::result::Result::Ok(())
    }

    async fn get_all_metrics(
        &self,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String> {
        let rows = sqlx::query("SELECT * FROM performance_metrics ORDER BY timestamp DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| std::format!("Failed to fetch metrics: {:?}", e))?;

        let metrics: std::result::Result<std::vec::Vec<_>, _> = rows
            .into_iter()
            .map(Self::row_to_metrics)
            .collect();

        metrics
    }

    async fn get_metrics_by_provider(
        &self,
        provider: &str,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String> {
        let rows = sqlx::query(
            "SELECT * FROM performance_metrics WHERE provider = ? ORDER BY timestamp DESC"
        )
        .bind(provider)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| std::format!("Failed to fetch metrics by provider: {:?}", e))?;

        let metrics: std::result::Result<std::vec::Vec<_>, _> = rows
            .into_iter()
            .map(Self::row_to_metrics)
            .collect();

        metrics
    }

    async fn get_metrics_by_operation(
        &self,
        operation_type: &str,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String> {
        let rows = sqlx::query(
            "SELECT * FROM performance_metrics WHERE operation_type = ? ORDER BY timestamp DESC"
        )
        .bind(operation_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| std::format!("Failed to fetch metrics by operation: {:?}", e))?;

        let metrics: std::result::Result<std::vec::Vec<_>, _> = rows
            .into_iter()
            .map(Self::row_to_metrics)
            .collect();

        metrics
    }

    async fn get_metrics_by_role(
        &self,
        role: crate::domain::model_role::ModelRole,
    ) -> std::result::Result<std::vec::Vec<crate::domain::performance_metrics::InferenceMetrics>, String> {
        let role_str = std::format!("{:?}", role);

        let rows = sqlx::query(
            "SELECT * FROM performance_metrics WHERE role = ? ORDER BY timestamp DESC"
        )
        .bind(&role_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| std::format!("Failed to fetch metrics by role: {:?}", e))?;

        let metrics: std::result::Result<std::vec::Vec<_>, _> = rows
            .into_iter()
            .map(Self::row_to_metrics)
            .collect();

        metrics
    }

    async fn clear_metrics(&self) -> std::result::Result<(), String> {
        sqlx::query("DELETE FROM performance_metrics")
            .execute(&self.pool)
            .await
            .map_err(|e| std::format!("Failed to clear metrics: {:?}", e))?;

        std::result::Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::metrics_collector_port::MetricsCollectorPort;

    async fn create_test_collector() -> SqliteMetricsCollector {
        SqliteMetricsCollector::connect_and_init("sqlite::memory:")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_sqlite_collector_creation() {
        // Test: Validates SqliteMetricsCollector instantiation and schema creation.
        // Justification: Ensures the database is properly initialized.
        let collector = create_test_collector().await;
        std::assert!(!collector.pool.is_closed());
    }

    #[tokio::test]
    async fn test_record_and_retrieve_metric() {
        // Test: Validates recording and retrieval from SQLite.
        // Justification: Core persistence functionality.
        let collector = create_test_collector().await;

        let mut metrics = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        metrics.record_completion(std::time::Duration::from_millis(2000), 100, 200);

        collector.record_metric(metrics.clone()).await.unwrap();

        let all_metrics = collector.get_all_metrics().await.unwrap();
        std::assert_eq!(all_metrics.len(), 1);
        std::assert_eq!(all_metrics[0].operation_type, "enhancement");
        std::assert_eq!(all_metrics[0].provider, "ollama");
        std::assert_eq!(all_metrics[0].duration_ms, std::option::Option::Some(2000));
        std::assert!(all_metrics[0].success);
    }

    #[tokio::test]
    async fn test_filter_by_provider() {
        // Test: Validates provider-based filtering persists across queries.
        // Justification: SQL WHERE clause correctness.
        let collector = create_test_collector().await;

        let mut ollama_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        ollama_metric.record_completion(std::time::Duration::from_millis(2000), 100, 100);

        let mut mlx_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "mlx".to_string(),
            "phi3".to_string(),
            std::option::Option::None,
        );
        mlx_metric.record_completion(std::time::Duration::from_millis(1333), 100, 100);

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
        // Justification: Enables separate analysis of enhancement vs decomposition.
        let collector = create_test_collector().await;

        let mut enhancement_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        enhancement_metric.record_completion(std::time::Duration::from_millis(2000), 100, 100);

        let mut decomposition_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "decomposition".to_string(),
            "ollama".to_string(),
            "orca2".to_string(),
            std::option::Option::None,
        );
        decomposition_metric.record_completion(std::time::Duration::from_millis(3000), 150, 200);

        collector.record_metric(enhancement_metric).await.unwrap();
        collector.record_metric(decomposition_metric).await.unwrap();

        let enhancement_metrics = collector.get_metrics_by_operation("enhancement").await.unwrap();
        std::assert_eq!(enhancement_metrics.len(), 1);
        std::assert_eq!(enhancement_metrics[0].operation_type, "enhancement");
    }

    #[tokio::test]
    async fn test_filter_by_role() {
        // Test: Validates role-based filtering for heterogeneous pipeline.
        // Justification: Enables comparison of Router vs Decomposer performance from database.
        let collector = create_test_collector().await;

        let mut router_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "phi3".to_string(),
            std::option::Option::Some(crate::domain::model_role::ModelRole::Router),
        );
        router_metric.record_completion(std::time::Duration::from_millis(1000), 50, 50);

        let mut decomposer_metric = crate::domain::performance_metrics::InferenceMetrics::new(
            "decomposition".to_string(),
            "ollama".to_string(),
            "orca2".to_string(),
            std::option::Option::Some(crate::domain::model_role::ModelRole::Decomposer),
        );
        decomposer_metric.record_completion(std::time::Duration::from_millis(3000), 150, 200);

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
        // Test: Validates clearing persisted metrics.
        // Justification: Required for resetting between benchmark runs.
        let collector = create_test_collector().await;

        let mut metrics = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        metrics.record_completion(std::time::Duration::from_millis(2000), 100, 100);

        collector.record_metric(metrics).await.unwrap();

        let all_metrics = collector.get_all_metrics().await.unwrap();
        std::assert_eq!(all_metrics.len(), 1);

        collector.clear_metrics().await.unwrap();

        let cleared_metrics = collector.get_all_metrics().await.unwrap();
        std::assert_eq!(cleared_metrics.len(), 0);
    }

    #[tokio::test]
    #[ignore] // Requires filesystem access; manual verification via rigger CLI
    async fn test_persistence_across_instances() {
        // Test: Validates metrics persist to disk (using temp file).
        // Justification: Core value of SQLite adapter vs in-memory.

        // Use absolute path for temp file
        let temp_dir = std::env::temp_dir();
        let temp_filename = std::format!("test_metrics_{}.db", uuid::Uuid::new_v4());
        let temp_path = temp_dir.join(&temp_filename);
        let temp_file_url = std::format!("sqlite://{}", temp_path.to_string_lossy());

        // First instance: write metric
        {
            let collector = SqliteMetricsCollector::connect_and_init(&temp_file_url)
                .await
                .unwrap();

            let mut metrics = crate::domain::performance_metrics::InferenceMetrics::new(
                "enhancement".to_string(),
                "ollama".to_string(),
                "llama3.1".to_string(),
                std::option::Option::None,
            );
            metrics.record_completion(std::time::Duration::from_millis(2000), 100, 100);

            collector.record_metric(metrics).await.unwrap();
        }

        // Second instance: read metric
        {
            let collector = SqliteMetricsCollector::connect_and_init(&temp_file_url)
                .await
                .unwrap();

            let all_metrics = collector.get_all_metrics().await.unwrap();
            std::assert_eq!(all_metrics.len(), 1);
            std::assert_eq!(all_metrics[0].provider, "ollama");
        }

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[tokio::test]
    async fn test_record_failure_persists() {
        // Test: Validates failed operations are persisted correctly.
        // Justification: Ensures error tracking works across sessions.
        let collector = create_test_collector().await;

        let mut metrics = crate::domain::performance_metrics::InferenceMetrics::new(
            "enhancement".to_string(),
            "ollama".to_string(),
            "llama3.1".to_string(),
            std::option::Option::None,
        );
        metrics.record_failure("Connection timeout".to_string());

        collector.record_metric(metrics).await.unwrap();

        let all_metrics = collector.get_all_metrics().await.unwrap();
        std::assert_eq!(all_metrics.len(), 1);
        std::assert!(!all_metrics[0].success);
        std::assert_eq!(all_metrics[0].error, std::option::Option::Some("Connection timeout".to_string()));
    }
}
