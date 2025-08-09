use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use regex::Regex;
use sqlx::Row;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::user::{
        AppleSignInRequest, CreateUserRequest, LoginRequest, OAuthAuthUrlResponse,
        OAuthCallbackRequest, TokenResponse, User, UserResponse,
    },
    services::{
        audit::AuditService,
        oauth_service::{OAuthAuthRequest, OAuthProvider, OAuthService, OAuthUserProfile},
    },
    state::AppState,
};

// Email validation regex
static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
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
    if !EMAIL_REGEX.is_match(&req.email) || req.email.len() > 254 {
        return Err(AppError::ValidationError(
            "Invalid email format".to_string(),
        ));
    }

    // Check for username/email restrictions
    if req
        .username
        .chars()
        .any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
    {
        return Err(AppError::ValidationError(
            "Username can only contain letters, numbers, underscores and hyphens".to_string(),
        ));
    }

    // Check if user already exists
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
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
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id_bytes.as_ref())
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
        password_hash: Some(password_hash),
        apple_user_id: None,
        github_user_id: None,
        oauth_provider_id: None,
        auth_provider: "local".to_string(),
        is_private_email: Some(false),
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

    Ok((
        StatusCode::CREATED,
        Json(UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
            metadata: user.metadata,
        }),
    ))
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
    let has_special = password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

    if !has_upper || !has_lower || !has_digit || !has_special {
        return Err(AppError::ValidationError(
            "Password must contain at least one uppercase letter, lowercase letter, digit, and special character".to_string(),
        ));
    }

    // Check for common weak patterns
    let lower_password = password.to_lowercase();
    let weak_patterns = vec![
        "password",
        "123456",
        "qwerty",
        "abc123",
        "admin",
        "letmein",
        "welcome",
        "monkey",
        "dragon",
        "master",
        "shadow",
        "password123",
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
        let lockout_until = chrono::Utc::now().timestamp()
            + (state.config.login_lockout_duration_minutes * 60) as i64;

        let _ = crate::cache::cmd("SET")
            .arg(&lockout_key)
            .arg(lockout_until.to_string())
            .query_async::<()>(&mut conn)
            .await;

        tracing::warn!(
            "Account locked due to too many failed attempts: {}",
            username
        );
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
                return Err(AppError::ValidationError(format!(
                    "Account locked. Try again in {} seconds",
                    remaining
                )));
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
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let user_row = sqlx::query(
        "SELECT id, username, email, password_hash, created_at, updated_at, metadata
         FROM users WHERE username = ?",
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
    let parsed_hash =
        PasswordHash::new(&password_hash).map_err(|_| AppError::InternalServerError)?;

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
    let user_id = Uuid::from_bytes(
        user_id_bytes
            .try_into()
            .map_err(|_| AppError::InternalServerError)?,
    );

    // Get user role and permissions from metadata
    let metadata_str: Option<String> = user_row.get("metadata");
    let metadata: serde_json::Value = metadata_str
        .clone()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let role = metadata
        .get("role")
        .and_then(|r| r.as_str())
        .unwrap_or("user")
        .to_string();

    let permissions = metadata
        .get("permissions")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
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

    // Use explicit HS256 algorithm for security
    let header = Header::new(Algorithm::HS256);

    let access_token = encode(
        &header,
        &access_claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )?;

    let refresh_token = encode(
        &header,
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
            .bind(&user_id_bytes[..])
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
        user: UserResponse {
            id: user_id,
            username: req.username,
            email: user_row.get::<String, _>("email"),
            created_at: user_row.get::<chrono::DateTime<Utc>, _>("created_at"),
            updated_at: user_row.get::<chrono::DateTime<Utc>, _>("updated_at"),
            metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
        },
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
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.algorithms = vec![jsonwebtoken::Algorithm::HS256];

    let token_data = jsonwebtoken::decode::<Claims>(
        refresh_token,
        &jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
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
        &Header::new(Algorithm::HS256),
        &access_claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )?;

    // Get user details for response
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&user_id.as_bytes()[..])
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: refresh_token.to_string(),
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiration_hours * 3600,
        user: UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
            metadata: user.metadata,
        },
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
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let user_row = sqlx::query(
        "SELECT id, username, email, password_hash, created_at, updated_at, metadata
         FROM users WHERE id = ?",
    )
    .bind(&user_id_bytes[..])
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let user_row = user_row.ok_or(AppError::NotFound("User not found".to_string()))?;

    let user = User {
        id: user_id,
        username: user_row.get("username"),
        email: user_row.get("email"),
        password_hash: user_row.get("password_hash"),
        apple_user_id: user_row.get("apple_user_id"),
        github_user_id: user_row.get("github_user_id"),
        oauth_provider_id: user_row.get("oauth_provider_id"),
        auth_provider: user_row
            .get::<Option<String>, _>("auth_provider")
            .unwrap_or_else(|| "local".to_string()),
        is_private_email: user_row.get("is_private_email"),
        created_at: user_row.get("created_at"),
        updated_at: user_row.get("updated_at"),
        metadata: user_row
            .get::<Option<String>, _>("metadata")
            .and_then(|s| serde_json::from_str(&s).ok()),
    };

    Ok(Json(user))
}

