# Monitoring Directory - Integration Guide

## Overview

The monitoring directory provides comprehensive observability infrastructure for the web-backend, including metrics collection, health checks, performance tracking, business intelligence monitoring, and OpenTelemetry integration. This directory implements a production-ready monitoring solution that scales with your application.

## How Core and Apps Should Use This Functionality

### Core Integration Patterns

#### Automatic Metrics Collection
```rust
// The core automatically integrates with global metrics
use crate::monitoring::{global_metrics, RequestTimer};

// For HTTP handlers - automatically timed
let timer = RequestTimer::new("api/experiment/create".to_string());
let result = create_experiment(payload).await;
timer.finish(result.is_err()).await;
```

#### Health Check Integration
```rust
// Core services should register health checks
use crate::monitoring::health::{register_health_check, HealthCheckFunction};

register_health_check("database", Arc::new(check_database_health)).await;
register_health_check("cache", Arc::new(check_cache_health)).await;
```

#### Custom Business Metrics
```rust
// Record domain-specific events
global_metrics().record_task_generation();
global_metrics().record_learner_update();
global_metrics().record_session_creation();
```

### Application Usage Patterns

#### Performance Monitoring
```rust
// Track critical path performance
let metrics = global_metrics().get_snapshot().await;
let p95_response_time = calculate_p95(&metrics.endpoint_metrics["api/critical"]);

if p95_response_time > 500.0 {
    alert_performance_degradation().await;
}
```

#### Business Intelligence
```rust
// Monitor research-specific metrics
use crate::monitoring::business::BusinessMetrics;

let business_metrics = BusinessMetrics::collect().await;
log_daily_active_researchers(&business_metrics.active_researchers).await;
```

## State Machines and Lifecycle Patterns

### Health Check Lifecycle
```
System Start → Health Check Registration → Periodic Checks → Status Updates → Alert Generation
```

### Metrics Collection Lifecycle
```
Request Start → Timer Creation → Request Processing → Timer Completion → Metrics Update → Aggregation
```

### OpenTelemetry Tracing Lifecycle
```
Span Creation → Context Propagation → Child Span Creation → Span Completion → Export to Collector
```

### Circuit Breaker Integration
```
Normal Operation → Error Threshold → Circuit Open → Recovery Period → Health Check → Circuit Closed
```

## Integration Patterns and Best Practices

### Middleware Integration
```rust
// Automatic request tracking via middleware
pub async fn metrics_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let timer = RequestTimer::new(request.uri().path().to_string());
    let response = next.run(request).await;
    let is_error = response.status().is_server_error();
    timer.finish(is_error).await;
    response
}
```

### Database Monitoring
```rust
// Automatic database query tracking
use crate::monitoring::database::DatabaseMetrics;

impl DatabaseMetrics {
    pub async fn record_query(&self, query: &str, duration: Duration) {
        global_metrics().record_db_query_with_timing(duration.as_millis() as u64).await;
        self.slow_query_detector.check_query(query, duration).await;
    }
}
```

### Error Tracking Integration
```rust
// Link errors with monitoring
pub async fn handle_error(error: &AppError) {
    global_metrics().error_count.fetch_add(1, Ordering::Relaxed);
    
    match error {
        AppError::Database(_) => record_database_error().await,
        AppError::Auth(_) => record_auth_error().await,
        _ => record_general_error().await,
    }
}
```

## Architectural Decisions and Constraints

### Thread-Safe Metrics Collection
- Uses atomic counters for high-frequency metrics
- RwLock for complex data structures with read-heavy access
- Histogram-based percentile calculation for accurate response time tracking

### Memory Management
- Bounded histogram buckets to prevent memory leaks
- Periodic metrics aggregation and cleanup
- Configurable retention periods for detailed metrics

### Performance Considerations
- Zero-allocation hot paths for counter updates
- Async-friendly locking strategies
- Batched exports to external systems

### Extensibility Framework
- Plugin-based health check registration
- Custom metric types via trait implementation
- Configurable export formats (Prometheus, JSON, OpenTelemetry)

## Security and Performance Considerations

### Security Measures
- Metrics endpoints protected by authentication
- Sensitive data scrubbing in error messages
- Rate limiting on metrics collection endpoints
- Access control for different metric visibility levels

### Performance Optimization
- Lock-free atomic operations for hot paths
- Histogram sampling for high-cardinality metrics
- Background aggregation to minimize request impact
- Connection pooling for external metric collectors

### Resource Management
- Configurable metric retention policies
- Memory bounds for histogram buckets
- CPU usage limits for complex calculations
- Disk space management for persistent metrics

## Common Usage Patterns and Examples

### Basic Request Monitoring
```rust
#[tracing::instrument]
pub async fn create_experiment(payload: CreateExperimentRequest) -> AppResult<Experiment> {
    let timer = RequestTimer::new("experiment_creation".to_string());
    
    let result = experiment_service.create(payload).await;
    
    match &result {
        Ok(_) => global_metrics().record_task_generation(),
        Err(e) => tracing::error!("Experiment creation failed: {}", e),
    }
    
    timer.finish(result.is_err()).await;
    result
}
```

### Custom Health Checks
```rust
async fn check_external_api_health() -> ComponentStatus {
    let start = Instant::now();
    let result = external_api_client.health_check().await;
    let response_time = start.elapsed().as_millis() as u64;
    
    match result {
        Ok(_) => ComponentStatus {
            status: HealthStatus::Healthy,
            response_time_ms: response_time,
            last_error: None,
            last_check: Utc::now(),
        },
        Err(e) => ComponentStatus {
            status: HealthStatus::Unhealthy,
            response_time_ms: response_time,
            last_error: Some(e.to_string()),
            last_check: Utc::now(),
        }
    }
}
```

### Business Metrics Dashboard
```rust
pub async fn get_research_dashboard() -> AppResult<ResearchMetrics> {
    let business_metrics = BusinessMetrics::collect().await;
    let performance_metrics = PerformanceMetrics::calculate().await;
    
    Ok(ResearchMetrics {
        active_experiments: business_metrics.active_experiments,
        daily_active_learners: business_metrics.daily_active_learners,
        avg_session_duration: performance_metrics.avg_session_duration_minutes,
        completion_rate: business_metrics.completion_rate_percent,
        p95_response_time: performance_metrics.p95_response_time_ms,
    })
}
```

### Alert Integration
```rust
pub async fn check_system_health() -> AppResult<()> {
    let health = EnhancedSystemHealth::collect().await;
    
    if health.database_pool_stats.utilization_percent > 80.0 {
        alert_service.send_alert(Alert::DatabasePoolHigh).await;
    }
    
    if health.error_rate > 5.0 {
        alert_service.send_alert(Alert::HighErrorRate).await;
    }
    
    Ok(())
}
```

## Configuration Examples

### Production Configuration
```toml
[monitoring]
metrics_enabled = true
health_check_interval_seconds = 30
prometheus_endpoint = "/metrics"
detailed_metrics = true
histogram_buckets = [1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000]

[monitoring.business]
session_timeout_minutes = 30
active_learner_threshold_hours = 24
completion_rate_window_days = 7

[monitoring.performance]
slow_query_threshold_ms = 1000
alert_threshold_p95_ms = 2000
memory_alert_threshold_mb = 1024
```

This monitoring system provides comprehensive observability while maintaining high performance and scalability. It integrates seamlessly with the rest of the application architecture and provides actionable insights for both operational and business concerns.