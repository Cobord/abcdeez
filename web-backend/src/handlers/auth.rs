use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::user::{CreateUserRequest, LoginRequest, TokenResponse, User},
    services::audit::{AuditService, AuditContext},
    state::AppState,
};

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<User>)> {
    // Enhanced input validation
    if req.username.len() < 3 || req.username.len() > 50 {
        return Err(AppError::ValidationError(
            "Username must be between 3 and 50 characters".to_string(),
        ));
    }
    
    // Validate password strength if required
    if state.config.require_strong_passwords {
        validate_password_strength(&req.password)?;
    } else if req.password.len() < 8 {
        return Err(AppError::ValidationError(
            "Password must be at least 8 characters".to_string(),
        ));
    }
    
    // Enhanced email validation
    if !req.email.contains('@') || !req.email.contains('.') || req.email.len() > 100 {
        return Err(AppError::ValidationError(
            "Invalid email format".to_string(),
        ));
    }
    
    // Check for username/email restrictions
    if req.username.chars().any(|c| !c.is_alphanumeric() && c != '_' && c != '-') {
        return Err(AppError::ValidationError(
            "Username can only contain letters, numbers, underscores and hyphens".to_string(),
        ));
    }

    // Check if user already exists
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let existing = sqlx::query("SELECT id FROM users WHERE username = ? OR email = ?")
        .bind(&req.username)
        .bind(&req.email)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    if existing.is_some() {
        return Err(AppError::ConflictError(
            "Username or email already exists".to_string(),
        ));
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

    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, created_at, updated_at, metadata)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&user_id_bytes)
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .bind(serde_json::json!({}).to_string())
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

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
    )
    .await
    .ok();

    Ok((StatusCode::CREATED, Json(user)))
}

