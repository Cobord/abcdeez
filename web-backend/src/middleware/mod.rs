use axum::{
    extract::{Request, State},
    http::{header, Method},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::Instant;
use uuid::Uuid;

use crate::{
    error::AppError,
    services::audit::{AuditContext, AuditService},
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // User ID
    pub username: String,
    pub exp: i64,                   // Expiration time
    pub iat: i64,                   // Issued at
    pub role: String,               // User role
    pub permissions: Vec<String>,   // User permissions
    pub session_id: Option<String>, // Session tracking
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Skip auth for certain paths
    let path = request.uri().path();
    let public_paths = vec!["/api/auth/register", "/api/auth/login", "/api/auth/refresh"];

    if public_paths.contains(&path) || path.starts_with("/health/") {
        return Ok(next.run(request).await);
    }

    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    // Enhanced token validation with explicit algorithm
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.leeway = 60; // Allow 60 seconds clock skew
    validation.algorithms = vec![Algorithm::HS256]; // Only allow HS256

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|e| {
        tracing::warn!("Token validation failed: {:?}", e);
        AppError::Unauthorized
    })?;

    // Check if token is in blacklist (for logout/revocation)
    if let Some(session_id) = &token_data.claims.session_id.clone() {
        let blacklist_key = format!("blacklist:session:{}", session_id);
        let mut conn = state.cache_conn.clone();
        if let Ok(blacklisted) = crate::cache::cmd("EXISTS")
            .arg(&blacklist_key)
            .query_async::<String>(&mut conn)
            .await
        {
            if blacklisted.parse::<i64>().unwrap_or(0) > 0 {
                tracing::warn!("Attempted use of blacklisted session: {}", session_id);
                return Err(AppError::Unauthorized);
            }
        }
    }

    // Verify user still exists and is active
    let user_id_bytes = token_data.claims.sub.as_bytes();
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let user_active = sqlx::query_scalar::<_, bool>(
        "SELECT CASE WHEN metadata->>'$.status' IS NULL OR metadata->>'$.status' != 'disabled' THEN true ELSE false END
         FROM users WHERE id = ?"
    )
    .bind(&user_id_bytes[..])
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?
    .unwrap_or(false);

    if !user_active {
        tracing::warn!(
            "Attempted access with disabled/deleted user: {}",
            token_data.claims.sub
        );
        return Err(AppError::Unauthorized);
    }

    // Rate limiting per user (in addition to global rate limiting)
    let user_rate_key = format!("user_rate_limit:{}", token_data.claims.sub);
    let mut conn = state.cache_conn.clone();
    let current_requests: i64 = crate::cache::cmd("GET")
        .arg(&user_rate_key)
        .query_async::<String>(&mut conn)
        .await
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if current_requests > (state.config.rate_limit_requests * 2) as i64 {
        // Higher limit for authenticated users
        return Err(AppError::RateLimitExceeded);
    }

    // Increment user-specific counter
    let _: Result<(), _> = crate::cache::cmd("INCR")
        .arg(&user_rate_key)
        .query_async(&mut conn)
        .await;
    let _: Result<(), _> = crate::cache::cmd("EXPIRE")
        .arg(&user_rate_key)
        .arg(state.config.rate_limit_window_seconds as i64)
        .query_async(&mut conn)
        .await;

    // Add user info to request extensions
    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}

// Researcher role authorization middleware
pub async fn require_researcher(
    claims: axum::Extension<Claims>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if claims.role != "researcher" && claims.role != "admin" {
        tracing::warn!(
            "Researcher access denied for user: {} with role: {}",
            claims.username,
            claims.role
        );
        return Err(AppError::Forbidden);
    }
    Ok(next.run(request).await)
}

// Permission-based authorization middleware (simplified approach)
pub async fn require_analytics_permission(
    claims: axum::Extension<Claims>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if !claims.permissions.contains(&"analytics_access".to_string()) && claims.role != "admin" {
        tracing::warn!(
            "Analytics access denied for user: {} with permissions: {:?}",
            claims.username,
            claims.permissions
        );
        return Err(AppError::Forbidden);
    }
    Ok(next.run(request).await)
}

// Admin-only authorization middleware
pub async fn require_admin(
    claims: axum::Extension<Claims>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if claims.role != "admin" {
        tracing::warn!(
            "Admin access denied for user: {} with role: {}",
            claims.username,
            claims.role
        );
        return Err(AppError::Forbidden);
    }
    Ok(next.run(request).await)
}

