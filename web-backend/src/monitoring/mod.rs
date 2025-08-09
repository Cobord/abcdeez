use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicI64, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

pub mod health;
pub mod metrics;
pub mod performance;

use crate::state::AppState;

/// Global metrics collector for the application
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    pub request_count: Arc<AtomicU64>,
    pub request_duration_ms: Arc<AtomicU64>,
    pub error_count: Arc<AtomicU64>,
    pub active_connections: Arc<AtomicI64>,
    pub database_queries: Arc<AtomicU64>,
    pub cache_hits: Arc<AtomicU64>,
    pub cache_misses: Arc<AtomicU64>,
    pub auth_successes: Arc<AtomicU64>,
    pub auth_failures: Arc<AtomicU64>,
    pub task_generations: Arc<AtomicU64>,
    pub learner_updates: Arc<AtomicU64>,
    pub session_creations: Arc<AtomicU64>,
    pub analytics_requests: Arc<AtomicU64>,
    pub admin_actions: Arc<AtomicU64>,
    pub start_time: SystemTime,
    pub endpoint_metrics: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub total_requests: u64,
    pub total_duration_ms: u64,
    pub error_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub avg_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub uptime_seconds: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
    pub database_status: ComponentStatus,
    pub cache_status: ComponentStatus,
    pub active_connections: i64,
    pub error_rate: f32,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub last_error: Option<String>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub request_rate_per_second: f64,
    pub avg_response_time_ms: f64,
    pub p50_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub error_rate_percent: f32,
    pub throughput_requests_per_minute: f64,
    pub active_users: u64,
    pub database_pool_utilization: f32,
    pub cache_hit_rate: f32,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            request_count: Arc::new(AtomicU64::new(0)),
            request_duration_ms: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicI64::new(0)),
            database_queries: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            auth_successes: Arc::new(AtomicU64::new(0)),
            auth_failures: Arc::new(AtomicU64::new(0)),
            task_generations: Arc::new(AtomicU64::new(0)),
            learner_updates: Arc::new(AtomicU64::new(0)),
            session_creations: Arc::new(AtomicU64::new(0)),
            analytics_requests: Arc::new(AtomicU64::new(0)),
            admin_actions: Arc::new(AtomicU64::new(0)),
            start_time: SystemTime::now(),
            endpoint_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Increment request count and record duration
    pub async fn record_request(&self, endpoint: &str, duration_ms: u64, is_error: bool) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.request_duration_ms
            .fetch_add(duration_ms, Ordering::Relaxed);

        if is_error {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // Update endpoint-specific metrics
        let mut endpoint_metrics = self.endpoint_metrics.write().await;
        let metrics = endpoint_metrics
            .entry(endpoint.to_string())
            .or_insert(EndpointMetrics {
                total_requests: 0,
                total_duration_ms: 0,
                error_count: 0,
                last_accessed: Utc::now(),
                avg_duration_ms: 0.0,
            });

        metrics.total_requests += 1;
        metrics.total_duration_ms += duration_ms;
        metrics.last_accessed = Utc::now();

        if is_error {
            metrics.error_count += 1;
        }

        metrics.avg_duration_ms = metrics.total_duration_ms as f64 / metrics.total_requests as f64;
    }

    /// Record database query
    pub fn record_db_query(&self) {
        self.database_queries.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache hit/miss
    pub fn record_cache_access(&self, hit: bool) {
        if hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record authentication result
    pub fn record_auth(&self, success: bool) {
        if success {
            self.auth_successes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.auth_failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record various application events
    pub fn record_task_generation(&self) {
        self.task_generations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_learner_update(&self) {
        self.learner_updates.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_session_creation(&self) {
        self.session_creations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_analytics_request(&self) {
        self.analytics_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_admin_action(&self) {
        self.admin_actions.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment/decrement active connections
    pub fn connection_opened(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub async fn get_snapshot(&self) -> MetricsSnapshot {
        let uptime = self.start_time.elapsed().unwrap_or(Duration::from_secs(0));
        let endpoint_metrics = self.endpoint_metrics.read().await.clone();

        MetricsSnapshot {
            uptime_seconds: uptime.as_secs(),
            request_count: self.request_count.load(Ordering::Relaxed),
            request_duration_ms: self.request_duration_ms.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            database_queries: self.database_queries.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            auth_successes: self.auth_successes.load(Ordering::Relaxed),
            auth_failures: self.auth_failures.load(Ordering::Relaxed),
            task_generations: self.task_generations.load(Ordering::Relaxed),
            learner_updates: self.learner_updates.load(Ordering::Relaxed),
            session_creations: self.session_creations.load(Ordering::Relaxed),
            analytics_requests: self.analytics_requests.load(Ordering::Relaxed),
            admin_actions: self.admin_actions.load(Ordering::Relaxed),
            endpoint_metrics,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub uptime_seconds: u64,
    pub request_count: u64,
    pub request_duration_ms: u64,
    pub error_count: u64,
    pub active_connections: i64,
    pub database_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub auth_successes: u64,
    pub auth_failures: u64,
    pub task_generations: u64,
    pub learner_updates: u64,
    pub session_creations: u64,
    pub analytics_requests: u64,
    pub admin_actions: u64,
    pub endpoint_metrics: HashMap<String, EndpointMetrics>,
    pub timestamp: DateTime<Utc>,
}

/// Global metrics instance
static METRICS: once_cell::sync::Lazy<MetricsCollector> =
    once_cell::sync::Lazy::new(MetricsCollector::new);

pub fn global_metrics() -> &'static MetricsCollector {
    &METRICS
}

/// Request timing helper
pub struct RequestTimer {
    start: Instant,
    endpoint: String,
}

impl RequestTimer {
    pub fn new(endpoint: String) -> Self {
        Self {
            start: Instant::now(),
            endpoint,
        }
    }

    pub async fn finish(self, is_error: bool) {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        global_metrics()
            .record_request(&self.endpoint, duration_ms, is_error)
            .await;
    }
}