// OAuth Authentication Endpoints

/// Get OAuth authorization URL for a provider
pub async fn oauth_authorization_url(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(provider): axum::extract::Path<String>,
) -> AppResult<Json<OAuthAuthUrlResponse>> {
    let oauth_provider: OAuthProvider = provider.parse().map_err(|_| {
        AppError::ValidationError(format!("Unsupported OAuth provider: {}", provider))
    })?;

    let oauth_service = OAuthService::new(state.config.clone());
    let oauth_state = OAuthService::generate_state();

    let authorization_url = oauth_service.get_authorization_url(oauth_provider, &oauth_state)?;

    // Store OAuth state in cache for CSRF protection
    let state_key = format!("oauth_state:{}", oauth_state);
    let mut conn = state.cache_conn.clone();
    let _: Result<(), _> = crate::cache::cmd("SETEX")
        .arg(&state_key)
        .arg(600) // 10 minutes
        .arg(
            serde_json::json!({
                "provider": oauth_provider,
                "created_at": chrono::Utc::now().timestamp()
            })
            .to_string(),
        )
        .query_async(&mut conn)
        .await;

    Ok(Json(OAuthAuthUrlResponse {
        authorization_url,
        state: oauth_state,
        provider: oauth_provider.to_string(),
    }))
}