// Password strength validation helper
fn validate_password_strength(password: &str) -> AppResult<()> {
    if password.len() < 8 {
        return Err(AppError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }
    
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    
    if !has_upper || !has_lower || !has_digit || !has_special {
        return Err(AppError::ValidationError(
            "Password must contain at least one uppercase letter, lowercase letter, digit, and special character".to_string(),
        ));
    }
    
    // Check for common weak patterns
    let lower_password = password.to_lowercase();
    let weak_patterns = vec![
        "password", "123456", "qwerty", "abc123", "admin", "letmein",
        "welcome", "monkey", "dragon", "master", "shadow", "password123"
    ];
    
    for pattern in weak_patterns {
        if lower_password.contains(pattern) {
            return Err(AppError::ValidationError(
                "Password contains common weak patterns".to_string(),
            ));
        }
    }
    
    Ok(())
}

// Helper function to track failed login attempts
async fn track_failed_login_attempt(state: &AppState, username: &str) -> AppResult<()> {
    let failed_attempts_key = format!("failed_attempts:{}", username);
    let mut conn = state.cache_conn.clone();
    
    // Increment failed attempts counter
    let current_attempts: i64 = crate::cache::cmd("INCR")
        .arg(&failed_attempts_key)
        .query_async::<String>(&mut conn)
        .await
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    
    // Set expiration for the counter (reset after lockout period)
    let _ = crate::cache::cmd("EXPIRE")
        .arg(&failed_attempts_key)
        .arg((state.config.login_lockout_duration_minutes * 60) as i32)
        .query_async::<()>(&mut conn)
        .await;
    
    // If exceeded max attempts, lock the account
    if current_attempts >= state.config.max_failed_login_attempts as i64 {
        let lockout_key = format!("lockout:{}", username);
        let lockout_until = chrono::Utc::now().timestamp() + (state.config.login_lockout_duration_minutes * 60) as i64;
        
        let _ = crate::cache::cmd("SET")
            .arg(&lockout_key)
            .arg(lockout_until.to_string())
            .query_async::<()>(&mut conn)
            .await;
        
        tracing::warn!("Account locked due to too many failed attempts: {}", username);
    }
    
    Ok(())
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    // Check for account lockout
    let lockout_key = format!("lockout:{}", req.username);
    let mut conn = state.cache_conn.clone();
    
    if let Ok(lockout_time) = crate::cache::cmd("GET")
        .arg(&lockout_key)
        .query_async::<String>(&mut conn)
        .await 
    {
        if let Ok(lockout_timestamp) = lockout_time.parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            if now < lockout_timestamp {
                let remaining = lockout_timestamp - now;
                return Err(AppError::ValidationError(
                    format!("Account locked. Try again in {} seconds", remaining)
                ));
            } else {
                // Lockout expired, remove it
                let _ = crate::cache::cmd("DEL")
                    .arg(&lockout_key)
                    .query_async::<()>(&mut conn)
                    .await;
            }
        }
    }
    // Get user from database
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let user_row = sqlx::query(
        "SELECT id, username, email, password_hash, created_at, updated_at, metadata
         FROM users WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let user_row = user_row.ok_or_else(|| {
        // Track failed attempt for non-existent user
        let _ = tokio::spawn({
            let username = req.username.clone();
            let state = state.clone();
            async move {
                track_failed_login_attempt(&state, &username).await;
            }
        });
        AppError::Unauthorized
    })?;

    // Verify password
    let password_hash: String = user_row.get::<String, _>("password_hash");
    let parsed_hash = PasswordHash::new(&password_hash).map_err(|_| AppError::InternalServerError)?;

    let argon2 = Argon2::default();
    let password_valid = argon2
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !password_valid {
        // Track failed attempt for existing user with wrong password
        track_failed_login_attempt(&state, &req.username).await?;
        crate::monitoring::global_metrics().record_auth(false);
        return Err(AppError::Unauthorized);
    }

    // Clear failed attempts on successful login
    let failed_attempts_key = format!("failed_attempts:{}", req.username);
    let mut conn = state.cache_conn.clone();
    let _ = crate::cache::cmd("DEL")
        .arg(&failed_attempts_key)
        .query_async::<()>(&mut conn)
        .await;

    // Record successful authentication
    crate::monitoring::global_metrics().record_auth(true);

    let user_id_bytes: Vec<u8> = user_row.get::<Vec<u8>, _>("id");
    let user_id = Uuid::from_bytes(user_id_bytes.try_into().map_err(|_| AppError::InternalServerError)?);

    // Get user role and permissions from metadata
    let metadata_str: Option<String> = user_row.get("metadata");
    let metadata: serde_json::Value = metadata_str
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    
    let role = metadata.get("role")
        .and_then(|r| r.as_str())
        .unwrap_or("user")
        .to_string();
        
    let permissions = metadata.get("permissions")
        .and_then(|p| p.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec!["read_own_data".to_string()]);
    
    // Generate session ID for tracking
    let session_id = Uuid::new_v4().to_string();

    // Generate tokens
    let now = Utc::now();
    let access_exp = now + Duration::hours(state.config.jwt_expiration_hours);
    let refresh_exp = now + Duration::days(state.config.refresh_token_expiration_days);

    let access_claims = Claims {
        sub: user_id,
        username: req.username.clone(),
        exp: access_exp.timestamp(),
        iat: now.timestamp(),
        role: role.clone(),
        permissions: permissions.clone(),
        session_id: Some(session_id.clone()),
    };

    let refresh_claims = Claims {
        sub: user_id,
        username: req.username.clone(),
        exp: refresh_exp.timestamp(),
        iat: now.timestamp(),
        role,
        permissions,
        session_id: Some(session_id.clone()),
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
    crate::cache::cmd("SETEX")
        .arg(&refresh_key)
        .arg(30 * 24 * 3600) // 30 days TTL
        .arg(&refresh_token)
        .query_async::<()>(&mut conn)
        .await
        .ok();

    // Update last active timestamp
    let user_id_bytes = user_id.as_bytes();
    let mut conn_update = state.db_pool.acquire().await.ok();
    if let Some(mut conn) = conn_update {
        sqlx::query("UPDATE users SET updated_at = ? WHERE id = ?")
            .bind(now)
            .bind(user_id_bytes)
            .execute(&mut *conn)
            .await
            .ok();
    }

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
    )
    .await
    .ok();

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
        .ok_or(AppError::ValidationError(
            "refresh_token required".to_string(),
        ))?;

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
    let stored_token: Option<String> = crate::cache::cmd("GET")
        .arg(&refresh_key)
        .query_async::<String>(&mut conn)
        .await
        .ok();

    if stored_token.as_ref() != Some(&refresh_token.to_string()) {
        return Err(AppError::Unauthorized);
    }

    // Generate new access token with existing session
    let now = Utc::now();
    let access_exp = now + Duration::hours(state.config.jwt_expiration_hours);

    let access_claims = Claims {
        sub: user_id,
        username: token_data.claims.username.clone(),
        exp: access_exp.timestamp(),
        iat: now.timestamp(),
        role: token_data.claims.role.clone(),
        permissions: token_data.claims.permissions.clone(),
        session_id: token_data.claims.session_id.clone(),
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

    // Remove refresh token from cache
    let refresh_key = format!("refresh_token:{}", user_id);
    let mut conn = state.redis_conn.clone();
    crate::cache::cmd("DEL")
        .arg(&refresh_key)
        .query_async::<()>(&mut conn)
        .await
        .ok();

    // Blacklist current session to invalidate all tokens with this session_id
    if let Some(session_id) = &claims.session_id {
        let blacklist_key = format!("blacklist:session:{}", session_id);
        let remaining_ttl = claims.exp - chrono::Utc::now().timestamp();
        if remaining_ttl > 0 {
            crate::cache::cmd("SETEX")
                .arg(&blacklist_key)
                .arg(remaining_ttl)
                .arg("1")
                .query_async::<()>(&mut conn)
                .await
                .ok();
        }
    }

    // Log audit event
    AuditService::log_event(
        &state.db_pool,
        Some(user_id),
        "logout".to_string(),
        "auth".to_string(),
        user_id.to_string(),
        Some(serde_json::json!({
            "session_id": claims.session_id,
            "role": claims.role
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    claims: axum::Extension<Claims>,
) -> AppResult<Json<User>> {
    let user_id = claims.sub;
    let user_id_bytes = user_id.as_bytes();

    // Get user from database
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let user_row = sqlx::query(
        "SELECT id, username, email, password_hash, created_at, updated_at, metadata
         FROM users WHERE id = ?"
    )
    .bind(user_id_bytes)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let user_row = user_row.ok_or(AppError::NotFound("User not found".to_string()))?;

    let user = User {
        id: user_id,
        username: user_row.get("username"),
        email: user_row.get("email"),
        password_hash: user_row.get("password_hash"),
        created_at: user_row.get("created_at"),
        updated_at: user_row.get("updated_at"),
        metadata: user_row
            .get::<Option<String>, _>("metadata")
            .and_then(|s| serde_json::from_str(&s).ok()),
    };

    Ok(Json(user))
}
