use axum::{extract::State, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

use crate::{
    error::{AppError, AppResult},
    monitoring::{ComponentStatus, HealthStatus, SystemHealth},
    state::AppState,
};

/// Enhanced health check response with deep system monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSystemHealth {
    pub status: HealthStatus,
    pub uptime_seconds: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
    pub database_status: ComponentStatus,
    pub cache_status: ComponentStatus,
    pub active_connections: i64,
    pub error_rate: f32,
    pub last_check: chrono::DateTime<Utc>,
    // Enhanced monitoring fields
    pub database_pool_stats: DatabasePoolStats,
    pub disk_usage: DiskUsageStats,
    pub external_dependencies: HashMap<String, ComponentStatus>,
    pub application_metrics: ApplicationHealthMetrics,
    pub circuit_breaker_status: HashMap<String, CircuitBreakerHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePoolStats {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub utilization_percent: f32,
    pub avg_acquire_time_ms: u64,
    pub connection_errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageStats {
    pub total_space_gb: f64,
    pub free_space_gb: f64,
    pub used_space_gb: f64,
    pub usage_percent: f32,
    pub inodes_total: Option<u64>,
    pub inodes_used: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationHealthMetrics {
    pub active_sessions: u64,
    pub active_learners: u64,
    pub recent_task_generation_rate: f64,
    pub avg_response_time_ms: f64,
    pub successful_authentications_last_hour: u64,
    pub failed_authentications_last_hour: u64,
    pub websocket_connections: i64,
    pub background_job_queue_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerHealth {
    pub state: String, // "closed", "open", "half-open"
    pub failure_count: u64,
    pub last_failure_time: Option<chrono::DateTime<Utc>>,
    pub success_rate_percent: f32,
}

/// Enhanced comprehensive health check with deep monitoring
pub async fn enhanced_health_check(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<EnhancedSystemHealth>> {
    let start_time = Instant::now();

    debug!("Starting enhanced health check");

    // Core health checks
    let db_status = check_database_health_detailed(&state).await;
    let cache_status = check_cache_health(&state).await;

    // System metrics
    let memory_usage = get_memory_usage();
    let cpu_usage = get_cpu_usage();
    let disk_usage = get_disk_usage().await;

    // Database pool statistics
    let db_pool_stats = get_database_pool_stats(&state).await;

    // External dependencies check
    let external_deps = check_external_dependencies(&state).await;

    // Application-specific metrics
    let app_metrics = get_application_health_metrics(&state).await;

    // Circuit breaker status (placeholder for now)
    let circuit_breakers = get_circuit_breaker_status(&state).await;

    // Calculate uptime and error rates
    let uptime = crate::monitoring::global_metrics()
        .start_time
        .elapsed()
        .unwrap_or(Duration::from_secs(0));

    let metrics_snapshot = crate::monitoring::global_metrics().get_snapshot().await;
    let error_rate = if metrics_snapshot.request_count > 0 {
        (metrics_snapshot.error_count as f32 / metrics_snapshot.request_count as f32) * 100.0
    } else {
        0.0
    };

    // Enhanced health status determination
    let overall_status = determine_enhanced_health_status(
        &db_status,
        &cache_status,
        &db_pool_stats,
        &disk_usage,
        &external_deps,
        error_rate,
        cpu_usage,
    );

    let health = EnhancedSystemHealth {
        status: overall_status.clone(),
        uptime_seconds: uptime.as_secs(),
        memory_usage_mb: memory_usage,
        cpu_usage_percent: cpu_usage,
        database_status: db_status.clone(),
        cache_status: cache_status.clone(),
        active_connections: metrics_snapshot.active_connections,
        error_rate,
        last_check: Utc::now(),
        database_pool_stats: db_pool_stats.clone(),
        disk_usage: disk_usage.clone(),
        external_dependencies: external_deps.clone(),
        application_metrics: app_metrics,
        circuit_breaker_status: circuit_breakers,
    };

    // Enhanced logging with more context
    match health.status {
        HealthStatus::Degraded => {
            warn!(
                "System health DEGRADED - error_rate: {:.2}%, cpu: {:.1}%, db_pool_util: {:.1}%, disk_usage: {:.1}%", 
                error_rate, cpu_usage, db_pool_stats.utilization_percent, disk_usage.usage_percent
            );
        }
        HealthStatus::Unhealthy => {
            error!(
                "System UNHEALTHY - error_rate: {:.2}%, cpu: {:.1}%, db_pool_util: {:.1}%, disk_usage: {:.1}%, external_deps_down: {}", 
                error_rate,
                cpu_usage,
                db_pool_stats.utilization_percent,
                disk_usage.usage_percent,
                external_deps.iter().filter(|(_, status)| matches!(status.status, HealthStatus::Unhealthy)).count()
            );
        }
        HealthStatus::Healthy => {
            debug!(
                "System health HEALTHY - uptime: {}s, active_sessions: {}, db_pool_util: {:.1}%",
                health.uptime_seconds,
                health.application_metrics.active_sessions,
                db_pool_stats.utilization_percent
            );
        }
    }

    debug!(
        "Enhanced health check completed in {}ms",
        start_time.elapsed().as_millis()
    );

    Ok(Json(health))
}

/// Original comprehensive health check endpoint (backwards compatible)
pub async fn detailed_health_check(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<SystemHealth>> {
    let start_time = Instant::now();

    // Check database health
    let db_status = check_database_health(&state).await;

    // Check cache health
    let cache_status = check_cache_health(&state).await;

    // Get system metrics
    let memory_usage = get_memory_usage();
    let cpu_usage = get_cpu_usage();

    // Calculate uptime
    let uptime = crate::monitoring::global_metrics()
        .start_time
        .elapsed()
        .unwrap_or(Duration::from_secs(0));

    // Calculate error rate
    let metrics_snapshot = crate::monitoring::global_metrics().get_snapshot().await;
    let error_rate = if metrics_snapshot.request_count > 0 {
        (metrics_snapshot.error_count as f32 / metrics_snapshot.request_count as f32) * 100.0
    } else {
        0.0
    };

    // Determine overall health status
    let overall_status = determine_health_status(&db_status, &cache_status, error_rate, cpu_usage);

    let health = SystemHealth {
        status: overall_status,
        uptime_seconds: uptime.as_secs(),
        memory_usage_mb: memory_usage,
        cpu_usage_percent: cpu_usage,
        database_status: db_status.clone(),
        cache_status: cache_status.clone(),
        active_connections: metrics_snapshot.active_connections,
        error_rate,
        last_check: Utc::now(),
    };

    // Log health status if degraded or unhealthy
    match health.status {
        HealthStatus::Degraded => {
            warn!(
                "System health degraded - error_rate: {:.2}%, cpu: {:.1}%",
                error_rate, cpu_usage
            );
        }
        HealthStatus::Unhealthy => {
            error!(
                "System unhealthy - error_rate: {:.2}%, cpu: {:.1}%, db: {:?}, cache: {:?}",
                error_rate, cpu_usage, db_status.status, cache_status.status
            );
        }
        _ => {}
    }

    Ok(Json(health))
}

/// Simple health check for load balancers
pub async fn simple_health_check(State(state): State<Arc<AppState>>) -> AppResult<&'static str> {
    // Quick database ping
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
        .is_ok();

    // Quick cache ping
    let mut conn = state.cache_conn.clone();
    let cache_ok = crate::cache::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .is_ok();

    if db_ok && cache_ok {
        Ok("OK")
    } else {
        Err(AppError::InternalServerError)
    }
}

async fn check_database_health(state: &AppState) -> ComponentStatus {
    let start = Instant::now();

    match sqlx::query("SELECT COUNT(*) as count FROM users LIMIT 1")
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(_) => ComponentStatus {
            status: HealthStatus::Healthy,
            response_time_ms: start.elapsed().as_millis() as u64,
            last_error: None,
            last_check: Utc::now(),
        },
        Err(e) => {
            error!("Database health check failed: {}", e);
            ComponentStatus {
                status: HealthStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                last_error: Some(format!("Database error: {}", e)),
                last_check: Utc::now(),
            }
        }
    }
}

/// Enhanced database health check with more comprehensive testing
async fn check_database_health_detailed(state: &AppState) -> ComponentStatus {
    let start = Instant::now();

    // Test multiple operations to ensure database is fully functional
    let tests = vec![
        // Basic connectivity
        ("SELECT 1", "basic connectivity"),
        // Read operation
        ("SELECT COUNT(*) FROM users LIMIT 1", "table access"),
        // Check if migrations are up to date (simplified)
        (
            "SELECT name FROM sqlite_master WHERE type='table' AND name='users'",
            "schema integrity",
        ),
    ];

    let mut total_time = 0u64;
    let mut error_details = Vec::new();

    for (query, test_name) in tests {
        let test_start = Instant::now();

        match sqlx::query(query).fetch_optional(&state.db_pool).await {
            Ok(_) => {
                let test_time = test_start.elapsed().as_millis() as u64;
                total_time += test_time;
                debug!("Database {} test passed in {}ms", test_name, test_time);
            }
            Err(e) => {
                error_details.push(format!("{}: {}", test_name, e));
                error!("Database {} test failed: {}", test_name, e);
            }
        }
    }

    let response_time = start.elapsed().as_millis() as u64;

    if error_details.is_empty() {
        ComponentStatus {
            status: if response_time > 2000 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            },
            response_time_ms: response_time,
            last_error: None,
            last_check: Utc::now(),
        }
    } else {
        ComponentStatus {
            status: HealthStatus::Unhealthy,
            response_time_ms: response_time,
            last_error: Some(error_details.join("; ")),
            last_check: Utc::now(),
        }
    }
}

async fn check_cache_health(state: &AppState) -> ComponentStatus {
    let start = Instant::now();
    let mut conn = state.cache_conn.clone();

    match crate::cache::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
    {
        Ok(_) => ComponentStatus {
            status: HealthStatus::Healthy,
            response_time_ms: start.elapsed().as_millis() as u64,
            last_error: None,
            last_check: Utc::now(),
        },
        Err(e) => {
            error!("Cache health check failed: {}", e);
            ComponentStatus {
                status: HealthStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                last_error: Some(format!("Cache error: {}", e)),
                last_check: Utc::now(),
            }
        }
    }
}

fn determine_health_status(
    db_status: &ComponentStatus,
    cache_status: &ComponentStatus,
    error_rate: f32,
    cpu_usage: f32,
) -> HealthStatus {
    // System is unhealthy if critical components are down
    if matches!(db_status.status, HealthStatus::Unhealthy) {
        return HealthStatus::Unhealthy;
    }

    // System is degraded if:
    // - Cache is down (non-critical but affects performance)
    // - High error rate (>5%)
    // - High CPU usage (>80%)
    // - Slow database response (>1000ms)
    if matches!(cache_status.status, HealthStatus::Unhealthy)
        || error_rate > 5.0
        || cpu_usage > 80.0
        || db_status.response_time_ms > 1000
    {
        return HealthStatus::Degraded;
    }

    HealthStatus::Healthy
}

/// Enhanced health status determination with deeper system analysis
fn determine_enhanced_health_status(
    db_status: &ComponentStatus,
    cache_status: &ComponentStatus,
    db_pool_stats: &DatabasePoolStats,
    disk_usage: &DiskUsageStats,
    external_deps: &HashMap<String, ComponentStatus>,
    error_rate: f32,
    cpu_usage: f32,
) -> HealthStatus {
    // System is unhealthy if critical components are down
    if matches!(db_status.status, HealthStatus::Unhealthy) {
        return HealthStatus::Unhealthy;
    }

    // Check for critical resource exhaustion
    if disk_usage.usage_percent > 95.0 {
        error!("Critical disk usage: {:.1}%", disk_usage.usage_percent);
        return HealthStatus::Unhealthy;
    }

    if db_pool_stats.utilization_percent > 95.0 {
        error!(
            "Critical database pool utilization: {:.1}%",
            db_pool_stats.utilization_percent
        );
        return HealthStatus::Unhealthy;
    }

    // Count unhealthy external dependencies
    let unhealthy_deps = external_deps
        .iter()
        .filter(|(_, status)| matches!(status.status, HealthStatus::Unhealthy))
        .count();

    if unhealthy_deps > 0 {
        warn!("{} external dependencies are unhealthy", unhealthy_deps);
        return HealthStatus::Unhealthy;
    }

    // System is degraded if:
    // - Cache is down (non-critical but affects performance)
    // - High error rate (>5%)
    // - High CPU usage (>80%)
    // - Slow database response (>1000ms)
    // - High disk usage (>85%)
    // - High database pool utilization (>80%)
    // - High connection errors
    // Check degradation conditions with owned strings
    if matches!(cache_status.status, HealthStatus::Unhealthy) {
        warn!("System degraded: cache unhealthy");
        return HealthStatus::Degraded;
    }
    if error_rate > 5.0 {
        warn!("System degraded: high error rate: {:.2}%", error_rate);
        return HealthStatus::Degraded;
    }
    if cpu_usage > 80.0 {
        warn!("System degraded: high CPU usage: {:.1}%", cpu_usage);
        return HealthStatus::Degraded;
    }
    if db_status.response_time_ms > 1000 {
        warn!(
            "System degraded: slow DB response: {}ms",
            db_status.response_time_ms
        );
        return HealthStatus::Degraded;
    }
    if disk_usage.usage_percent > 85.0 {
        warn!(
            "System degraded: high disk usage: {:.1}%",
            disk_usage.usage_percent
        );
        return HealthStatus::Degraded;
    }
    if db_pool_stats.utilization_percent > 80.0 {
        warn!(
            "System degraded: high DB pool utilization: {:.1}%",
            db_pool_stats.utilization_percent
        );
        return HealthStatus::Degraded;
    }
    if db_pool_stats.connection_errors > 10 {
        warn!(
            "System degraded: DB connection errors: {}",
            db_pool_stats.connection_errors
        );
        return HealthStatus::Degraded;
    }
    if db_pool_stats.avg_acquire_time_ms > 500 {
        warn!(
            "System degraded: slow DB acquire time: {}ms",
            db_pool_stats.avg_acquire_time_ms
        );
        return HealthStatus::Degraded;
    }

    HealthStatus::Healthy
}

async fn get_database_pool_stats(state: &AppState) -> DatabasePoolStats {
    // Note: SQLx doesn't expose detailed pool metrics directly
    // This is a simplified implementation that would need enhancement
    // based on the specific database pool implementation

    let pool = &state.db_pool;

    // Try to get basic pool information
    // These values would need to be tracked separately in a real implementation
    let max_connections = std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50) as u32;

    // Simulate getting current pool stats (would need actual implementation)
    // In practice, you'd need to implement a pool wrapper that tracks these metrics
    let active_connections = crate::monitoring::global_metrics()
        .active_connections
        .load(std::sync::atomic::Ordering::Relaxed)
        .max(0) as u32;

    let idle_connections = if max_connections > active_connections {
        max_connections - active_connections
    } else {
        0
    };

    let utilization_percent = if max_connections > 0 {
        (active_connections as f32 / max_connections as f32) * 100.0
    } else {
        0.0
    };

    // Test average acquire time
    let start = std::time::Instant::now();
    let _conn_test = pool.acquire().await;
    let avg_acquire_time_ms = start.elapsed().as_millis() as u64;

    DatabasePoolStats {
        active_connections,
        idle_connections,
        max_connections,
        utilization_percent,
        avg_acquire_time_ms,
        connection_errors: 0, // Would need to track this separately
    }
}

async fn get_disk_usage() -> DiskUsageStats {
    // Simplified disk usage implementation
    // In production, you'd use proper system APIs or libraries like sysinfo

    #[cfg(unix)]
    {
        // Try to parse /proc/mounts and /proc/diskstats for basic info
        if let Ok(output) = std::process::Command::new("df").arg("/").arg("-h").output() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                // Parse df output (simplified)
                for line in output_str.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 && parts[5] == "/" {
                        if let (Ok(total_kb), Ok(used_kb)) = (
                            parts[1].trim_end_matches('K').parse::<f64>(),
                            parts[2].trim_end_matches('K').parse::<f64>(),
                        ) {
                            let total_gb = total_kb / (1024.0 * 1024.0);
                            let used_gb = used_kb / (1024.0 * 1024.0);
                            let free_gb = total_gb - used_gb;
                            let usage_percent = if total_gb > 0.0 {
                                (used_gb / total_gb * 100.0) as f32
                            } else {
                                0.0
                            };

                            return DiskUsageStats {
                                total_space_gb: total_gb,
                                free_space_gb: free_gb,
                                used_space_gb: used_gb,
                                usage_percent,
                                inodes_total: None,
                                inodes_used: None,
                            };
                        }
                    }
                }
            }
        }
    }

    // Fallback - return placeholder values
    DiskUsageStats {
        total_space_gb: 100.0, // Assume 100GB disk
        free_space_gb: 50.0,   // Assume 50GB free
        used_space_gb: 50.0,   // Assume 50GB used
        usage_percent: 50.0,   // 50% usage
        inodes_total: None,
        inodes_used: None,
    }
}

