use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>, // Optional for OAuth users
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    
    // OAuth provider fields
    pub apple_user_id: Option<String>,
    pub github_user_id: Option<String>,
    pub oauth_provider_id: Option<String>, // Primary OAuth provider ID
    pub auth_provider: String, // "local", "apple", "github"
    pub is_private_email: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse, // Include user info in token response
}

/// OAuth-specific request structures
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
    pub state: String,
    pub provider: String,
}

#[derive(Debug, Deserialize)]
pub struct AppleSignInRequest {
    pub identity_token: String,
    pub authorization_code: Option<String>,
    pub user_info: Option<AppleUserInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppleUserInfo {
    pub name: Option<AppleUserName>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppleUserName {
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
}

/// OAuth authorization URL response
#[derive(Debug, Serialize)]
pub struct OAuthAuthUrlResponse {
    pub authorization_url: String,
    pub state: String,
    pub provider: String,
}