// Enhanced security headers middleware
pub async fn security_headers(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Core security headers (always applied)
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // Conditional security headers based on environment
    if state.config.environment == crate::config::Environment::Production {
        // Strict HSTS for production
        headers.insert(
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains; preload"
                .parse()
                .unwrap(),
        );

        // Strict CSP for production
        headers.insert("Content-Security-Policy", 
            "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data: https:; connect-src 'self'; font-src 'self'; object-src 'none'; media-src 'self'; frame-src 'none'; sandbox allow-scripts allow-same-origin allow-forms; base-uri 'self';".parse().unwrap());
    } else {
        // Relaxed headers for development
        headers.insert("Strict-Transport-Security", "max-age=0".parse().unwrap());

        headers.insert("Content-Security-Policy", 
            "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' ws: wss:;".parse().unwrap());
    }

    // API-specific headers
    if path.starts_with("/api/") {
        headers.insert("X-API-Version", "1.0".parse().unwrap());
        headers.insert(
            "Cache-Control",
            "no-store, no-cache, must-revalidate, max-age=0"
                .parse()
                .unwrap(),
        );
        headers.insert("Pragma", "no-cache".parse().unwrap());

        // Remove server info for APIs
        headers.remove("server");
    }

    // Security headers for sensitive endpoints
    if path.starts_with("/api/admin/") || path.starts_with("/api/auth/") {
        headers.insert("X-Permitted-Cross-Domain-Policies", "none".parse().unwrap());
        headers.insert("X-DNS-Prefetch-Control", "off".parse().unwrap());
        headers.insert("X-Download-Options", "noopen".parse().unwrap());

        // Additional CSP for admin endpoints
        if let Some(csp) = headers.get_mut("Content-Security-Policy") {
            if let Ok(csp_str) = csp.to_str() {
                *csp = format!("{}; require-trusted-types-for 'script';", csp_str)
                    .parse()
                    .unwrap();
            }
        }
    }

    response
}

/// Enhanced rate limiting with endpoint-specific limits and burst control
pub async fn rate_limit(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();
    let method = request.method();

    // Endpoint-specific rate limits
    let (limit, window, burst_limit) = determine_rate_limits(&state.config, path, method);

    // Get client identifier with better IP detection
    let client_id = get_client_identifier(&request);
    let rate_key = format!("rate_limit:{}", client_id);
    let burst_key = format!("burst_limit:{}", client_id);

    let mut conn = state.cache_conn.clone();

    // Check burst limit (shorter window, higher threshold)
    let burst_count: i64 = crate::cache::cmd("GET")
        .arg(&burst_key)
        .query_async::<String>(&mut conn)
        .await
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if burst_count > burst_limit {
        return Err(AppError::RateLimitExceeded);
    }

    // Increment burst counter with short TTL (1 minute)
    crate::cache::cmd("INCR")
        .arg(&burst_key)
        .query_async::<String>(&mut conn)
        .await
        .ok();
    crate::cache::cmd("EXPIRE")
        .arg(&burst_key)
        .arg(60) // 1 minute burst window
        .query_async::<()>(&mut conn)
        .await
        .ok();

    // Check regular rate limit
    let current_count: i64 = crate::cache::cmd("GET")
        .arg(&rate_key)
        .query_async::<String>(&mut conn)
        .await
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if current_count >= limit {
        return Err(AppError::RateLimitExceeded);
    }

    // Increment rate limit counter
    crate::cache::cmd("INCR")
        .arg(&rate_key)
        .query_async::<String>(&mut conn)
        .await
        .ok();
    crate::cache::cmd("EXPIRE")
        .arg(&rate_key)
        .arg(window as i64)
        .query_async::<()>(&mut conn)
        .await
        .ok();

    // Add rate limit headers to response
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert("X-RateLimit-Limit", limit.to_string().parse().unwrap());
    headers.insert(
        "X-RateLimit-Remaining",
        (limit - current_count - 1)
            .max(0)
            .to_string()
            .parse()
            .unwrap(),
    );
    headers.insert("X-RateLimit-Window", window.to_string().parse().unwrap());

    Ok(response)
}

