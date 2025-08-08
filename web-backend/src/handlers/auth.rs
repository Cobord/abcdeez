use axum::{extract::State, http::StatusCode, Json};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::{user::{User, CreateUserRequest, LoginRequest, TokenResponse}},
    state::AppState,
    services::audit::AuditService,
};

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<User>)> {
    // Validate input
    if req.username.len() < 3 {
        return Err(AppError::ValidationError("Username must be at least 3 characters".to_string()));
    }
    if req.password.len() < 8 {
        return Err(AppError::ValidationError("Password must be at least 8 characters".to_string()));
    }
    if !req.email.contains('@') {
        return Err(AppError::ValidationError("Invalid email format".to_string()));
    }

    // Check if user already exists
    let existing = sqlx::query!(
        "SELECT id FROM users WHERE username = $1 OR email = $2",
        req.username,
        req.email
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::ConflictError("Username or email already exists".to_string()));
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalServerError)?
        .to_string();

    // Create user in database
    let user_id = Uuid::new_v4();
    let user_id_bytes = user_id.as_bytes();
    let now = Utc::now();

    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, created_at, updated_at, metadata) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        user_id_bytes,
        req.username,
        req.email,
        password_hash,
        now,
        now,
        serde_json::json!({}).to_string()
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let user = User {
        id: user_id,
        username: req.username.clone(),
        email: req.email.clone(),
        password_hash,
        created_at: now,
        updated_at: now,
        metadata: Some(serde_json::json!({})),
    };

    // Log audit event
    AuditService::log_event(
        &state.db_pool,
        Some(user_id),
        "create".to_string(),
        "user".to_string(),
        user_id.to_string(),
        Some(serde_json::json!({
            "username": req.username,
            "email": req.email
        })),
        None,
        None,
    ).await.ok();

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    // Get user from database
    let user_row = sqlx::query!(
        "SELECT id, username, email, password_hash, created_at, updated_at, metadata 
         FROM users WHERE username = $1",
        req.username
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let user_row = user_row.ok_or(AppError::Unauthorized)?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user_row.password_hash)
        .map_err(|_| AppError::InternalServerError)?;
    
    let argon2 = Argon2::default();
    argon2
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized)?;

    let user_id = Uuid::from_bytes(
        user_row.id.try_into().unwrap_or_default()
    );

    // Generate tokens
    let now = Utc::now();
    let access_exp = now + Duration::hours(state.config.jwt_expiration_hours);
    let refresh_exp = now + Duration::days(30); // Refresh tokens last 30 days

    let access_claims = Claims {
        sub: user_id,
        username: req.username.clone(),
        exp: access_exp.timestamp(),
        iat: now.timestamp(),
    };

    let refresh_claims = Claims {
        sub: user_id,
        username: req.username.clone(),
        exp: refresh_exp.timestamp(),
        iat: now.timestamp(),
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )?;

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )?;

    // Store refresh token in Redis
    let refresh_key = format!("refresh_token:{}", user_id);
    let mut conn = state.redis_conn.clone();
    redis::cmd("SETEX")
        .arg(&refresh_key)
        .arg(30 * 24 * 3600) // 30 days TTL
        .arg(&refresh_token)
        .query_async(&mut conn)
        .await
        .ok();

    // Update last active timestamp
    let user_id_bytes = user_id.as_bytes();
    sqlx::query!(
        "UPDATE users SET updated_at = $1 WHERE id = $2",
        now,
        user_id_bytes
    )
    .execute(&state.db_pool)
    .await
    .ok();

    // Log audit event
    AuditService::log_event(
        &state.db_pool,
        Some(user_id),
        "login".to_string(),
        "auth".to_string(),
        user_id.to_string(),
        None,
        None,
        None,
    ).await.ok();

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiration_hours * 3600,
    }))
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> AppResult<Json<TokenResponse>> {
    let refresh_token = req["refresh_token"]
        .as_str()
        .ok_or(AppError::ValidationError("refresh_token required".to_string()))?;

    // Decode refresh token to get user info
    let token_data = jsonwebtoken::decode::<Claims>(
        refresh_token,
        &jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let user_id = token_data.claims.sub;

    // Verify refresh token exists in Redis
    let refresh_key = format!("refresh_token:{}", user_id);
    let mut conn = state.redis_conn.clone();
    let stored_token: Option<String> = redis::cmd("GET")
        .arg(&refresh_key)
        .query_async(&mut conn)
        .await
        .ok();

    if stored_token.as_ref() != Some(refresh_token) {
        return Err(AppError::Unauthorized);
    }

    // Generate new access token
    let now = Utc::now();
    let access_exp = now + Duration::hours(state.config.jwt_expiration_hours);

    let access_claims = Claims {
        sub: user_id,
        username: token_data.claims.username.clone(),
        exp: access_exp.timestamp(),
        iat: now.timestamp(),
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: refresh_token.to_string(),
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiration_hours * 3600,
    }))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    claims: axum::Extension<Claims>,
) -> AppResult<StatusCode> {
    let user_id = claims.sub;

    // Remove refresh token from Redis
    let refresh_key = format!("refresh_token:{}", user_id);
    let mut conn = state.redis_conn.clone();
    redis::cmd("DEL")
        .arg(&refresh_key)
        .query_async::<()>(&mut conn)
        .await
        .ok();

    // Log audit event
    AuditService::log_event(
        &state.db_pool,
        Some(user_id),
        "logout".to_string(),
        "auth".to_string(),
        user_id.to_string(),
        None,
        None,
        None,
    ).await.ok();

    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    claims: axum::Extension<Claims>,
) -> AppResult<Json<User>> {
    let user_id = claims.sub;
    let user_id_bytes = user_id.as_bytes();

    // Get user from database
    let user_row = sqlx::query!(
        "SELECT id, username, email, password_hash, created_at, updated_at, metadata 
         FROM users WHERE id = $1",
        user_id_bytes
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let user_row = user_row.ok_or(AppError::NotFound("User not found".to_string()))?;

    let user = User {
        id: user_id,
        username: user_row.username,
        email: user_row.email,
        password_hash: user_row.password_hash,
        created_at: user_row.created_at,
        updated_at: user_row.updated_at,
        metadata: user_row.metadata.and_then(|s| serde_json::from_str(&s).ok()),
    };

    Ok(Json(user))
}