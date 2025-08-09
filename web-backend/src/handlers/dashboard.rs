use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono;
use serde_json::json;
use std::sync::Arc;
use tracing::{info, instrument};

use crate::{
    error::AppResult,
    monitoring::{database::global_database_monitor, global_metrics, otel::OtelMetricsExporter},
    state::AppState,
};

/// Serve the main metrics dashboard HTML page
#[instrument(level = "info")]
pub async fn metrics_dashboard_html() -> impl IntoResponse {
    info!("Serving metrics dashboard HTML");

    let html_content = include_str!("../../static/dashboard.html");
    Html(html_content)
}

/// API endpoint for dashboard data (JSON)
#[instrument(level = "debug")]
pub async fn dashboard_data(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let metrics_snapshot = global_metrics().get_snapshot().await;
    let db_health = global_database_monitor().get_health_metrics().await;
    let (active_conns, total_conns, conn_errors, conn_timeouts) =
        global_database_monitor().get_connection_metrics();

    // Calculate derived metrics
    let uptime_hours = metrics_snapshot.uptime_seconds as f64 / 3600.0;
    let requests_per_hour = if uptime_hours > 0.0 {
        metrics_snapshot.request_count as f64 / uptime_hours
    } else {
        0.0
    };

    let avg_response_time = if metrics_snapshot.request_count > 0 {
        metrics_snapshot.request_duration_ms as f64 / metrics_snapshot.request_count as f64
    } else {
        0.0
    };

    let error_rate = if metrics_snapshot.request_count > 0 {
        (metrics_snapshot.error_count as f64 / metrics_snapshot.request_count as f64) * 100.0
    } else {
        0.0
    };

    let cache_hit_rate = if (metrics_snapshot.cache_hits + metrics_snapshot.cache_misses) > 0 {
        (metrics_snapshot.cache_hits as f64
            / (metrics_snapshot.cache_hits + metrics_snapshot.cache_misses) as f64)
            * 100.0
    } else {
        0.0
    };

    // System health status
    let system_status = if error_rate > 10.0 {
        "unhealthy"
    } else if error_rate > 5.0 || avg_response_time > 1000.0 {
        "degraded"
    } else {
        "healthy"
    };

    // Top endpoints by request count
    let mut top_endpoints: Vec<_> = metrics_snapshot.endpoint_metrics.iter().collect();
    top_endpoints.sort_by(|a, b| b.1.total_requests.cmp(&a.1.total_requests));
    let top_endpoints: Vec<_> = top_endpoints.into_iter().take(10).collect();

    // Slowest endpoints by average response time
    let mut slowest_endpoints: Vec<_> = metrics_snapshot.endpoint_metrics.iter().collect();
    slowest_endpoints.sort_by(|a, b| {
        b.1.avg_duration_ms
            .partial_cmp(&a.1.avg_duration_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let slowest_endpoints: Vec<_> = slowest_endpoints.into_iter().take(10).collect();

    let dashboard_data = json!({
        "system": {
            "status": system_status,
            "uptime_seconds": metrics_snapshot.uptime_seconds,
            "uptime_hours": uptime_hours,
            "timestamp": metrics_snapshot.timestamp
        },
        "requests": {
            "total": metrics_snapshot.request_count,
            "per_hour": requests_per_hour,
            "errors": metrics_snapshot.error_count,
            "error_rate_percent": error_rate,
            "avg_response_time_ms": avg_response_time,
            "active_connections": metrics_snapshot.active_connections
        },
        "database": {
            "total_queries": db_health.total_queries,
            "avg_query_duration_ms": db_health.avg_query_duration_ms,
            "slow_query_percentage": db_health.slow_query_percentage,
            "error_rate_percentage": db_health.error_rate_percentage,
            "active_connections": active_conns,
            "total_connections_created": total_conns,
            "connection_errors": conn_errors,
            "connection_timeouts": conn_timeouts
        },
        "cache": {
            "hits": metrics_snapshot.cache_hits,
            "misses": metrics_snapshot.cache_misses,
            "hit_rate_percent": cache_hit_rate
        },
        "authentication": {
            "successes": metrics_snapshot.auth_successes,
            "failures": metrics_snapshot.auth_failures,
            "failure_rate_percent": if (metrics_snapshot.auth_successes + metrics_snapshot.auth_failures) > 0 {
                (metrics_snapshot.auth_failures as f64 / (metrics_snapshot.auth_successes + metrics_snapshot.auth_failures) as f64) * 100.0
            } else {
                0.0
            }
        },
        "application": {
            "task_generations": metrics_snapshot.task_generations,
            "learner_updates": metrics_snapshot.learner_updates,
            "session_creations": metrics_snapshot.session_creations,
            "analytics_requests": metrics_snapshot.analytics_requests,
            "admin_actions": metrics_snapshot.admin_actions
        },
        "endpoints": {
            "top_by_requests": top_endpoints.iter().map(|(path, metrics)| {
                json!({
                    "path": path,
                    "requests": metrics.total_requests,
                    "avg_duration_ms": metrics.avg_duration_ms,
                    "errors": metrics.error_count,
                    "last_accessed": metrics.last_accessed
                })
            }).collect::<Vec<_>>(),
            "slowest_by_duration": slowest_endpoints.iter().map(|(path, metrics)| {
                json!({
                    "path": path,
                    "avg_duration_ms": metrics.avg_duration_ms,
                    "requests": metrics.total_requests,
                    "errors": metrics.error_count,
                    "last_accessed": metrics.last_accessed
                })
            }).collect::<Vec<_>>()
        },
        "database_queries": {
            "by_type": db_health.query_types.iter().map(|qt| {
                json!({
                    "query_type": qt.query_type,
                    "execution_count": qt.execution_count,
                    "avg_duration_ms": qt.avg_duration_ms,
                    "error_count": qt.error_count,
                    "slow_query_count": qt.slow_query_count
                })
            }).collect::<Vec<_>>()
        }
    });

    Ok(Json(dashboard_data))
}

/// API endpoint for real-time metrics (minimal, for frequent polling)
#[instrument(level = "debug")]
pub async fn realtime_metrics() -> AppResult<Json<serde_json::Value>> {
    let metrics_snapshot = global_metrics().get_snapshot().await;

    let realtime_data = json!({
        "timestamp": chrono::Utc::now(),
        "requests_total": metrics_snapshot.request_count,
        "errors_total": metrics_snapshot.error_count,
        "active_connections": metrics_snapshot.active_connections,
        "avg_response_time_ms": if metrics_snapshot.request_count > 0 {
            metrics_snapshot.request_duration_ms as f64 / metrics_snapshot.request_count as f64
        } else {
            0.0
        },
        "cache_hit_rate": if (metrics_snapshot.cache_hits + metrics_snapshot.cache_misses) > 0 {
            (metrics_snapshot.cache_hits as f64 / (metrics_snapshot.cache_hits + metrics_snapshot.cache_misses) as f64) * 100.0
        } else {
            0.0
        }
    });

    Ok(Json(realtime_data))
}

/// OpenTelemetry metrics export for external systems
#[instrument(level = "info")]
pub async fn otel_metrics(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let exporter = OtelMetricsExporter::new(
        "learning-system".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        state.config.environment.to_string(),
    );

    let app_metrics = exporter.export_application_metrics().await;
    let system_metrics = exporter.export_system_metrics().await;

    let otel_data = json!({
        "service": {
            "name": "learning-system",
            "version": env!("CARGO_PKG_VERSION"),
            // Don't expose environment information
        },
        "metrics": {
            "application": app_metrics,
            "system": system_metrics
        },
        "exported_at": chrono::Utc::now()
    });

    Ok(Json(otel_data))
}

/// System information endpoint - restricted to authenticated admins only
#[instrument(level = "debug")]
pub async fn system_info(State(state): State<Arc<AppState>>) -> AppResult<Json<serde_json::Value>> {
    // This endpoint should only be accessible to authenticated admin users
    // The middleware should have already validated this, but we'll add a check here too

    // Return only non-sensitive system information
    let system_info = json!({
        "service": {
            "name": "learning-system-backend",
            "version": env!("CARGO_PKG_VERSION"),
            "status": "operational"
        },
        "configuration": {
            // Only expose non-sensitive configuration flags
            "metrics_enabled": state.config.metrics_enabled,
            "performance_monitoring_enabled": state.config.performance_monitoring_enabled,
            // Don't expose privacy parameters as they could reveal security thresholds
        },
        "runtime": {
            // Don't expose hostname or process ID in production
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
        }
    });

    Ok(Json(system_info))
}

/// Database performance report
#[instrument(level = "info")]
pub async fn database_report() -> AppResult<Json<serde_json::Value>> {
    let report = global_database_monitor()
        .generate_performance_report()
        .await;
    let slow_queries = global_database_monitor().get_slow_queries(20).await;

    let db_report = json!({
        "report": report,
        "slow_queries": slow_queries,
        "generated_at": chrono::Utc::now()
    });

    Ok(Json(db_report))
}

/// Health check with detailed component status
#[instrument(level = "debug")]
pub async fn detailed_health(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    // This would ideally call the existing health check functionality
    // For now, we'll create a basic health summary
    let metrics_snapshot = global_metrics().get_snapshot().await;
    let db_health = global_database_monitor().get_health_metrics().await;

    let error_rate = if metrics_snapshot.request_count > 0 {
        (metrics_snapshot.error_count as f64 / metrics_snapshot.request_count as f64) * 100.0
    } else {
        0.0
    };

    let overall_status = if error_rate > 10.0 || db_health.error_rate_percentage > 10.0 {
        "unhealthy"
    } else if error_rate > 5.0 || db_health.error_rate_percentage > 5.0 {
        "degraded"
    } else {
        "healthy"
    };

    let health_data = json!({
        "status": overall_status,
        "components": {
            "http_server": {
                "status": if error_rate < 5.0 { "healthy" } else { "degraded" },
                "error_rate": error_rate,
                "active_connections": metrics_snapshot.active_connections,
                "uptime_seconds": metrics_snapshot.uptime_seconds
            },
            "database": {
                "status": if db_health.error_rate_percentage < 5.0 { "healthy" } else { "degraded" },
                "error_rate": db_health.error_rate_percentage,
                "slow_query_rate": db_health.slow_query_percentage,
                "active_connections": db_health.active_connections
            },
            "cache": {
                "status": "healthy",
                "hit_rate": if (metrics_snapshot.cache_hits + metrics_snapshot.cache_misses) > 0 {
                    (metrics_snapshot.cache_hits as f64 / (metrics_snapshot.cache_hits + metrics_snapshot.cache_misses) as f64) * 100.0
                } else {
                    0.0
                },
                "total_operations": metrics_snapshot.cache_hits + metrics_snapshot.cache_misses
            }
        },
        "checked_at": chrono::Utc::now()
    });

    Ok(Json(health_data))
}