/// Determine rate limits based on endpoint and method
fn determine_rate_limits(
    config: &crate::config::Config,
    path: &str,
    method: &Method,
) -> (i64, u32, i64) {
    let base_limit = config.rate_limit_requests as i64;
    let base_window = config.rate_limit_window_seconds;

    // More restrictive limits for sensitive endpoints
    match (path, method) {
        // Authentication endpoints - very restrictive
        (path, &Method::POST) if path.starts_with("/api/auth/login") => {
            (5, base_window as u32, 10) // 5 login attempts per window, burst of 10
        }
        (path, &Method::POST) if path.starts_with("/api/auth/register") => {
            (3, base_window as u32, 5) // 3 registration attempts per window
        }

        // Admin endpoints - restrictive
        (path, _) if path.starts_with("/api/admin/") => {
            (base_limit / 4, base_window as u32, base_limit / 2) // Quarter normal limit
        }

        // Analytics endpoints - moderate restriction
        (path, _) if path.starts_with("/api/analytics/") => {
            (base_limit / 2, base_window as u32, base_limit) // Half normal limit
        }

        // Export endpoints - very restrictive
        (path, _) if path.contains("/export") => {
            (2, (base_window * 2) as u32, 3) // 2 exports per double window
        }

        // Write operations - more restrictive than reads
        (path, &Method::POST)
        | (path, &Method::PUT)
        | (path, &Method::PATCH)
        | (path, &Method::DELETE)
            if path.starts_with("/api/") =>
        {
            (base_limit / 2, base_window as u32, base_limit) // Half limit for writes
        }

        // Health checks - very permissive
        (path, _) if path.starts_with("/health/") => {
            (base_limit * 5, base_window as u32, base_limit * 10) // 5x normal limit
        }

        // Default limits
        _ => (base_limit, base_window as u32, base_limit * 2),
    }
}

/// Get client identifier with better IP detection and user identification
fn get_client_identifier(request: &Request) -> String {
    // Use user ID from claims if authenticated
    if let Some(claims) = request.extensions().get::<Claims>() {
        return format!("user:{}", claims.sub);
    }

    // Extract IP address with proxy support
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
        })
        .unwrap_or("unknown");

    // Combine IP with user agent for better fingerprinting
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Create a simple hash to avoid storing full user agent strings
    format!(
        "ip:{}:ua:{}",
        ip,
        user_agent.chars().take(20).collect::<String>()
    )
}

/// IP blocking middleware for security
pub async fn ip_blocking(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
        })
        .unwrap_or("unknown");

    // Check if IP is in blocklist
    let blocklist_key = format!("blocked_ip:{}", ip);
    let mut conn = state.cache_conn.clone();

    if let Ok(_) = crate::cache::cmd("GET")
        .arg(&blocklist_key)
        .query_async::<String>(&mut conn)
        .await
    {
        tracing::warn!("Blocked request from IP: {}", ip);
        return Err(AppError::Forbidden);
    }

    // Check for suspicious patterns (high error rate)
    let error_key = format!("ip_errors:{}", ip);
    let error_count: i64 = crate::cache::cmd("GET")
        .arg(&error_key)
        .query_async::<String>(&mut conn)
        .await
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Auto-block IPs with too many errors
    if error_count > 50 {
        // Block for 1 hour
        let _ = crate::cache::cmd("SETEX")
            .arg(&blocklist_key)
            .arg(3600) // 1 hour
            .arg("auto_blocked")
            .query_async::<String>(&mut conn)
            .await;

        tracing::warn!("Auto-blocked IP due to error rate: {}", ip);
        return Err(AppError::Forbidden);
    }

    Ok(next.run(request).await)
}

/// Content validation middleware for API endpoints
pub async fn content_validation(request: Request, next: Next) -> Result<Response, AppError> {
    let path = request.uri().path();

    // Skip validation for non-API endpoints
    if !path.starts_with("/api/") {
        return Ok(next.run(request).await);
    }

    // Validate Content-Type for POST/PUT/PATCH requests
    if matches!(
        request.method(),
        &Method::POST | &Method::PUT | &Method::PATCH
    ) {
        let content_type = request
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        // Require JSON content type for API endpoints (except file uploads)
        if !content_type.starts_with("application/json")
            && !content_type.starts_with("multipart/form-data")
            && !path.contains("/upload")
        {
            return Err(AppError::BadRequest(
                "Invalid Content-Type. Expected application/json".to_string(),
            ));
        }

        // Check Content-Length to prevent large payloads
        if let Some(length_header) = request.headers().get("content-length") {
            if let Ok(length) = length_header.to_str().unwrap_or("0").parse::<u64>() {
                let max_size = if path.contains("/upload") {
                    10_000_000 // 10MB for uploads
                } else {
                    1_000_000 // 1MB for regular API calls
                };

                if length > max_size {
                    return Err(AppError::BadRequest(format!(
                        "Request too large. Maximum size: {} bytes",
                        max_size
                    )));
                }
            }
        }
    }

    Ok(next.run(request).await)
}

