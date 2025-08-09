use axum::{extract::State, Json};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::{
    error::AppResult,
    monitoring::{global_metrics, PerformanceMetrics},
    state::AppState,
};

/// Get detailed performance metrics
pub async fn get_performance_metrics(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<PerformanceMetrics>> {
    if !state.config.performance_monitoring_enabled {
        return Err(crate::error::AppError::Forbidden);
    }

    let snapshot = global_metrics().get_snapshot().await;

    // Calculate performance metrics
    let request_rate_per_second = if snapshot.uptime_seconds > 0 {
        snapshot.request_count as f64 / snapshot.uptime_seconds as f64
    } else {
        0.0
    };

    let avg_response_time_ms = if snapshot.request_count > 0 {
        snapshot.request_duration_ms as f64 / snapshot.request_count as f64
    } else {
        0.0
    };

    // Calculate percentiles from endpoint metrics
    let mut all_durations = Vec::new();
    for metrics in snapshot.endpoint_metrics.values() {
        // Approximate individual response times from averages
        // This is a simplified approach - in production you'd store actual response times
        for _ in 0..metrics.total_requests {
            all_durations.push(metrics.avg_duration_ms);
        }
    }

    all_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let (p50, p95, p99) = calculate_percentiles(&all_durations);

    let error_rate_percent = if snapshot.request_count > 0 {
        (snapshot.error_count as f32 / snapshot.request_count as f32) * 100.0
    } else {
        0.0
    };

    let throughput_requests_per_minute = request_rate_per_second * 60.0;

    // Estimate active users based on recent activity
    let active_users = estimate_active_users(&snapshot).await;

    // Database pool utilization (simplified)
    let database_pool_utilization = calculate_db_pool_utilization(&state).await;

    // Cache hit rate
    let cache_hit_rate = if snapshot.cache_hits + snapshot.cache_misses > 0 {
        snapshot.cache_hits as f32 / (snapshot.cache_hits + snapshot.cache_misses) as f32
    } else {
        0.0
    };

    let performance = PerformanceMetrics {
        request_rate_per_second,
        avg_response_time_ms,
        p50_response_time_ms: p50,
        p95_response_time_ms: p95,
        p99_response_time_ms: p99,
        error_rate_percent,
        throughput_requests_per_minute,
        active_users,
        database_pool_utilization,
        cache_hit_rate,
    };

    Ok(Json(performance))
}

fn calculate_percentiles(sorted_durations: &[f64]) -> (f64, f64, f64) {
    if sorted_durations.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let len = sorted_durations.len();
    let p50_idx = (len as f64 * 0.5) as usize;
    let p95_idx = (len as f64 * 0.95) as usize;
    let p99_idx = (len as f64 * 0.99) as usize;

    (
        sorted_durations.get(p50_idx).copied().unwrap_or(0.0),
        sorted_durations
            .get(p95_idx.min(len - 1))
            .copied()
            .unwrap_or(0.0),
        sorted_durations
            .get(p99_idx.min(len - 1))
            .copied()
            .unwrap_or(0.0),
    )
}

async fn estimate_active_users(snapshot: &crate::monitoring::MetricsSnapshot) -> u64 {
    // Simplified active user estimation based on authentication events
    // In a real system, you'd track unique session IDs or user IDs within a time window

    // Estimate based on recent auth successes
    let recent_auth_rate = if snapshot.uptime_seconds > 0 && snapshot.uptime_seconds < 3600 {
        // For uptime less than 1 hour, use total auth successes
        snapshot.auth_successes
    } else {
        // For longer uptime, estimate recent activity
        // This is a rough heuristic - you'd want actual session tracking
        snapshot.auth_successes.min(snapshot.request_count / 10)
    };

    recent_auth_rate
}

async fn calculate_db_pool_utilization(state: &AppState) -> f32 {
    // Get database pool stats
    // This is simplified - sqlx doesn't expose detailed pool metrics easily
    // In production, you might use a monitoring-aware connection pool

    // For now, estimate based on active connections vs some reasonable maximum
    let active_connections = global_metrics()
        .active_connections
        .load(std::sync::atomic::Ordering::Relaxed);
    let max_connections = 20; // This should match your pool configuration

    if max_connections > 0 {
        (active_connections.max(0) as f32 / max_connections as f32 * 100.0).min(100.0)
    } else {
        0.0
    }
}

/// Performance monitoring middleware that can be applied to routes
pub async fn performance_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let start = std::time::Instant::now();
    let path = request.uri().path().to_string();

    global_metrics().connection_opened();

    let response = next.run(request).await;

    global_metrics().connection_closed();

    let duration_ms = start.elapsed().as_millis() as u64;
    let is_error = response.status().is_client_error() || response.status().is_server_error();

    global_metrics()
        .record_request(&path, duration_ms, is_error)
        .await;

    // Log slow requests
    if duration_ms > 5000 {
        // 5 seconds
        tracing::warn!("Slow request detected: {} took {}ms", path, duration_ms);
    }

    response
}

