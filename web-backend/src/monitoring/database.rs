use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Database performance monitoring and query analysis
#[derive(Debug, Clone)]
pub struct DatabaseMonitor {
    query_metrics: Arc<RwLock<HashMap<String, QueryMetrics>>>,
    slow_query_threshold_ms: u64,
    connection_metrics: ConnectionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub query_type: String,
    pub total_executions: u64,
    pub total_duration_ms: u64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub slow_query_count: u64,
    pub error_count: u64,
    pub last_executed: DateTime<Utc>,
    pub first_executed: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ConnectionMetrics {
    pub active_connections: Arc<AtomicU64>,
    pub total_connections_created: Arc<AtomicU64>,
    pub connection_errors: Arc<AtomicU64>,
    pub connection_timeouts: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowQuery {
    pub query_type: String,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub parameters: Option<String>,
    pub trace_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealthMetrics {
    pub total_queries: u64,
    pub avg_query_duration_ms: f64,
    pub slow_query_percentage: f64,
    pub error_rate_percentage: f64,
    pub active_connections: u64,
    pub connection_pool_utilization: f64,
    pub query_types: Vec<QueryTypeMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTypeMetrics {
    pub query_type: String,
    pub execution_count: u64,
    pub avg_duration_ms: f64,
    pub error_count: u64,
    pub slow_query_count: u64,
}

impl Default for ConnectionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionMetrics {
    pub fn new() -> Self {
        Self {
            active_connections: Arc::new(AtomicU64::new(0)),
            total_connections_created: Arc::new(AtomicU64::new(0)),
            connection_errors: Arc::new(AtomicU64::new(0)),
            connection_timeouts: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn connection_acquired(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        self.total_connections_created
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_released(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn connection_error(&self) {
        self.connection_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_timeout(&self) {
        self.connection_timeouts.fetch_add(1, Ordering::Relaxed);
    }
}

impl DatabaseMonitor {
    pub fn new(slow_query_threshold_ms: u64) -> Self {
        Self {
            query_metrics: Arc::new(RwLock::new(HashMap::new())),
            slow_query_threshold_ms,
            connection_metrics: ConnectionMetrics::new(),
        }
    }

    /// Track a database query execution
    #[instrument(level = "debug", fields(query_type = %query_type, duration_ms))]
    pub async fn track_query(
        &self,
        query_type: &str,
        duration: Duration,
        is_error: bool,
        trace_id: Option<&str>,
    ) {
        let duration_ms = duration.as_millis() as u64;
        let now = Utc::now();
        let is_slow = duration_ms > self.slow_query_threshold_ms;

        tracing::Span::current().record("duration_ms", &duration_ms);

        // Log based on performance characteristics
        if is_error {
            error!(
                query_type = %query_type,
                duration_ms = duration_ms,
                trace_id = ?trace_id,
                "Database query failed"
            );
        } else if is_slow {
            warn!(
                query_type = %query_type,
                duration_ms = duration_ms,
                threshold_ms = self.slow_query_threshold_ms,
                trace_id = ?trace_id,
                "Slow database query detected"
            );
        } else {
            debug!(
                query_type = %query_type,
                duration_ms = duration_ms,
                trace_id = ?trace_id,
                "Database query completed"
            );
        }

        // Update metrics
        let mut metrics = self.query_metrics.write().await;
        let query_metrics = metrics
            .entry(query_type.to_string())
            .or_insert(QueryMetrics {
                query_type: query_type.to_string(),
                total_executions: 0,
                total_duration_ms: 0,
                min_duration_ms: duration_ms,
                max_duration_ms: duration_ms,
                avg_duration_ms: 0.0,
                slow_query_count: 0,
                error_count: 0,
                last_executed: now,
                first_executed: now,
            });

        // Update statistics
        query_metrics.total_executions += 1;
        query_metrics.total_duration_ms += duration_ms;
        query_metrics.min_duration_ms = query_metrics.min_duration_ms.min(duration_ms);
        query_metrics.max_duration_ms = query_metrics.max_duration_ms.max(duration_ms);
        query_metrics.avg_duration_ms =
            query_metrics.total_duration_ms as f64 / query_metrics.total_executions as f64;
        query_metrics.last_executed = now;

        if is_slow {
            query_metrics.slow_query_count += 1;
        }

        if is_error {
            query_metrics.error_count += 1;
        }

        // Record global metrics
        crate::monitoring::global_metrics().record_db_query();

        info!(
            query_type = %query_type,
            total_executions = query_metrics.total_executions,
            avg_duration_ms = query_metrics.avg_duration_ms,
            slow_queries = query_metrics.slow_query_count,
            errors = query_metrics.error_count,
            "Database query metrics updated"
        );
    }

    /// Get comprehensive database health metrics
    pub async fn get_health_metrics(&self) -> DatabaseHealthMetrics {
        let metrics = self.query_metrics.read().await;

        let total_queries: u64 = metrics.values().map(|m| m.total_executions).sum();
        let total_duration: u64 = metrics.values().map(|m| m.total_duration_ms).sum();
        let total_slow_queries: u64 = metrics.values().map(|m| m.slow_query_count).sum();
        let total_errors: u64 = metrics.values().map(|m| m.error_count).sum();

        let avg_query_duration_ms = if total_queries > 0 {
            total_duration as f64 / total_queries as f64
        } else {
            0.0
        };

        let slow_query_percentage = if total_queries > 0 {
            (total_slow_queries as f64 / total_queries as f64) * 100.0
        } else {
            0.0
        };

        let error_rate_percentage = if total_queries > 0 {
            (total_errors as f64 / total_queries as f64) * 100.0
        } else {
            0.0
        };

        let query_types: Vec<QueryTypeMetrics> = metrics
            .values()
            .map(|m| QueryTypeMetrics {
                query_type: m.query_type.clone(),
                execution_count: m.total_executions,
                avg_duration_ms: m.avg_duration_ms,
                error_count: m.error_count,
                slow_query_count: m.slow_query_count,
            })
            .collect();

        let active_connections = self
            .connection_metrics
            .active_connections
            .load(Ordering::Relaxed);

        DatabaseHealthMetrics {
            total_queries,
            avg_query_duration_ms,
            slow_query_percentage,
            error_rate_percentage,
            active_connections,
            connection_pool_utilization: 0.0, // Would need connection pool info
            query_types,
        }
    }

    /// Get slow queries that exceed the threshold
    pub async fn get_slow_queries(&self, limit: usize) -> Vec<SlowQuery> {
        let metrics = self.query_metrics.read().await;

        let mut slow_queries: Vec<SlowQuery> = metrics
            .values()
            .filter(|m| m.slow_query_count > 0)
            .map(|m| SlowQuery {
                query_type: m.query_type.clone(),
                duration_ms: m.max_duration_ms,
                timestamp: m.last_executed,
                parameters: None, // Would need to track parameters separately
                trace_id: None,   // Would need to correlate with trace IDs
                error: None,
            })
            .collect();

        slow_queries.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));
        slow_queries.truncate(limit);
        slow_queries
    }

    /// Generate a database performance report
    pub async fn generate_performance_report(&self) -> String {
        let health_metrics = self.get_health_metrics().await;
        let slow_queries = self.get_slow_queries(10).await;

        let mut report = String::new();
        report.push_str("Database Performance Report\n");
        report.push_str("==========================\n\n");

        report.push_str(&format!(
            "Overall Statistics:\n\
            - Total Queries: {}\n\
            - Average Query Duration: {:.2}ms\n\
            - Slow Query Percentage: {:.2}%\n\
            - Error Rate: {:.2}%\n\
            - Active Connections: {}\n\n",
            health_metrics.total_queries,
            health_metrics.avg_query_duration_ms,
            health_metrics.slow_query_percentage,
            health_metrics.error_rate_percentage,
            health_metrics.active_connections
        ));

        report.push_str("Query Type Breakdown:\n");
        for query_type in &health_metrics.query_types {
            report.push_str(&format!(
                "- {}: {} executions, {:.2}ms avg, {} errors, {} slow\n",
                query_type.query_type,
                query_type.execution_count,
                query_type.avg_duration_ms,
                query_type.error_count,
                query_type.slow_query_count
            ));
        }

        if !slow_queries.is_empty() {
            report.push_str("\nSlowest Queries:\n");
            for (i, slow_query) in slow_queries.iter().enumerate() {
                report.push_str(&format!(
                    "{}. {} - {}ms ({})\n",
                    i + 1,
                    slow_query.query_type,
                    slow_query.duration_ms,
                    slow_query.timestamp.format("%Y-%m-%d %H:%M:%S")
                ));
            }
        }

        report.push_str(&format!(
            "\nReport Generated: {}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        report
    }

    /// Reset all metrics (useful for testing or periodic resets)
    pub async fn reset_metrics(&self) {
        let mut metrics = self.query_metrics.write().await;
        metrics.clear();

        // Reset connection metrics
        self.connection_metrics
            .active_connections
            .store(0, Ordering::Relaxed);
        self.connection_metrics
            .total_connections_created
            .store(0, Ordering::Relaxed);
        self.connection_metrics
            .connection_errors
            .store(0, Ordering::Relaxed);
        self.connection_metrics
            .connection_timeouts
            .store(0, Ordering::Relaxed);

        info!("Database monitoring metrics reset");
    }

    /// Get connection metrics
    pub fn get_connection_metrics(&self) -> (u64, u64, u64, u64) {
        (
            self.connection_metrics
                .active_connections
                .load(Ordering::Relaxed),
            self.connection_metrics
                .total_connections_created
                .load(Ordering::Relaxed),
            self.connection_metrics
                .connection_errors
                .load(Ordering::Relaxed),
            self.connection_metrics
                .connection_timeouts
                .load(Ordering::Relaxed),
        )
    }
}

/// Global database monitor instance
static DATABASE_MONITOR: once_cell::sync::Lazy<DatabaseMonitor> =
    once_cell::sync::Lazy::new(|| DatabaseMonitor::new(1000)); // 1 second slow query threshold

pub fn global_database_monitor() -> &'static DatabaseMonitor {
    &DATABASE_MONITOR
}

/// Utility macro for tracking database queries
#[macro_export]
macro_rules! track_db_query {
    ($query_type:expr, $query:expr) => {{
        let start = std::time::Instant::now();
        let result = $query;
        let duration = start.elapsed();
        let is_error = result.is_err();

        // Get trace ID from current span if available
        let trace_id = tracing::Span::current()
            .field("trace_id")
            .and_then(|field| {
                // This would need proper implementation to extract trace_id
                None::<String>
            });

        tokio::spawn(async move {
            crate::monitoring::database::global_database_monitor()
                .track_query($query_type, duration, is_error, trace_id.as_deref())
                .await;
        });

        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_database_monitor() {
        let monitor = DatabaseMonitor::new(100); // 100ms threshold

        // Track some queries
        monitor
            .track_query(
                "SELECT",
                Duration::from_millis(50),
                false,
                Some("trace-123"),
            )
            .await;
        monitor
            .track_query(
                "INSERT",
                Duration::from_millis(150),
                false,
                Some("trace-124"),
            )
            .await;
        monitor
            .track_query(
                "UPDATE",
                Duration::from_millis(200),
                true,
                Some("trace-125"),
            )
            .await;

        let health_metrics = monitor.get_health_metrics().await;

        assert_eq!(health_metrics.total_queries, 3);
        assert!(health_metrics.slow_query_percentage > 0.0);
        assert!(health_metrics.error_rate_percentage > 0.0);
        assert!(!health_metrics.query_types.is_empty());
    }

    #[tokio::test]
    async fn test_slow_query_detection() {
        let monitor = DatabaseMonitor::new(100); // 100ms threshold

        // Track a slow query
        monitor
            .track_query("SLOW_SELECT", Duration::from_millis(500), false, None)
            .await;

        let slow_queries = monitor.get_slow_queries(10).await;
        assert!(!slow_queries.is_empty());
        assert_eq!(slow_queries[0].query_type, "SLOW_SELECT");
        assert!(slow_queries[0].duration_ms >= 500);
    }

    #[tokio::test]
    async fn test_connection_metrics() {
        let monitor = DatabaseMonitor::new(100);

        monitor.connection_metrics.connection_acquired();
        monitor.connection_metrics.connection_acquired();
        monitor.connection_metrics.connection_released();
        monitor.connection_metrics.connection_error();

        let (active, total, errors, timeouts) = monitor.get_connection_metrics();
        assert_eq!(active, 1);
        assert_eq!(total, 2);
        assert_eq!(errors, 1);
        assert_eq!(timeouts, 0);
    }
}