/// Comprehensive audit logging middleware
pub async fn audit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|q| q.to_string());

    // Skip audit logging for health checks and metrics endpoints
    if path.starts_with("/health/") || path.starts_with("/metrics") {
        return next.run(request).await;
    }

    // Extract user context from request extensions (set by auth middleware)
    let claims = request.extensions().get::<Claims>().cloned();
    let audit_context = AuditContext::from_request(&request, claims.as_ref());

    // Run the request
    let response = next.run(request).await;
    let status = response.status();
    let duration = start_time.elapsed();

    // Log API call for compliance
    let action = match method {
        Method::GET => "read",
        Method::POST => "create",
        Method::PUT | Method::PATCH => "update",
        Method::DELETE => "delete",
        _ => "unknown",
    };

    // Determine resource type from path
    let (resource_type, resource_id) = parse_resource_from_path(&path);

    let audit_details = serde_json::json!({
        "method": method.to_string(),
        "path": path,
        "query": query,
        "status_code": status.as_u16(),
        "duration_ms": duration.as_millis(),
        "api_call": true,
        "user_agent": audit_context.user_agent,
        "session_id": audit_context.session_id
    });

    // Log failed requests as security events
    if status.is_client_error() || status.is_server_error() {
        let _result = AuditService::log_security_event(
            &state.db_pool,
            &audit_context,
            if status.is_client_error() {
                "client_error".to_string()
            } else {
                "server_error".to_string()
            },
            audit_details.clone(),
        )
        .await;
    }

    // Log all API calls for compliance (background task to avoid blocking)
    let db_pool = state.db_pool.clone();
    let context = audit_context.clone();
    tokio::spawn(async move {
        let _ = AuditService::log_event_with_context(
            &db_pool,
            &context,
            action.to_string(),
            resource_type,
            resource_id,
            Some(audit_details),
        )
        .await;
    });

    response
}

/// Parse resource type and ID from API path
fn parse_resource_from_path(path: &str) -> (String, String) {
    let segments: Vec<&str> = path.trim_start_matches("/api/").split('/').collect();

    match segments.as_slice() {
        ["auth", action] => ("auth".to_string(), action.to_string()),
        ["learners"] => ("learners".to_string(), "collection".to_string()),
        ["learners", id] => ("learner".to_string(), id.to_string()),
        ["learners", id, sub] => ("learner".to_string(), format!("{}:{}", id, sub)),
        ["sessions"] => ("sessions".to_string(), "collection".to_string()),
        ["sessions", id] => ("session".to_string(), id.to_string()),
        ["sessions", id, sub] => ("session".to_string(), format!("{}:{}", id, sub)),
        ["analytics", endpoint] => ("analytics".to_string(), endpoint.to_string()),
        ["admin", endpoint] => ("admin".to_string(), endpoint.to_string()),
        ["tasks", endpoint] => ("tasks".to_string(), endpoint.to_string()),
        ["experiments", id] => ("experiment".to_string(), id.to_string()),
        ["music", endpoint] => ("music".to_string(), endpoint.to_string()),
        _ => ("api".to_string(), path.to_string()),
    }
}

/// Enhanced audit logging for sensitive operations
pub async fn audit_sensitive_operation(
    state: &AppState,
    context: &AuditContext,
    operation: &str,
    resource_type: &str,
    resource_id: &str,
    details: serde_json::Value,
) {
    let enhanced_details = serde_json::json!({
        "operation": operation,
        "details": details,
        "sensitive_operation": true,
        "timestamp": chrono::Utc::now()
    });

    let _ = AuditService::log_event_with_context(
        &state.db_pool,
        context,
        "sensitive_operation".to_string(),
        resource_type.to_string(),
        resource_id.to_string(),
        Some(enhanced_details),
    )
    .await;
}