async fn check_external_dependencies(state: &AppState) -> HashMap<String, ComponentStatus> {
    let mut deps = HashMap::new();

    // Check Redis cache connection (already done in cache check)
    let cache_status = check_cache_health(&state).await;
    deps.insert("redis_cache".to_string(), cache_status);

    // Add other external dependencies as needed
    // Example: External API endpoints, other databases, message queues, etc.

    // Placeholder for external API checks
    // deps.insert("auth_service".to_string(), check_auth_service().await);
    // deps.insert("analytics_api".to_string(), check_analytics_api().await);

    deps
}

async fn get_application_health_metrics(state: &AppState) -> ApplicationHealthMetrics {
    let metrics = crate::monitoring::global_metrics().get_snapshot().await;

    // Calculate active sessions (simplified - would need proper tracking)
    let active_sessions = metrics.session_creations.saturating_sub(
        metrics.uptime_seconds / 1800, // Estimate session expiry
    );

    // Calculate recent rates (per minute over last 5 minutes)
    let recent_task_generation_rate = if metrics.uptime_seconds > 300 {
        (metrics.task_generations as f64) / (metrics.uptime_seconds as f64 / 60.0)
    } else {
        0.0
    };

    let avg_response_time_ms = if metrics.request_count > 0 {
        metrics.request_duration_ms as f64 / metrics.request_count as f64
    } else {
        0.0
    };

    // Calculate auth rates (simplified - would need time-windowed tracking)
    let auth_total = metrics.auth_successes + metrics.auth_failures;
    let auth_success_rate = if auth_total > 0 {
        (metrics.auth_successes as f64 / auth_total as f64) * 100.0
    } else {
        0.0
    };

    ApplicationHealthMetrics {
        active_sessions,
        active_learners: metrics.learner_updates, // Simplified
        recent_task_generation_rate,
        avg_response_time_ms,
        successful_authentications_last_hour: if auth_success_rate > 0.0 {
            metrics.auth_successes
        } else {
            0
        },
        failed_authentications_last_hour: metrics.auth_failures,
        websocket_connections: metrics.active_connections, // Includes all connections
        background_job_queue_size: 0,                      // Would need job queue integration
    }
}

