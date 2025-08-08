use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    config::Config,
    error::{AppError, AppResult},
    services::{
        apple_auth_service::{AppleAuthService, AppleIdToken},
        github_oauth_service::{GitHubOAuthService, GitHubUser},
    },
    state::AppState,
};

/// Supported OAuth providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Apple,
    GitHub,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Apple => write!(f, "apple"),
            OAuthProvider::GitHub => write!(f, "github"),
        }
    }
}

impl std::str::FromStr for OAuthProvider {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "apple" => Ok(OAuthProvider::Apple),
            "github" => Ok(OAuthProvider::GitHub),
            _ => Err(AppError::ValidationError(format!("Unsupported OAuth provider: {}", s))),
        }
    }
}

/// Unified OAuth user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserProfile {
    pub provider: OAuthProvider,
    pub provider_user_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub is_private_email: bool,
    pub avatar_url: Option<String>,
    pub profile_url: Option<String>,
    pub raw_profile: serde_json::Value,
}

/// OAuth authentication request
#[derive(Debug, Deserialize)]
pub struct OAuthAuthRequest {
    pub provider: String,
    pub identity_token: Option<String>,     // For Apple Sign In
    pub authorization_code: Option<String>, // For GitHub/other OAuth2 flows
    pub state: Option<String>,              // For OAuth2 CSRF protection
    pub user_info: Option<serde_json::Value>, // Additional user info from client
}

/// OAuth authentication response  
#[derive(Debug, Serialize)]
pub struct OAuthAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: OAuthUserProfile,
}

/// OAuth credential validation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialState {
    Authorized,
    Revoked,
    NotFound,
    Unknown,
}

impl std::fmt::Display for CredentialState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialState::Authorized => write!(f, "authorized"),
            CredentialState::Revoked => write!(f, "revoked"),
            CredentialState::NotFound => write!(f, "not_found"),
            CredentialState::Unknown => write!(f, "unknown"),
        }
    }
}

/// Unified OAuth Service
pub struct OAuthService {
    config: Arc<Config>,
    apple_service: AppleAuthService,
    github_service: GitHubOAuthService,
}

