use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicI64, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

pub mod business;
pub mod database;
pub mod health;
pub mod metrics;
pub mod otel;
pub mod performance;


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
    pub response_time_histogram: ResponseTimeHistogram,
}

/// Histogram-based response time tracking for accurate percentiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeHistogram {
    pub buckets: Vec<HistogramBucket>,
    pub total_samples: u64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound_ms: f64,
    pub count: u64,
}

impl Default for ResponseTimeHistogram {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseTimeHistogram {
    pub fn new() -> Self {
        // Create buckets for response times: 1ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 2.5s, 5s, 10s, +Inf
        let bucket_bounds = vec![
            1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 
            1000.0, 2500.0, 5000.0, 10000.0, f64::INFINITY
        ];
        
        let buckets = bucket_bounds.into_iter()
            .map(|bound| HistogramBucket {
                upper_bound_ms: bound,
                count: 0,
            })
            .collect();
        
        Self {
            buckets,
            total_samples: 0,
            min_response_time_ms: f64::INFINITY,
            max_response_time_ms: 0.0,
        }
    }
    
    pub fn record_response_time(&mut self, response_time_ms: f64) {
        self.total_samples += 1;
        self.min_response_time_ms = self.min_response_time_ms.min(response_time_ms);
        self.max_response_time_ms = self.max_response_time_ms.max(response_time_ms);
        
        // Find the appropriate bucket and increment its count
        for bucket in &mut self.buckets {
            if response_time_ms <= bucket.upper_bound_ms {
                bucket.count += 1;
                break;
            }
        }
    }
    
    pub fn calculate_percentile(&self, percentile: f64) -> f64 {
        if self.total_samples == 0 {
            return 0.0;
        }
        
        let target_count = (self.total_samples as f64 * percentile / 100.0).ceil() as u64;
        let mut cumulative_count = 0u64;
        
        for (i, bucket) in self.buckets.iter().enumerate() {
            cumulative_count += bucket.count;
            
            if cumulative_count >= target_count {
                // Linear interpolation within bucket
                if i == 0 {
                    return bucket.upper_bound_ms * (target_count as f64 / bucket.count as f64);
                }
                
                let prev_bound = if i > 0 { self.buckets[i - 1].upper_bound_ms } else { 0.0 };
                let bucket_width = bucket.upper_bound_ms - prev_bound;
                let bucket_start_count = cumulative_count - bucket.count;
                let position_in_bucket = (target_count as f64 - bucket_start_count as f64) / bucket.count as f64;
                
                return prev_bound + (bucket_width * position_in_bucket);
            }
        }
        
        self.max_response_time_ms
    }
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
    pub p90_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub p999_response_time_ms: f64,
    pub error_rate_percent: f32,
    pub throughput_requests_per_minute: f64,
    pub active_users: u64,
    pub database_pool_utilization: f32,
    pub cache_hit_rate: f32,
    pub database_p50_response_time_ms: f64,
    pub database_p95_response_time_ms: f64,
    pub database_p99_response_time_ms: f64,
    pub slowest_endpoints: Vec<SlowEndpointInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowEndpointInfo {
    pub endpoint: String,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub total_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPerformanceMetrics {
    pub endpoints: std::collections::HashMap<String, EndpointPerformanceDetails>,
    pub total_endpoints: usize,
    pub measurement_period_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPerformanceDetails {
    pub total_requests: u64,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
    pub p50_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub p999_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub last_accessed: DateTime<Utc>,
    pub histogram_buckets: Vec<HistogramBucket>,
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
                response_time_histogram: ResponseTimeHistogram::new(),
            });

        metrics.total_requests += 1;
        metrics.total_duration_ms += duration_ms;
        metrics.last_accessed = Utc::now();

        if is_error {
            metrics.error_count += 1;
        }

        metrics.avg_duration_ms = metrics.total_duration_ms as f64 / metrics.total_requests as f64;
        
        // Record response time in histogram for accurate percentile calculation
        metrics.response_time_histogram.record_response_time(duration_ms as f64);
    }

    /// Record database query
    pub fn record_db_query(&self) {
        self.database_queries.fetch_add(1, Ordering::Relaxed);
    }

    /// Record database query with response time
    pub async fn record_db_query_with_timing(&self, duration_ms: u64) {
        self.database_queries.fetch_add(1, Ordering::Relaxed);
        
        // Track database queries as a special endpoint for response time percentiles
        self.record_request("database_queries", duration_ms, false).await;
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
