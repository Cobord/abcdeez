use apple_signin::{AppleJwtClient, IdInfo};
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::auth::jwt::{generate_jwt, Claims};
use crate::models::user::{User, UserRole};

#[derive(Debug, Deserialize)]
pub struct AppleSignInRequest {
    /// The identity token from Apple Sign-In
    pub identity_token: String,
    /// The authorization code (optional, for server-to-server validation)
    pub authorization_code: Option<String>,
    /// User info from Apple (only provided on first sign-in)
    pub user_info: Option<AppleUserInfo>,
}

#[derive(Debug, Deserialize)]
pub struct AppleUserInfo {
    pub email: Option<String>,
    pub name: Option<AppleUserName>,
}

#[derive(Debug, Deserialize)]
pub struct AppleUserName {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AppleSignInResponse {
    pub token: String,
    pub user: UserResponse,
    pub is_new_user: bool,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub apple_id: String,
}

pub struct AppleAuthConfig {
    /// Your app's bundle ID (e.g., "com.example.app")
    pub client_id: String,
    /// Your Apple Team ID
    pub team_id: String,
    /// Your Apple Key ID for Sign in with Apple
    pub key_id: String,
    /// The private key content (P256 format)
    pub private_key: String,
}

impl AppleAuthConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            client_id: std::env::var("APPLE_CLIENT_ID")
                .unwrap_or_else(|_| "online.fg-goose.abcdeez".to_string()),
            team_id: std::env::var("APPLE_TEAM_ID")
                .unwrap_or_else(|_| "992772WV7K".to_string()),
            key_id: std::env::var("APPLE_KEY_ID")
                .unwrap_or_else(|_| "".to_string()),
            private_key: std::env::var("APPLE_PRIVATE_KEY")
                .unwrap_or_else(|_| "".to_string()),
        })
    }
}

pub async fn handle_apple_signin(
    Extension(pool): Extension<PgPool>,
    Extension(apple_config): Extension<Arc<AppleAuthConfig>>,
    Json(payload): Json<AppleSignInRequest>,
) -> impl IntoResponse {
    debug!("Processing Apple Sign-In request");

    // Create Apple JWT client for token validation
    let client = match create_apple_client(&apple_config) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create Apple client: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Authentication service configuration error"
                })),
            );
        }
    };

    // Validate the identity token
    let id_info = match validate_identity_token(&client, &payload.identity_token).await {
        Ok(info) => info,
        Err(e) => {
            warn!("Invalid Apple identity token: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid identity token"
                })),
            );
        }
    };

    // Extract user information
    let apple_id = id_info.sub;
    let email = id_info.email.unwrap_or_else(|| {
        // If email is not in token, try to get from user_info
        payload.user_info
            .as_ref()
            .and_then(|ui| ui.email.clone())
            .unwrap_or_else(|| format!("{}@privaterelay.appleid.com", apple_id))
    });

    let name = payload.user_info.as_ref().and_then(|ui| {
        ui.name.as_ref().and_then(|n| {
            match (&n.first_name, &n.last_name) {
                (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                (Some(first), None) => Some(first.clone()),
                (None, Some(last)) => Some(last.clone()),
                (None, None) => None,
            }
        })
    });

    // Check if user exists or create new one
    let (user, is_new_user) = match get_or_create_apple_user(
        &pool,
        apple_id.clone(),
        email.clone(),
        name.clone(),
    ).await {
        Ok(result) => result,
        Err(e) => {
            error!("Database error during Apple Sign-In: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to process sign-in"
                })),
            );
        }
    };

    // Generate JWT token
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
    };

    let token = match generate_jwt(&claims) {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate JWT: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to generate authentication token"
                })),
            );
        }
    };

    info!(
        user_id = %user.id,
        is_new_user = is_new_user,
        "Apple Sign-In successful"
    );

    (
        StatusCode::OK,
        Json(serde_json::json!(AppleSignInResponse {
            token,
            user: UserResponse {
                id: user.id,
                email: user.email,
                name: user.name,
                apple_id: user.apple_id.unwrap_or_default(),
            },
            is_new_user,
        })),
    )
}

fn create_apple_client(config: &AppleAuthConfig) -> Result<AppleJwtClient, String> {
    // For now, we'll use the public key validation method
    // In production, you'd also set up the private key for server-to-server validation
    
    let client = AppleJwtClient::new(&[
        &config.client_id,
        // Also accept the service ID if different from app ID
        "online.fg-goose.abcdeez.service",
    ]);
    
    Ok(client)
}

async fn validate_identity_token(
    client: &AppleJwtClient,
    token: &str,
) -> Result<IdInfo, String> {
    // Validate and decode the identity token
    client
        .decode(token)
        .await
        .map_err(|e| format!("Token validation failed: {:?}", e))?
        .id_info()
        .ok_or_else(|| "No identity info in token".to_string())
}

async fn get_or_create_apple_user(
    pool: &PgPool,
    apple_id: String,
    email: String,
    name: Option<String>,
) -> Result<(User, bool), sqlx::Error> {
    // First, try to find existing user by Apple ID
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE apple_id = $1"
    )
    .bind(&apple_id)
    .fetch_optional(pool)
    .await?;

    if let Some(user) = existing_user {
        debug!("Found existing user with Apple ID: {}", apple_id);
        return Ok((user, false));
    }

    // Check if user exists with same email (might have signed up differently)
    let existing_email_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&email)
    .fetch_optional(pool)
    .await?;

    if let Some(mut user) = existing_email_user {
        // Update existing user with Apple ID
        debug!("Linking existing email user with Apple ID: {}", apple_id);
        
        sqlx::query(
            "UPDATE users SET apple_id = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(&apple_id)
        .bind(&user.id)
        .execute(pool)
        .await?;
        
        user.apple_id = Some(apple_id);
        return Ok((user, false));
    }

    // Create new user
    info!("Creating new user from Apple Sign-In: {}", email);
    
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, email, name, apple_id, role, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&email)
    .bind(&name)
    .bind(&apple_id)
    .bind(UserRole::User.to_string())
    .fetch_one(pool)
    .await?;

    Ok((user, true))
}

/// Verify Apple user's current credential state (optional)
pub async fn verify_credential_state(
    Extension(pool): Extension<PgPool>,
    Extension(apple_config): Extension<Arc<AppleAuthConfig>>,
    Json(payload): Json<VerifyCredentialRequest>,
) -> impl IntoResponse {
    // This endpoint can be used to check if a user's Apple ID is still valid
    // Useful for re-authentication or checking account status
    
    debug!("Verifying Apple credential state for user: {}", payload.user_id);
    
    // In a full implementation, you would:
    // 1. Make a server-to-server call to Apple
    // 2. Check the credential state
    // 3. Update user status if needed
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "valid": true,
            "state": "authorized"
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct VerifyCredentialRequest {
    pub user_id: Uuid,
    pub apple_id: String,
}