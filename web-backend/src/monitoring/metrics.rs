use axum::{extract::State, response::Response, http::HeaderMap, Json};
use std::sync::Arc;
use crate::{error::AppResult, state::AppState, monitoring::{global_metrics, MetricsSnapshot}};

/// Export Prometheus-style metrics
pub async fn prometheus_metrics(State(state): State<Arc<AppState>>) -> AppResult<Response<String>> {
    if !state.config.metrics_enabled {
        return Ok(Response::builder()
            .status(404)
            .body("Metrics disabled".to_string())
            .unwrap());
    }

    let snapshot = global_metrics().get_snapshot().await;
    let prometheus_format = format_prometheus_metrics(&snapshot).await;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/plain; version=0.0.4; charset=utf-8".parse().unwrap());

    Ok(Response::builder()
        .status(200)
        .body(prometheus_format)
        .unwrap())
}

/// Export JSON metrics
pub async fn json_metrics(State(state): State<Arc<AppState>>) -> AppResult<Json<MetricsSnapshot>> {
    if !state.config.metrics_enabled {
        return Err(crate::error::AppError::Forbidden);
    }

    let snapshot = global_metrics().get_snapshot().await;
    Ok(Json(snapshot))
}

async fn format_prometheus_metrics(snapshot: &MetricsSnapshot) -> String {
    let mut output = String::new();

    // Basic counters
    output.push_str(&format!(
        "# HELP http_requests_total Total number of HTTP requests\n# TYPE http_requests_total counter\nhttp_requests_total {}\n\n",
        snapshot.request_count
    ));

    output.push_str(&format!(
        "# HELP http_request_duration_ms_total Total duration of HTTP requests in milliseconds\n# TYPE http_request_duration_ms_total counter\nhttp_request_duration_ms_total {}\n\n",
        snapshot.request_duration_ms
    ));

    output.push_str(&format!(
        "# HELP http_errors_total Total number of HTTP errors\n# TYPE http_errors_total counter\nhttp_errors_total {}\n\n",
        snapshot.error_count
    ));

    // Connection metrics
    output.push_str(&format!(
        "# HELP active_connections Current number of active connections\n# TYPE active_connections gauge\nactive_connections {}\n\n",
        snapshot.active_connections
    ));

    // Database metrics
    output.push_str(&format!(
        "# HELP database_queries_total Total number of database queries\n# TYPE database_queries_total counter\ndatabase_queries_total {}\n\n",
        snapshot.database_queries
    ));

    // Cache metrics
    output.push_str(&format!(
        "# HELP cache_hits_total Total number of cache hits\n# TYPE cache_hits_total counter\ncache_hits_total {}\n\n",
        snapshot.cache_hits
    ));

    output.push_str(&format!(
        "# HELP cache_misses_total Total number of cache misses\n# TYPE cache_misses_total counter\ncache_misses_total {}\n\n",
        snapshot.cache_misses
    ));

    // Authentication metrics
    output.push_str(&format!(
        "# HELP auth_successes_total Total number of successful authentications\n# TYPE auth_successes_total counter\nauth_successes_total {}\n\n",
        snapshot.auth_successes
    ));

    output.push_str(&format!(
        "# HELP auth_failures_total Total number of failed authentications\n# TYPE auth_failures_total counter\nauth_failures_total {}\n\n",
        snapshot.auth_failures
    ));

    // Application-specific metrics
    output.push_str(&format!(
        "# HELP task_generations_total Total number of tasks generated\n# TYPE task_generations_total counter\ntask_generations_total {}\n\n",
        snapshot.task_generations
    ));

    output.push_str(&format!(
        "# HELP learner_updates_total Total number of learner model updates\n# TYPE learner_updates_total counter\nlearner_updates_total {}\n\n",
        snapshot.learner_updates
    ));

    output.push_str(&format!(
        "# HELP session_creations_total Total number of sessions created\n# TYPE session_creations_total counter\nsession_creations_total {}\n\n",
        snapshot.session_creations
    ));

    output.push_str(&format!(
        "# HELP analytics_requests_total Total number of analytics requests\n# TYPE analytics_requests_total counter\nanalytics_requests_total {}\n\n",
        snapshot.analytics_requests
    ));

    output.push_str(&format!(
        "# HELP admin_actions_total Total number of admin actions\n# TYPE admin_actions_total counter\nadmin_actions_total {}\n\n",
        snapshot.admin_actions
    ));

    // Uptime
    output.push_str(&format!(
        "# HELP uptime_seconds Application uptime in seconds\n# TYPE uptime_seconds counter\nuptime_seconds {}\n\n",
        snapshot.uptime_seconds
    ));

    // Endpoint-specific metrics
    output.push_str("# HELP endpoint_requests_total Total requests per endpoint\n# TYPE endpoint_requests_total counter\n");
    for (endpoint, metrics) in &snapshot.endpoint_metrics {
        output.push_str(&format!("endpoint_requests_total{{endpoint=\"{}\"}} {}\n", endpoint, metrics.total_requests));
    }
    output.push('\n');

    output.push_str("# HELP endpoint_duration_ms_total Total duration per endpoint in milliseconds\n# TYPE endpoint_duration_ms_total counter\n");
    for (endpoint, metrics) in &snapshot.endpoint_metrics {
        output.push_str(&format!("endpoint_duration_ms_total{{endpoint=\"{}\"}} {}\n", endpoint, metrics.total_duration_ms));
    }
    output.push('\n');

    output.push_str("# HELP endpoint_errors_total Total errors per endpoint\n# TYPE endpoint_errors_total counter\n");
    for (endpoint, metrics) in &snapshot.endpoint_metrics {
        output.push_str(&format!("endpoint_errors_total{{endpoint=\"{}\"}} {}\n", endpoint, metrics.error_count));
    }
    output.push('\n');

    output.push_str("# HELP endpoint_avg_duration_ms Average response time per endpoint in milliseconds\n# TYPE endpoint_avg_duration_ms gauge\n");
    for (endpoint, metrics) in &snapshot.endpoint_metrics {
        output.push_str(&format!("endpoint_avg_duration_ms{{endpoint=\"{}\"}} {:.2}\n", endpoint, metrics.avg_duration_ms));
    }
    output.push('\n');

    // Calculated metrics
    let cache_hit_rate = if snapshot.cache_hits + snapshot.cache_misses > 0 {
        snapshot.cache_hits as f64 / (snapshot.cache_hits + snapshot.cache_misses) as f64
    } else {
        0.0
    };
    output.push_str(&format!(
        "# HELP cache_hit_rate Cache hit rate (0.0 to 1.0)\n# TYPE cache_hit_rate gauge\ncache_hit_rate {:.4}\n\n",
        cache_hit_rate
    ));

    let error_rate = if snapshot.request_count > 0 {
        snapshot.error_count as f64 / snapshot.request_count as f64
    } else {
        0.0
    };
    output.push_str(&format!(
        "# HELP error_rate HTTP error rate (0.0 to 1.0)\n# TYPE error_rate gauge\nerror_rate {:.4}\n\n",
        error_rate
    ));

    let avg_response_time = if snapshot.request_count > 0 {
        snapshot.request_duration_ms as f64 / snapshot.request_count as f64
    } else {
        0.0
    };
    output.push_str(&format!(
        "# HELP avg_response_time_ms Average response time in milliseconds\n# TYPE avg_response_time_ms gauge\navg_response_time_ms {:.2}\n\n",
        avg_response_time
    ));

    output
}