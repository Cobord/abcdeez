use std::collections::HashMap;
use std::time::SystemTime;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// OpenTelemetry-compatible metrics exporter
#[derive(Debug, Clone)]
pub struct OtelMetricsExporter {
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
    pub custom_attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelMetric {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, String>,
    pub resource: OtelResource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelResource {
    pub service_name: String,
    pub service_version: String,
    pub service_instance_id: String,
    pub deployment_environment: String,
    pub host_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration_nanos: Option<u64>,
    pub status: SpanStatus,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
    pub resource: OtelResource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: SystemTime,
    pub attributes: HashMap<String, String>,
}

impl OtelMetricsExporter {
    pub fn new(service_name: String, service_version: String, environment: String) -> Self {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let mut custom_attributes = HashMap::new();
        custom_attributes.insert("host.name".to_string(), hostname);
        custom_attributes.insert("process.pid".to_string(), std::process::id().to_string());
        
        Self {
            service_name,
            service_version,
            environment,
            custom_attributes,
        }
    }

    /// Export application metrics in OpenTelemetry format
    pub async fn export_application_metrics(&self) -> Vec<OtelMetric> {
        let metrics_snapshot = crate::monitoring::global_metrics().get_snapshot().await;
        let resource = self.create_resource();
        let now = Utc::now();
        
        let mut otel_metrics = Vec::new();

        // Request metrics
        otel_metrics.push(OtelMetric {
            name: "http_requests_total".to_string(),
            description: "Total number of HTTP requests".to_string(),
            unit: "1".to_string(),
            metric_type: MetricType::Counter,
            value: metrics_snapshot.request_count as f64,
            timestamp: now,
            attributes: HashMap::new(),
            resource: resource.clone(),
        });

        // Request duration
        if metrics_snapshot.request_count > 0 {
            otel_metrics.push(OtelMetric {
                name: "http_request_duration_ms".to_string(),
                description: "Average HTTP request duration in milliseconds".to_string(),
                unit: "ms".to_string(),
                metric_type: MetricType::Gauge,
                value: metrics_snapshot.request_duration_ms as f64 / metrics_snapshot.request_count as f64,
                timestamp: now,
                attributes: HashMap::new(),
                resource: resource.clone(),
            });
        }

        // Error rate
        otel_metrics.push(OtelMetric {
            name: "http_requests_errors_total".to_string(),
            description: "Total number of HTTP request errors".to_string(),
            unit: "1".to_string(),
            metric_type: MetricType::Counter,
            value: metrics_snapshot.error_count as f64,
            timestamp: now,
            attributes: HashMap::new(),
            resource: resource.clone(),
        });

        // Active connections
        otel_metrics.push(OtelMetric {
            name: "http_connections_active".to_string(),
            description: "Number of active HTTP connections".to_string(),
            unit: "1".to_string(),
            metric_type: MetricType::Gauge,
            value: metrics_snapshot.active_connections as f64,
            timestamp: now,
            attributes: HashMap::new(),
            resource: resource.clone(),
        });

        // Database metrics
        otel_metrics.push(OtelMetric {
            name: "database_queries_total".to_string(),
            description: "Total number of database queries".to_string(),
            unit: "1".to_string(),
            metric_type: MetricType::Counter,
            value: metrics_snapshot.database_queries as f64,
            timestamp: now,
            attributes: HashMap::new(),
            resource: resource.clone(),
        });

        // Cache metrics
        let cache_total = metrics_snapshot.cache_hits + metrics_snapshot.cache_misses;
        if cache_total > 0 {
            let hit_rate = metrics_snapshot.cache_hits as f64 / cache_total as f64;
            otel_metrics.push(OtelMetric {
                name: "cache_hit_rate".to_string(),
                description: "Cache hit rate percentage".to_string(),
                unit: "1".to_string(),
                metric_type: MetricType::Gauge,
                value: hit_rate,
                timestamp: now,
                attributes: HashMap::new(),
                resource: resource.clone(),
            });
        }

        // Application-specific metrics
        otel_metrics.push(OtelMetric {
            name: "learning_tasks_generated_total".to_string(),
            description: "Total number of learning tasks generated".to_string(),
            unit: "1".to_string(),
            metric_type: MetricType::Counter,
            value: metrics_snapshot.task_generations as f64,
            timestamp: now,
            attributes: HashMap::new(),
            resource: resource.clone(),
        });

        otel_metrics.push(OtelMetric {
            name: "learner_updates_total".to_string(),
            description: "Total number of learner model updates".to_string(),
            unit: "1".to_string(),
            metric_type: MetricType::Counter,
            value: metrics_snapshot.learner_updates as f64,
            timestamp: now,
            attributes: HashMap::new(),
            resource: resource.clone(),
        });

        otel_metrics.push(OtelMetric {
            name: "learning_sessions_created_total".to_string(),
            description: "Total number of learning sessions created".to_string(),
            unit: "1".to_string(),
            metric_type: MetricType::Counter,
            value: metrics_snapshot.session_creations as f64,
            timestamp: now,
            attributes: HashMap::new(),
            resource: resource.clone(),
        });

        // Per-endpoint metrics
        for (endpoint, endpoint_metrics) in &metrics_snapshot.endpoint_metrics {
            let mut endpoint_attrs = HashMap::new();
            endpoint_attrs.insert("endpoint".to_string(), endpoint.clone());
            
            otel_metrics.push(OtelMetric {
                name: "http_requests_per_endpoint_total".to_string(),
                description: "Total number of HTTP requests per endpoint".to_string(),
                unit: "1".to_string(),
                metric_type: MetricType::Counter,
                value: endpoint_metrics.total_requests as f64,
                timestamp: now,
                attributes: endpoint_attrs.clone(),
                resource: resource.clone(),
            });

            otel_metrics.push(OtelMetric {
                name: "http_request_duration_per_endpoint_avg_ms".to_string(),
                description: "Average HTTP request duration per endpoint in milliseconds".to_string(),
                unit: "ms".to_string(),
                metric_type: MetricType::Gauge,
                value: endpoint_metrics.avg_duration_ms,
                timestamp: now,
                attributes: endpoint_attrs,
                resource: resource.clone(),
            });
        }

        info!(
            metrics_count = otel_metrics.len(),
            "Exported OpenTelemetry metrics"
        );

        otel_metrics
    }

    /// Export system metrics in OpenTelemetry format
    pub async fn export_system_metrics(&self) -> Vec<OtelMetric> {
        let resource = self.create_resource();
        let now = Utc::now();
        let mut otel_metrics = Vec::new();

        // Memory usage
        if let Some(memory_mb) = get_memory_usage_mb() {
            otel_metrics.push(OtelMetric {
                name: "process_memory_usage_mb".to_string(),
                description: "Process memory usage in megabytes".to_string(),
                unit: "MB".to_string(),
                metric_type: MetricType::Gauge,
                value: memory_mb as f64,
                timestamp: now,
                attributes: HashMap::new(),
                resource: resource.clone(),
            });
        }

        // CPU usage (if available)
        if let Some(cpu_percent) = get_cpu_usage_percent() {
            otel_metrics.push(OtelMetric {
                name: "process_cpu_usage_percent".to_string(),
                description: "Process CPU usage percentage".to_string(),
                unit: "%".to_string(),
                metric_type: MetricType::Gauge,
                value: cpu_percent as f64,
                timestamp: now,
                attributes: HashMap::new(),
                resource: resource.clone(),
            });
        }

        // File descriptor usage (Unix-like systems)
        #[cfg(unix)]
        if let Some(fd_count) = get_file_descriptor_count() {
            otel_metrics.push(OtelMetric {
                name: "process_open_file_descriptors".to_string(),
                description: "Number of open file descriptors".to_string(),
                unit: "1".to_string(),
                metric_type: MetricType::Gauge,
                value: fd_count as f64,
                timestamp: now,
                attributes: HashMap::new(),
                resource: resource.clone(),
            });
        }

        otel_metrics
    }

    fn create_resource(&self) -> OtelResource {
        let instance_id = uuid::Uuid::new_v4().to_string();
        let hostname = self.custom_attributes
            .get("host.name")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        OtelResource {
            service_name: self.service_name.clone(),
            service_version: self.service_version.clone(),
            service_instance_id: instance_id,
            deployment_environment: self.environment.clone(),
            host_name: hostname,
        }
    }

    /// Export metrics in Prometheus format for compatibility
    pub async fn export_prometheus_format(&self) -> String {
        let app_metrics = self.export_application_metrics().await;
        let system_metrics = self.export_system_metrics().await;
        
        let mut output = String::new();
        
        for metric in app_metrics.iter().chain(system_metrics.iter()) {
            // Add metric help and type
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.description));
            output.push_str(&format!("# TYPE {} {}\n", metric.name, metric_type_to_prometheus(&metric.metric_type)));
            
            // Add metric value with labels
            if metric.attributes.is_empty() {
                output.push_str(&format!("{} {}\n", metric.name, metric.value));
            } else {
                let labels: Vec<String> = metric.attributes
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                output.push_str(&format!("{}{{{}}} {}\n", metric.name, labels.join(","), metric.value));
            }
            output.push('\n');
        }
        
        output
    }
}