/// Background performance monitoring task
pub async fn start_performance_monitor(state: Arc<AppState>) {
    if !state.config.performance_monitoring_enabled {
        return;
    }

    let mut interval = tokio::time::interval(Duration::from_secs(60)); // Every minute

    loop {
        interval.tick().await;

        if let Ok(Json(perf)) = get_performance_metrics(axum::extract::State(state.clone())).await {
            // Log performance summary
            info!(
                "Performance metrics - RPS: {:.2}, Avg RT: {:.2}ms, P95: {:.2}ms, Error Rate: {:.2}%, Active Users: {}, Cache Hit Rate: {:.1}%",
                perf.request_rate_per_second,
                perf.avg_response_time_ms,
                perf.p95_response_time_ms,
                perf.error_rate_percent,
                perf.active_users,
                perf.cache_hit_rate * 100.0
            );

            // Alert on performance issues
            if perf.error_rate_percent > 5.0 {
                tracing::warn!("High error rate detected: {:.2}%", perf.error_rate_percent);
            }

            if perf.p95_response_time_ms > 2000.0 {
                tracing::warn!("High P95 response time: {:.2}ms", perf.p95_response_time_ms);
            }

            if perf.database_pool_utilization > 80.0 {
                tracing::warn!(
                    "High database pool utilization: {:.1}%",
                    perf.database_pool_utilization
                );
            }

            if perf.cache_hit_rate < 0.8 {
                tracing::warn!("Low cache hit rate: {:.1}%", perf.cache_hit_rate * 100.0);
            }
        }
    }
}

/// Get endpoint-specific performance statistics
pub async fn get_endpoint_performance() -> AppResult<Json<Vec<EndpointPerformanceStats>>> {
    let snapshot = global_metrics().get_snapshot().await;

    let mut endpoint_stats = Vec::new();

    for (endpoint, metrics) in snapshot.endpoint_metrics {
        let error_rate = if metrics.total_requests > 0 {
            (metrics.error_count as f32 / metrics.total_requests as f32) * 100.0
        } else {
            0.0
        };

        let requests_per_minute = if snapshot.uptime_seconds > 0 {
            (metrics.total_requests as f64 * 60.0) / snapshot.uptime_seconds as f64
        } else {
            0.0
        };

        endpoint_stats.push(EndpointPerformanceStats {
            endpoint,
            total_requests: metrics.total_requests,
            avg_duration_ms: metrics.avg_duration_ms,
            error_rate,
            requests_per_minute,
            last_accessed: metrics.last_accessed,
        });
    }

    // Sort by total requests descending
    endpoint_stats.sort_by(|a, b| b.total_requests.cmp(&a.total_requests));

    Ok(Json(endpoint_stats))
}

#[derive(serde::Serialize)]
pub struct EndpointPerformanceStats {
    pub endpoint: String,
    pub total_requests: u64,
    pub avg_duration_ms: f64,
    pub error_rate: f32,
    pub requests_per_minute: f64,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}