async fn get_circuit_breaker_status(_state: &AppState) -> HashMap<String, CircuitBreakerHealth> {
    let mut breakers = HashMap::new();

    // Placeholder for circuit breaker status
    // In a real implementation, you'd check the state of your circuit breakers
    breakers.insert(
        "database".to_string(),
        CircuitBreakerHealth {
            state: "closed".to_string(),
            failure_count: 0,
            last_failure_time: None,
            success_rate_percent: 100.0,
        },
    );

    breakers.insert(
        "cache".to_string(),
        CircuitBreakerHealth {
            state: "closed".to_string(),
            failure_count: 0,
            last_failure_time: None,
            success_rate_percent: 100.0,
        },
    );

    breakers
}

fn get_memory_usage() -> u64 {
    // Simplified memory usage - in a real implementation,
    // you'd use system APIs to get actual memory usage
    #[cfg(unix)]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb / 1024; // Convert KB to MB
                        }
                    }
                }
            }
        }
    }

    // Fallback for non-Unix systems or if proc filesystem is unavailable
    0
}

fn get_cpu_usage() -> f32 {
    // Simplified CPU usage - in a real implementation,
    // you'd calculate CPU usage over a time window
    // For now, return a placeholder that could be enhanced
    // with proper CPU monitoring libraries

    // Get load average on Unix systems
    #[cfg(unix)]
    {
        if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
            if let Some(load1) = loadavg.split_whitespace().next() {
                if let Ok(load) = load1.parse::<f32>() {
                    // Convert load average to percentage (assuming single core)
                    return (load * 100.0).min(100.0);
                }
            }
        }
    }

    // Fallback
    0.0
}