/// Convert MetricType to Prometheus format string
fn metric_type_to_prometheus(metric_type: &MetricType) -> &str {
    match metric_type {
        MetricType::Counter => "counter",
        MetricType::Gauge => "gauge",
        MetricType::Histogram => "histogram",
        MetricType::Summary => "summary",
    }
}

/// Get memory usage in MB (platform-specific)
fn get_memory_usage_mb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return Some(kb / 1024); // Convert KB to MB
                        }
                    }
                }
            }
        }
    }
    
    // Fallback for other platforms or if reading fails
    None
}

/// Get CPU usage percentage (simplified implementation)
fn get_cpu_usage_percent() -> Option<f32> {
    // This is a simplified placeholder - in production you'd want to use
    // a proper system monitoring library like `sysinfo`
    None
}

/// Get file descriptor count (Unix-like systems)
#[cfg(unix)]
fn get_file_descriptor_count() -> Option<u32> {
    use std::fs;
    if let Ok(entries) = fs::read_dir("/proc/self/fd") {
        Some(entries.count() as u32)
    } else {
        None
    }
}

#[cfg(not(unix))]
fn get_file_descriptor_count() -> Option<u32> {
    None
}

/// Create a structured log event compatible with OpenTelemetry
pub fn create_otel_log_event(
    level: tracing::Level,
    message: &str,
    trace_id: Option<&str>,
    span_id: Option<&str>,
    attributes: HashMap<String, String>,
) {
    let mut fields = HashMap::new();
    
    if let Some(trace_id) = trace_id {
        fields.insert("trace_id".to_string(), trace_id.to_string());
    }
    
    if let Some(span_id) = span_id {
        fields.insert("span_id".to_string(), span_id.to_string());
    }
    
    for (key, value) in attributes {
        fields.insert(key, value);
    }

    // Use tracing macros with structured fields
    match level {
        tracing::Level::ERROR => error!(?fields, "{}", message),
        tracing::Level::WARN => warn!(?fields, "{}", message),
        tracing::Level::INFO => info!(?fields, "{}", message),
        tracing::Level::DEBUG => tracing::debug!(?fields, "{}", message),
        tracing::Level::TRACE => tracing::trace!(?fields, "{}", message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_otel_metrics_export() {
        let exporter = OtelMetricsExporter::new(
            "test_service".to_string(),
            "1.0.0".to_string(),
            "test".to_string(),
        );
        
        let metrics = exporter.export_application_metrics().await;
        assert!(!metrics.is_empty());
        
        // Check that we have basic HTTP metrics
        let has_request_count = metrics.iter().any(|m| m.name == "http_requests_total");
        assert!(has_request_count);
    }

    #[tokio::test]
    async fn test_prometheus_format_export() {
        let exporter = OtelMetricsExporter::new(
            "test_service".to_string(),
            "1.0.0".to_string(),
            "test".to_string(),
        );
        
        let prometheus_output = exporter.export_prometheus_format().await;
        assert!(!prometheus_output.is_empty());
        assert!(prometheus_output.contains("# HELP"));
        assert!(prometheus_output.contains("# TYPE"));
    }
}