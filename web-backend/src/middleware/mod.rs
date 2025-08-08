use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // User ID
    pub username: String,
    pub exp: i64, // Expiration time
    pub iat: i64, // Issued at
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Skip auth for certain paths
    let path = request.uri().path();
    if path.starts_with("/api/auth/") && !path.ends_with("/me") {
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

    // Validate token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    // Add user info to request extensions
    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}

pub async fn rate_limit(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Simple in-memory rate limiting using the cache (replaces Redis)
    let limit = state.config.rate_limit_requests as i64;
    let window = state.config.rate_limit_window_seconds;

    // Identify the client (use auth header if present, otherwise path)
    let client_id = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("anon:{}", request.uri().path()));

    let key = format!("rate_limit:{}", client_id);

    // Increment counter with TTL
    let mut conn = state.cache_conn.clone();
    let current: Option<String> = crate::cache::cmd("GET")
        .arg(key.clone())
        .query_async::<String>(&mut conn)
        .await
        .ok();

    let mut count = current
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    count += 1;

    let _ = crate::cache::cmd("SETEX")
        .arg(key.clone())
        .arg(window)
        .arg(count.to_string())
        .query_async::<String>(&mut conn)
        .await;

    if count > limit {
        return Err(AppError::RateLimitExceeded);
    }

    Ok(next.run(request).await)
}
