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
    models::{CreateUserRequest, LoginRequest, TokenResponse, User},
    state::AppState,
};

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<User>)> {
    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalServerError)?
        .to_string();

    // Create user in database
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    // TODO: Implement actual database insertion
    let user = User {
        id: user_id,
        username: req.username,
        email: req.email,
        password_hash,
        created_at: now,
        metadata: None,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    // TODO: Get user from database
    // For now, return a dummy token
    
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    let exp = now + Duration::hours(state.config.jwt_expiration_hours);

    let claims = Claims {
        sub: user_id,
        username: req.username.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )?;

    Ok(Json(TokenResponse {
        access_token: token,
        refresh_token: Uuid::new_v4().to_string(), // TODO: Implement refresh tokens
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiration_hours * 3600,
    }))
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(refresh_token): Json<String>,
) -> AppResult<Json<TokenResponse>> {
    // TODO: Implement refresh token logic
    Err(AppError::Unauthorized)
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
) -> AppResult<StatusCode> {
    // TODO: Invalidate refresh token
    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    claims: axum::Extension<Claims>,
) -> AppResult<Json<User>> {
    // TODO: Get user from database using claims.sub
    Err(AppError::NotFound)
}