/// Apple Sign In endpoint (native iOS flow)
pub async fn apple_signin(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AppleSignInRequest>,
) -> AppResult<Json<TokenResponse>> {
    let mut oauth_service = OAuthService::new(state.config.clone());

    let oauth_request = OAuthAuthRequest {
        provider: "apple".to_string(),
        identity_token: Some(req.identity_token),
        authorization_code: req.authorization_code,
        state: None,
        user_info: req.user_info.map(|info| serde_json::json!(info)),
    };

    // Authenticate with Apple
    let oauth_profile = oauth_service.authenticate(oauth_request).await?;

    // Create or get existing user
    let user = get_or_create_oauth_user(&state, &oauth_profile).await?;

    // Generate our app's JWT tokens
    let token_response = generate_tokens(&state, &user).await?;

    // Log successful OAuth sign in
    AuditService::log_event(
        &state.db_pool,
        Some(user.id),
        "oauth_signin".to_string(),
        "auth".to_string(),
        user.id.to_string(),
        Some(serde_json::json!({
            "provider": oauth_profile.provider,
            "username": oauth_profile.username
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(token_response))
}

/// OAuth callback endpoint (for web-based OAuth flows like GitHub)
pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OAuthCallbackRequest>,
) -> AppResult<Json<TokenResponse>> {
    // Validate OAuth state parameter
    let state_key = format!("oauth_state:{}", req.state);
    let mut conn = state.cache_conn.clone();
    let cached_state: Option<String> = crate::cache::cmd("GET")
        .arg(&state_key)
        .query_async(&mut conn)
        .await
        .ok();

    if cached_state.is_none() {
        return Err(AppError::AuthenticationError(
            "Invalid or expired OAuth state".to_string(),
        ));
    }

    // Remove used state token
    let _: Result<(), _> = crate::cache::cmd("DEL")
        .arg(&state_key)
        .query_async(&mut conn)
        .await;

    let mut oauth_service = OAuthService::new(state.config.clone());

    let oauth_request = OAuthAuthRequest {
        provider: req.provider.clone(),
        identity_token: None,
        authorization_code: Some(req.code),
        state: Some(req.state),
        user_info: None,
    };

    // Authenticate with OAuth provider
    let oauth_profile = oauth_service.authenticate(oauth_request).await?;

    // Create or get existing user
    let user = get_or_create_oauth_user(&state, &oauth_profile).await?;

    // Generate our app's JWT tokens
    let token_response = generate_tokens(&state, &user).await?;

    // Log successful OAuth sign in
    AuditService::log_event(
        &state.db_pool,
        Some(user.id),
        "oauth_signin".to_string(),
        "auth".to_string(),
        user.id.to_string(),
        Some(serde_json::json!({
            "provider": oauth_profile.provider,
            "username": oauth_profile.username
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(token_response))
}

/// Helper function to get or create OAuth user
async fn get_or_create_oauth_user(
    state: &AppState,
    oauth_profile: &OAuthUserProfile,
) -> AppResult<User> {
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    // Check if user exists by OAuth provider ID
    let existing_user = match oauth_profile.provider {
        OAuthProvider::Apple => {
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE apple_user_id = ?")
                .bind(&oauth_profile.provider_user_id)
                .fetch_optional(&mut *conn)
                .await
                .map_err(|e| AppError::DatabaseError(e))?
        }
        OAuthProvider::GitHub => {
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE github_user_id = ?")
                .bind(&oauth_profile.provider_user_id)
                .fetch_optional(&mut *conn)
                .await
                .map_err(|e| AppError::DatabaseError(e))?
        }
    };

    if let Some(user) = existing_user {
        // Update last login
        update_user_last_login(&state.db_pool, user.id).await?;
        return Ok(user);
    }

    // Create new OAuth user
    create_oauth_user(state, oauth_profile).await
}

/// Helper function to create new OAuth user
async fn create_oauth_user(state: &AppState, oauth_profile: &OAuthUserProfile) -> AppResult<User> {
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Ensure username is unique
    let username = ensure_unique_username(&state.db_pool, &oauth_profile.username).await?;
    let email = oauth_profile.email.clone().unwrap_or_default();

    // Create OAuth-specific metadata
    let metadata = serde_json::json!({
        "auth_provider": oauth_profile.provider,
        "oauth_profile": oauth_profile.raw_profile,
        "email_verified": oauth_profile.email_verified,
        "is_private_email": oauth_profile.is_private_email,
        "avatar_url": oauth_profile.avatar_url,
        "profile_url": oauth_profile.profile_url,
        "display_name": oauth_profile.display_name,
        "created_via_oauth": true
    });

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    // Insert new OAuth user with provider-specific fields
    let (apple_user_id, github_user_id) = match oauth_profile.provider {
        OAuthProvider::Apple => (Some(oauth_profile.provider_user_id.clone()), None),
        OAuthProvider::GitHub => (None, Some(oauth_profile.provider_user_id.clone())),
    };

    sqlx::query(
        "INSERT INTO users (
            id, username, email, password_hash, apple_user_id, github_user_id, 
            oauth_provider_id, auth_provider, is_private_email, created_at, updated_at, metadata
        ) VALUES (?, ?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(&username)
    .bind(&email)
    .bind(&apple_user_id)
    .bind(&github_user_id)
    .bind(&oauth_profile.provider_user_id)
    .bind(oauth_profile.provider.to_string())
    .bind(oauth_profile.is_private_email)
    .bind(now)
    .bind(now)
    .bind(metadata.to_string())
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    // Create the User object to return
    let user = User {
        id: user_id,
        username,
        email,
        password_hash: None,
        apple_user_id,
        github_user_id,
        oauth_provider_id: Some(oauth_profile.provider_user_id.clone()),
        auth_provider: oauth_profile.provider.to_string(),
        is_private_email: Some(oauth_profile.is_private_email),
        created_at: now,
        updated_at: now,
        metadata: Some(metadata),
    };

    info!(
        "Created new OAuth user: {} (provider: {})",
        user.username, oauth_profile.provider
    );

    Ok(user)
}

/// Helper function to ensure username uniqueness
async fn ensure_unique_username(
    db_pool: &crate::db::DbPool,
    base_username: &str,
) -> AppResult<String> {
    let mut conn = db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    let mut username = base_username.to_string();
    let mut counter = 1;

    loop {
        let exists = sqlx::query("SELECT COUNT(*) as count FROM users WHERE username = ?")
            .bind(&username)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| AppError::DatabaseError(e))?
            .get::<i64, _>("count")
            > 0;

        if !exists {
            return Ok(username);
        }

        username = format!("{}_{}", base_username, counter);
        counter += 1;

        if counter > 1000 {
            return Err(AppError::InternalServerError);
        }
    }
}

/// Helper function to update user's last login time
async fn update_user_last_login(db_pool: &crate::db::DbPool, user_id: Uuid) -> AppResult<()> {
    let mut conn = db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    sqlx::query("UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&user_id.as_bytes()[..])
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    Ok(())
}

/// Generate tokens with updated response format
async fn generate_tokens(state: &AppState, user: &User) -> AppResult<TokenResponse> {
    let now = chrono::Utc::now();
    let access_exp = now + chrono::Duration::hours(state.config.jwt_expiration_hours);
    let refresh_exp = now + chrono::Duration::days(state.config.refresh_token_expiration_days);

    // Create access token claims
    let access_claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        exp: access_exp.timestamp(),
        iat: now.timestamp(),
        role: "user".to_string(), // Default role
        permissions: vec!["read".to_string(), "write".to_string()], // Default permissions
        session_id: Some(format!("oauth_{}", Uuid::new_v4())),
    };

    // Create refresh token claims
    let refresh_claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        exp: refresh_exp.timestamp(),
        iat: now.timestamp(),
        role: "user".to_string(),
        permissions: vec!["refresh".to_string()],
        session_id: access_claims.session_id.clone(),
    };

    // Use explicit HS256 algorithm for security
    let header = Header::new(Algorithm::HS256);

    let access_token = encode(
        &header,
        &access_claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )?;

    let refresh_token = encode(
        &header,
        &refresh_claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )?;

    // Store refresh token in cache
    let refresh_key = format!("refresh_token:{}", user.id);
    let mut conn = state.cache_conn.clone();
    let _: Result<(), _> = crate::cache::cmd("SETEX")
        .arg(&refresh_key)
        .arg(state.config.refresh_token_expiration_days * 24 * 60 * 60)
        .arg(&refresh_token)
        .query_async(&mut conn)
        .await;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiration_hours * 3600,
        user: UserResponse {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
            metadata: user.metadata.clone(),
        },
    })
}
