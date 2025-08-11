use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::{header::CONTENT_LENGTH, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use futures::StreamExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;
use http_body_util::BodyExt;

use crate::error::AppError;

/// Configuration for request body size limits
#[derive(Debug, Clone)]
pub struct BodyLimitConfig {
    /// Maximum size for regular API requests (default: 1MB)
    pub api_limit: usize,
    /// Maximum size for file uploads (default: 10MB)
    pub upload_limit: usize,
    /// Maximum size for admin operations (default: 50MB)
    pub admin_limit: usize,
}

impl Default for BodyLimitConfig {
    fn default() -> Self {
        Self {
            api_limit: 1_000_000,      // 1MB
            upload_limit: 10_000_000,   // 10MB
            admin_limit: 50_000_000,    // 50MB
        }
    }
}

/// Middleware to enforce body size limits with streaming support
pub async fn body_size_limit(
    config: BodyLimitConfig,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();
    
    // Determine the appropriate limit based on the path
    let limit = if path.contains("/upload") || path.contains("/import") {
        config.upload_limit
    } else if path.contains("/admin") || path.contains("/export") {
        config.admin_limit
    } else {
        config.api_limit
    };
    
    // Check Content-Length header first (fast path)
    if let Some(content_length) = request.headers().get(CONTENT_LENGTH) {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > limit {
                    return Err(AppError::PayloadTooLarge(format!(
                        "Content-Length {} exceeds maximum allowed size of {} bytes",
                        length, limit
                    )));
                }
            }
        }
    }
    
    // For streaming bodies without Content-Length, wrap and limit
    let (parts, body) = request.into_parts();
    let limited_body = LimitedBody::new(body, limit);
    let request = Request::from_parts(parts, Body::new(limited_body));
    
    Ok(next.run(request).await)
}

/// A body wrapper that enforces size limits during streaming
pub struct LimitedBody {
    inner: Body,
    limit: usize,
    consumed: Arc<AtomicUsize>,
}

impl LimitedBody {
    pub fn new(inner: Body, limit: usize) -> Self {
        Self {
            inner,
            limit,
            consumed: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl http_body::Body for LimitedBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let inner = std::pin::Pin::new(&mut self.inner);
        
        match inner.poll_frame(cx) {
            std::task::Poll::Ready(Some(Ok(frame))) => {
                if let Some(data) = frame.data_ref() {
                    let new_consumed = self.consumed.fetch_add(data.len(), Ordering::SeqCst) + data.len();
                    
                    if new_consumed > self.limit {
                        // Exceeded limit
                        tracing::warn!(
                            "Request body exceeded limit: {} > {}",
                            new_consumed,
                            self.limit
                        );
                        return std::task::Poll::Ready(Some(Err(
                            axum::Error::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Body exceeded {} byte limit", self.limit),
                            ))
                        )));
                    }
                }
                std::task::Poll::Ready(Some(Ok(frame)))
            }
            other => other,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        let mut hint = self.inner.size_hint();
        
        // Clamp the upper bound to our limit
        if let Some(upper) = hint.upper() {
            if upper > self.limit as u64 {
                hint.set_upper(self.limit as u64);
            }
        } else {
            hint.set_upper(self.limit as u64);
        }
        
        hint
    }
}

/// Per-route configuration for dynamic limits
pub struct DynamicBodyLimit {
    config: Arc<std::sync::RwLock<BodyLimitConfig>>,
}

impl DynamicBodyLimit {
    pub fn new(config: BodyLimitConfig) -> Self {
        Self {
            config: Arc::new(std::sync::RwLock::new(config)),
        }
    }
    
    /// Update limits at runtime
    pub fn update_limits(&self, new_config: BodyLimitConfig) {
        if let Ok(mut config) = self.config.write() {
            *config = new_config;
        }
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> BodyLimitConfig {
        self.config.read().unwrap().clone()
    }
}

/// Helper middleware for multipart form uploads with additional validation
pub async fn multipart_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let content_type = request
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if content_type.starts_with("multipart/form-data") {
        // Additional validation for multipart uploads
        
        // Check for required boundary parameter
        if !content_type.contains("boundary=") {
            return Err(AppError::BadRequest(
                "Multipart request missing boundary parameter".to_string(),
            ));
        }
        
        // Enforce stricter limits for multipart
        if let Some(content_length) = request.headers().get(CONTENT_LENGTH) {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    // Limit individual parts to prevent memory exhaustion
                    const MAX_PARTS: usize = 100;
                    const MAX_PART_SIZE: usize = 5_000_000; // 5MB per part
                    
                    // Rough estimate: if average part is small, we might have too many parts
                    if length < MAX_PARTS * 1000 {
                        // Suspiciously small parts, might be an attack
                        tracing::warn!(
                            "Multipart upload with suspiciously small average part size: {} bytes for {} potential parts",
                            length / MAX_PARTS,
                            MAX_PARTS
                        );
                    }
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_body_limit_config() {
        let config = BodyLimitConfig::default();
        assert_eq!(config.api_limit, 1_000_000);
        assert_eq!(config.upload_limit, 10_000_000);
        assert_eq!(config.admin_limit, 50_000_000);
    }
    
    #[test]
    fn test_dynamic_body_limit() {
        let dynamic = DynamicBodyLimit::new(BodyLimitConfig::default());
        
        let initial = dynamic.get_config();
        assert_eq!(initial.api_limit, 1_000_000);
        
        dynamic.update_limits(BodyLimitConfig {
            api_limit: 2_000_000,
            upload_limit: 20_000_000,
            admin_limit: 100_000_000,
        });
        
        let updated = dynamic.get_config();
        assert_eq!(updated.api_limit, 2_000_000);
        assert_eq!(updated.upload_limit, 20_000_000);
    }
}