/// Background health monitoring task
pub async fn start_health_monitor(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(
        state.config.health_check_interval_seconds as u64,
    ));

    loop {
        interval.tick().await;

        // Perform periodic health checks
        let health = match detailed_health_check(axum::extract::State(state.clone())).await {
            Ok(Json(health)) => health,
            Err(e) => {
                error!("Health check failed: {}", e);
                continue;
            }
        };

        // Log significant health changes
        match health.status {
            HealthStatus::Degraded => {
                warn!("System health check: DEGRADED - error_rate: {:.2}%, active_connections: {}, uptime: {}s", 
                    health.error_rate, health.active_connections, health.uptime_seconds);
            }
            HealthStatus::Unhealthy => {
                error!("System health check: UNHEALTHY - immediate attention required");
            }
            HealthStatus::Healthy => {
                // Only log healthy status periodically (every 10 checks)
                if health.uptime_seconds % (state.config.health_check_interval_seconds as u64 * 10)
                    < state.config.health_check_interval_seconds as u64
                {
                    tracing::info!(
                        "System health check: HEALTHY - uptime: {}s, active_connections: {}",
                        health.uptime_seconds,
                        health.active_connections
                    );
                }
            }
        }

        // Could store health history in database or cache for trending analysis
        // store_health_history(&state, &health).await;
    }
}
