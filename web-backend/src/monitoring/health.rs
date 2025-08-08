use axum::{extract::State, Json};
use std::sync::Arc;
use std::time::{Instant, Duration};
use chrono::Utc;
use tracing::{warn, error};

use crate::{
    error::{AppResult, AppError}, 
    state::AppState,
    monitoring::{SystemHealth, HealthStatus, ComponentStatus}
};

/// Comprehensive health check endpoint
pub async fn detailed_health_check(State(state): State<Arc<AppState>>) -> AppResult<Json<SystemHealth>> {
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
            warn!("System health degraded - error_rate: {:.2}%, cpu: {:.1}%", error_rate, cpu_usage);
        },
        HealthStatus::Unhealthy => {
            error!("System unhealthy - error_rate: {:.2}%, cpu: {:.1}%, db: {:?}, cache: {:?}", 
                error_rate, cpu_usage, db_status.status, cache_status.status);
        },
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
    if matches!(cache_status.status, HealthStatus::Unhealthy) ||
       error_rate > 5.0 ||
       cpu_usage > 80.0 ||
       db_status.response_time_ms > 1000 {
        return HealthStatus::Degraded;
    }
    
    HealthStatus::Healthy
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
    let mut interval = tokio::time::interval(Duration::from_secs(state.config.health_check_interval_seconds as u64));
    
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
            },
            HealthStatus::Unhealthy => {
                error!("System health check: UNHEALTHY - immediate attention required");
            },
            HealthStatus::Healthy => {
                // Only log healthy status periodically (every 10 checks)
                if health.uptime_seconds % (state.config.health_check_interval_seconds as u64 * 10) < state.config.health_check_interval_seconds as u64 {
                    tracing::info!("System health check: HEALTHY - uptime: {}s, active_connections: {}", 
                        health.uptime_seconds, health.active_connections);
                }
            }
        }
        
        // Could store health history in database or cache for trending analysis
        // store_health_history(&state, &health).await;
    }
}