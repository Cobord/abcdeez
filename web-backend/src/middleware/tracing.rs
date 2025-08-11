use axum::{
    extract::Request,
    http::{header::HeaderName, HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{error, info, warn, Instrument};
use uuid::Uuid;

/// HTTP header for correlation/trace ID
pub const TRACE_ID_HEADER: &str = "x-trace-id";
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Middleware for distributed tracing with correlation IDs
pub async fn tracing_middleware(mut request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    // Extract or generate trace ID
    let trace_id = extract_or_generate_trace_id(request.headers());
    let request_id = Uuid::new_v4().to_string();

    // Add trace context to request headers for downstream services
    if let Ok(trace_header) = HeaderValue::from_str(&trace_id) {
        request.headers_mut().insert(
            HeaderName::from_static(TRACE_ID_HEADER),
            trace_header,
        );
    }
    if let Ok(request_header) = HeaderValue::from_str(&request_id) {
        request.headers_mut().insert(
            HeaderName::from_static(REQUEST_ID_HEADER),
            request_header,
        );
    }

    // Create span for this request
    let span = tracing::span!(
        tracing::Level::INFO,
        "http_request",
        method = %method,
        path = %path,
        trace_id = %trace_id,
        request_id = %request_id,
        query = %query
    );

    // Execute request within the span context
    let response = async move {
        info!(
            method = %method,
            path = %path,
            query = %query,
            trace_id = %trace_id,
            request_id = %request_id,
            "Request started"
        );

        let mut response = next.run(request).await;
        let duration = start_time.elapsed();
        let status = response.status();

        // Add correlation headers to response
        if let Ok(trace_header) = HeaderValue::from_str(&trace_id) {
            response.headers_mut().insert(
                HeaderName::from_static(TRACE_ID_HEADER),
                trace_header,
            );
        }
        if let Ok(request_header) = HeaderValue::from_str(&request_id) {
            response.headers_mut().insert(
                HeaderName::from_static(REQUEST_ID_HEADER),
                request_header,
            );
        }

        // Log response with metrics based on status
        if status.is_success() {
            tracing::info!(
                method = %method,
                path = %path,
                status = %status,
                duration_ms = duration.as_millis(),
                trace_id = %trace_id,
                request_id = %request_id,
                "Request completed successfully"
            );
        } else if status.is_client_error() {
            tracing::warn!(
                method = %method,
                path = %path,
                status = %status,
                duration_ms = duration.as_millis(),
                trace_id = %trace_id,
                request_id = %request_id,
                "Request completed with client error"
            );
        } else {
            tracing::error!(
                method = %method,
                path = %path,
                status = %status,
                duration_ms = duration.as_millis(),
                trace_id = %trace_id,
                request_id = %request_id,
                "Request completed with server error"
            );
        }

        // Record metrics
        crate::monitoring::global_metrics()
            .record_request(path, duration.as_millis() as u64, !status.is_success())
            .await;

        response
    }
    .instrument(span);

    response.await
}

/// Extract trace ID from headers or generate a new one
fn extract_or_generate_trace_id(headers: &HeaderMap) -> String {
    // Try multiple common trace ID headers
    let trace_headers = [
        TRACE_ID_HEADER,
        "x-correlation-id",
        "x-amzn-trace-id",
        "x-cloud-trace-context",
        "traceparent", // W3C Trace Context
    ];

    for header_name in &trace_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(trace_id) = header_value.to_str() {
                // For W3C traceparent, extract just the trace ID part
                if *header_name == "traceparent" {
                    if let Some(trace_id) = extract_w3c_trace_id(trace_id) {
                        return trace_id;
                    }
                } else {
                    return trace_id.to_string();
                }
            }
        }
    }

    // Generate new UUID-based trace ID
    Uuid::new_v4().to_string()
}

/// Extract trace ID from W3C traceparent header
/// Format: version-trace_id-parent_id-trace_flags
/// Example: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01
fn extract_w3c_trace_id(traceparent: &str) -> Option<String> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() >= 4 && parts[0] == "00" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Middleware for request/response body logging (development only)
pub async fn request_logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    // Only log in debug builds or when explicitly enabled
    #[cfg(debug_assertions)]
    {
        let user_agent = headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        let content_length = headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        tracing::debug!(
            method = %method,
            uri = %uri,
            user_agent = %user_agent,
            content_type = %content_type,
            content_length = content_length,
            "Request details"
        );
    }

    let response = next.run(request).await;

    #[cfg(debug_assertions)]
    {
        let status = response.status();
        let response_headers = response.headers();
        let response_content_length = response_headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        tracing::debug!(
            status = %status,
            response_content_length = response_content_length,
            "Response details"
        );
    }

    response
}

/// Error tracking middleware for automatic error logging and metrics
pub async fn error_tracking_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let trace_id = request
        .headers()
        .get(TRACE_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let response = next.run(request).await;
    let status = response.status();

    // Track errors and log them appropriately
    if status.is_client_error() {
        warn!(
            method = %method,
            path = %path,
            status = %status,
            trace_id = %trace_id,
            "Client error encountered"
        );

        // Record client error metrics
        crate::monitoring::global_metrics()
            .error_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    } else if status.is_server_error() {
        error!(
            method = %method,
            path = %path,
            status = %status,
            trace_id = %trace_id,
            "Server error encountered"
        );

        // Record server error metrics
        crate::monitoring::global_metrics()
            .error_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // TODO: Send alert to monitoring system for 5xx errors
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_w3c_trace_id() {
        let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let trace_id = extract_w3c_trace_id(traceparent);
        assert_eq!(
            trace_id,
            Some("4bf92f3577b34da6a3ce929d0e0e4736".to_string())
        );

        // Test invalid format
        let invalid_traceparent = "invalid-format";
        let trace_id = extract_w3c_trace_id(invalid_traceparent);
        assert_eq!(trace_id, None);
    }

    #[test]
    fn test_extract_or_generate_trace_id() {
        let mut headers = HeaderMap::new();
        headers.insert(TRACE_ID_HEADER, HeaderValue::from_static("test-trace-id"));

        let trace_id = extract_or_generate_trace_id(&headers);
        assert_eq!(trace_id, "test-trace-id");

        // Test generation when no header present
        let empty_headers = HeaderMap::new();
        let trace_id = extract_or_generate_trace_id(&empty_headers);
        assert!(!trace_id.is_empty());
        // Should be UUID format
        assert!(Uuid::parse_str(&trace_id).is_ok());
    }
}