impl OAuthService {
    /// Create new OAuth Service
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            apple_service: AppleAuthService::new(config.clone()),
            github_service: GitHubOAuthService::new(config.clone()),
            config,
        }
    }

    /// Generate authorization URL for OAuth provider
    pub fn get_authorization_url(&self, provider: OAuthProvider, state: &str) -> AppResult<String> {
        match provider {
            OAuthProvider::Apple => {
                // Apple Sign In uses the native iOS flow, not web redirect
                // But we can provide a web fallback URL
                let url = format!(
                    "https://appleid.apple.com/auth/authorize?client_id={}&redirect_uri={}&scope=name email&response_type=code&state={}",
                    urlencoding::encode(&self.config.apple_client_id),
                    urlencoding::encode(&self.config.apple_redirect_uri),
                    urlencoding::encode(state)
                );
                Ok(url)
            }
            OAuthProvider::GitHub => {
                self.github_service.get_authorization_url(state)
            }
        }
    }

    /// Authenticate user with OAuth provider
    pub async fn authenticate(&mut self, request: OAuthAuthRequest) -> AppResult<OAuthUserProfile> {
        let provider: OAuthProvider = request.provider.parse()?;

        match provider {
            OAuthProvider::Apple => {
                self.authenticate_apple(request).await
            }
            OAuthProvider::GitHub => {
                self.authenticate_github(request).await
            }
        }
    }

    /// Authenticate with Apple Sign In
    async fn authenticate_apple(&mut self, request: OAuthAuthRequest) -> AppResult<OAuthUserProfile> {
        let identity_token = request.identity_token
            .ok_or_else(|| AppError::ValidationError("identity_token required for Apple Sign In".to_string()))?;

        let apple_token = self.apple_service.verify_identity_token(&identity_token).await?;

        // Convert Apple token to unified profile
        let profile = self.convert_apple_token_to_profile(apple_token, request.user_info)?;

        info!("Successfully authenticated Apple user: {}", profile.username);
        Ok(profile)
    }

    /// Authenticate with GitHub OAuth
    async fn authenticate_github(&self, request: OAuthAuthRequest) -> AppResult<OAuthUserProfile> {
        let auth_code = request.authorization_code
            .ok_or_else(|| AppError::ValidationError("authorization_code required for GitHub OAuth".to_string()))?;
        
        let state = request.state
            .ok_or_else(|| AppError::ValidationError("state required for GitHub OAuth".to_string()))?;

        // Exchange code for access token
        let token_response = self.github_service.exchange_code_for_token(&auth_code, &state).await?;

        // Get user profile
        let github_user = self.github_service.get_user_profile(&token_response.access_token).await?;

        // Get primary email if not in profile
        let email = if github_user.email.is_some() {
            github_user.email.clone()
        } else {
            self.github_service.get_user_primary_email(&token_response.access_token).await?
        };

        // Convert GitHub user to unified profile
        let profile = self.convert_github_user_to_profile(github_user, email)?;

        info!("Successfully authenticated GitHub user: {}", profile.username);
        Ok(profile)
    }

    /// Convert Apple ID token to unified profile
    fn convert_apple_token_to_profile(
        &self,
        apple_token: AppleIdToken,
        user_info: Option<serde_json::Value>,
    ) -> AppResult<OAuthUserProfile> {
        // Extract display name from user_info if available
        let display_name = user_info
            .as_ref()
            .and_then(|info| info.get("name"))
            .and_then(|name| {
                if let (Some(first), Some(last)) = (
                    name.get("firstName").and_then(|f| f.as_str()),
                    name.get("lastName").and_then(|l| l.as_str())
                ) {
                    Some(format!("{} {}", first, last))
                } else {
                    name.get("firstName").and_then(|f| f.as_str()).map(|s| s.to_string())
                }
            });

        // Generate username from email or use Apple ID
        let username = apple_token.email
            .as_ref()
            .map(|email| {
                email.split('@')
                    .next()
                    .unwrap_or("apple_user")
                    .to_string()
            })
            .unwrap_or_else(|| format!("apple_{}", &apple_token.sub[..8]));

        let email_verified = apple_token.is_email_verified();
        let is_private_email = apple_token.is_private_email();
        let raw_profile = serde_json::to_value(&apple_token).unwrap_or_default();
        
        Ok(OAuthUserProfile {
            provider: OAuthProvider::Apple,
            provider_user_id: apple_token.sub,
            username,
            display_name,
            email: apple_token.email,
            email_verified,
            is_private_email,
            avatar_url: None, // Apple doesn't provide avatars
            profile_url: None, // Apple doesn't have public profiles
            raw_profile,
        })
    }

    /// Convert GitHub user to unified profile
    fn convert_github_user_to_profile(
        &self,
        github_user: GitHubUser,
        email: Option<String>,
    ) -> AppResult<OAuthUserProfile> {
        let raw_profile = serde_json::to_value(&github_user).unwrap_or_default();
        
        Ok(OAuthUserProfile {
            provider: OAuthProvider::GitHub,
            provider_user_id: github_user.id.to_string(),
            username: github_user.login,
            display_name: github_user.name,
            email,
            email_verified: true, // GitHub emails are verified when primary
            is_private_email: false, // GitHub doesn't use private relay
            avatar_url: Some(github_user.avatar_url),
            profile_url: Some(github_user.html_url),
            raw_profile,
        })
    }

    /// Validate OAuth credentials for a provider
    pub async fn validate_credentials(
        &self, 
        provider: OAuthProvider,
        access_token: &str,
    ) -> AppResult<CredentialState> {
        match provider {
            OAuthProvider::Apple => {
                // Apple doesn't provide a direct REST API for credential validation
                // This would typically be done through refresh token validation
                // or by re-authenticating with the client
                warn!("Apple credential validation not fully implemented");
                Ok(CredentialState::Unknown)
            }
            OAuthProvider::GitHub => {
                let is_valid = self.github_service.validate_access_token(access_token).await?;
                Ok(if is_valid {
                    CredentialState::Authorized
                } else {
                    CredentialState::Revoked
                })
            }
        }
    }

    /// Revoke OAuth credentials
    pub async fn revoke_credentials(
        &self,
        provider: OAuthProvider,
        access_token: &str,
    ) -> AppResult<()> {
        match provider {
            OAuthProvider::Apple => {
                // Apple credential revocation is handled client-side
                // We can't revoke from the server directly
                info!("Apple credential revocation must be done client-side");
                Ok(())
            }
            OAuthProvider::GitHub => {
                self.github_service.revoke_access_token(access_token).await
            }
        }
    }

    /// Record OAuth credential check result in database
    pub async fn record_credential_check(
        &self,
        app_state: &AppState,
        user_id: Uuid,
        provider: OAuthProvider,
        provider_user_id: &str,
        credential_state: CredentialState,
        error_details: Option<&str>,
    ) -> AppResult<()> {
        let mut conn = app_state.db_pool.acquire().await
            .map_err(|e| AppError::DatabaseError(e))?;

        let check_id = Uuid::new_v4();
        let now = Utc::now();
        let error_json = error_details.map(|e| serde_json::json!({"error": e}).to_string());

        sqlx::query(
            "INSERT OR REPLACE INTO oauth_credential_checks 
             (id, user_id, provider, provider_user_id, last_check_time, credential_state, error_details, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(check_id.as_bytes())
        .bind(user_id.as_bytes())
        .bind(provider.to_string())
        .bind(provider_user_id)
        .bind(now)
        .bind(credential_state.to_string())
        .bind(error_json)
        .bind(now)
        .bind(now)
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        info!(
            "Recorded OAuth credential check: user={}, provider={}, state={}",
            user_id, provider, credential_state
        );

        Ok(())
    }

    /// Get last credential check for a user and provider
    pub async fn get_last_credential_check(
        &self,
        app_state: &AppState,
        user_id: Uuid,
        provider: OAuthProvider,
    ) -> AppResult<Option<(DateTime<Utc>, CredentialState)>> {
        let mut conn = app_state.db_pool.acquire().await
            .map_err(|e| AppError::DatabaseError(e))?;

        let result = sqlx::query(
            "SELECT last_check_time, credential_state FROM oauth_credential_checks
             WHERE user_id = ? AND provider = ?
             ORDER BY last_check_time DESC LIMIT 1"
        )
        .bind(user_id.as_bytes())
        .bind(provider.to_string())
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        if let Some(row) = result {
            let last_check_time: DateTime<Utc> = row.try_get("last_check_time")
                .map_err(|e| AppError::DatabaseError(e))?;
            let state_str: String = row.try_get("credential_state")
                .map_err(|e| AppError::DatabaseError(e))?;
            
            let credential_state = match state_str.as_str() {
                "authorized" => CredentialState::Authorized,
                "revoked" => CredentialState::Revoked,
                "not_found" => CredentialState::NotFound,
                _ => CredentialState::Unknown,
            };

            Ok(Some((last_check_time, credential_state)))
        } else {
            Ok(None)
        }
    }

    /// Generate secure state parameter for OAuth flows
    pub fn generate_state() -> String {
        GitHubOAuthService::generate_state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_provider_parsing() {
        assert_eq!("apple".parse::<OAuthProvider>().unwrap(), OAuthProvider::Apple);
        assert_eq!("github".parse::<OAuthProvider>().unwrap(), OAuthProvider::GitHub);
        assert_eq!("APPLE".parse::<OAuthProvider>().unwrap(), OAuthProvider::Apple);
        assert!("invalid".parse::<OAuthProvider>().is_err());
    }

    #[test]
    fn test_oauth_provider_display() {
        assert_eq!(OAuthProvider::Apple.to_string(), "apple");
        assert_eq!(OAuthProvider::GitHub.to_string(), "github");
    }

    #[test]
    fn test_credential_state_display() {
        assert_eq!(CredentialState::Authorized.to_string(), "authorized");
        assert_eq!(CredentialState::Revoked.to_string(), "revoked");
        assert_eq!(CredentialState::NotFound.to_string(), "not_found");
        assert_eq!(CredentialState::Unknown.to_string(), "unknown");
    }
}