use std::collections::HashMap;
use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use url::Url;

use crate::{
    config::Config,
    error::{AppError, AppResult},
};

/// GitHub OAuth2 Token Response
#[derive(Debug, Deserialize)]
pub struct GitHubTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_in: Option<u64>,
}

/// GitHub User Profile
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
    pub bio: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub blog: Option<String>,
    pub twitter_username: Option<String>,
    pub public_repos: u32,
    pub public_gists: u32,
    pub followers: u32,
    pub following: u32,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
    pub type_field: String,
    pub site_admin: bool,
    pub hireable: Option<bool>,
}

/// GitHub Primary Email
#[derive(Debug, Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
    pub visibility: Option<String>,
}

/// GitHub OAuth Service
pub struct GitHubOAuthService {
    config: Arc<Config>,
    http_client: Client,
}

impl GitHubOAuthService {
    /// Create new GitHub OAuth Service
    pub fn new(config: Arc<Config>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Learning-App-Backend/1.0")
            .build()
            .unwrap_or_default();

        Self {
            config,
            http_client,
        }
    }

    /// Generate GitHub authorization URL
    pub fn get_authorization_url(&self, state: &str) -> AppResult<String> {
        let mut auth_url = Url::parse("https://github.com/login/oauth/authorize").map_err(|e| {
            error!("Failed to parse GitHub auth URL: {}", e);
            AppError::InternalServerError
        })?;

        {
            let mut query_pairs = auth_url.query_pairs_mut();
            query_pairs.append_pair("client_id", &self.config.github_client_id);
            query_pairs.append_pair("redirect_uri", &self.config.github_redirect_uri);
            query_pairs.append_pair("scope", "user:email");
            query_pairs.append_pair("state", state);
            query_pairs.append_pair("allow_signup", "true");
        }

        Ok(auth_url.to_string())
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(
        &self,
        code: &str,
        state: &str,
    ) -> AppResult<GitHubTokenResponse> {
        let mut params = HashMap::new();
        params.insert("client_id", self.config.github_client_id.as_str());
        params.insert("client_secret", self.config.github_client_secret.as_str());
        params.insert("code", code);
        params.insert("redirect_uri", self.config.github_redirect_uri.as_str());
        params.insert("state", state);

        let response = self
            .http_client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to exchange GitHub code for token: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            error!(
                "GitHub token exchange failed with status: {}",
                response.status()
            );
            let error_text = response.text().await.unwrap_or_default();
            error!("GitHub error response: {}", error_text);
            return Err(AppError::AuthenticationError(
                "Failed to authenticate with GitHub".to_string(),
            ));
        }

        let token_response: GitHubTokenResponse = response.json().await.map_err(|e| {
            error!("Failed to parse GitHub token response: {}", e);
            AppError::AuthenticationError("Invalid response from GitHub".to_string())
        })?;

        info!("Successfully exchanged GitHub authorization code for access token");
        Ok(token_response)
    }

    /// Get GitHub user profile
    pub async fn get_user_profile(&self, access_token: &str) -> AppResult<GitHubUser> {
        let response = self
            .http_client
            .get("https://api.github.com/user")
            .bearer_auth(access_token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch GitHub user profile: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            error!(
                "GitHub user profile request failed with status: {}",
                response.status()
            );
            let error_text = response.text().await.unwrap_or_default();
            error!("GitHub error response: {}", error_text);
            return Err(AppError::AuthenticationError(
                "Failed to fetch user profile from GitHub".to_string(),
            ));
        }

        let user_profile: GitHubUser = response.json().await.map_err(|e| {
            error!("Failed to parse GitHub user profile: {}", e);
            AppError::InternalServerError
        })?;

        info!(
            "Successfully fetched GitHub user profile for user: {}",
            user_profile.login
        );
        Ok(user_profile)
    }

    /// Get GitHub user's primary verified email
    pub async fn get_user_primary_email(&self, access_token: &str) -> AppResult<Option<String>> {
        let response = self
            .http_client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch GitHub user emails: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            error!(
                "GitHub user emails request failed with status: {}",
                response.status()
            );
            return Ok(None);
        }

        let emails: Vec<GitHubEmail> = response.json().await.map_err(|e| {
            error!("Failed to parse GitHub user emails: {}", e);
            AppError::InternalServerError
        })?;

        // Find primary verified email
        let primary_email = emails
            .iter()
            .find(|email| email.primary && email.verified)
            .map(|email| email.email.clone());

        if let Some(ref email) = primary_email {
            info!("Found primary verified email for GitHub user: {}", email);
        } else {
            warn!("No primary verified email found for GitHub user");
        }

        Ok(primary_email)
    }

    /// Validate GitHub access token (check if still valid)
    pub async fn validate_access_token(&self, access_token: &str) -> AppResult<bool> {
        let response = self
            .http_client
            .get("https://api.github.com/user")
            .bearer_auth(access_token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to validate GitHub access token: {}", e);
                AppError::InternalServerError
            })?;

        match response.status().as_u16() {
            200 => {
                info!("GitHub access token is valid");
                Ok(true)
            }
            401 => {
                info!("GitHub access token is invalid or expired");
                Ok(false)
            }
            _ => {
                warn!(
                    "Unexpected status from GitHub token validation: {}",
                    response.status()
                );
                Ok(false)
            }
        }
    }

    /// Revoke GitHub access token
    pub async fn revoke_access_token(&self, access_token: &str) -> AppResult<()> {
        let response = self
            .http_client
            .delete(&format!(
                "https://api.github.com/applications/{}/grant",
                self.config.github_client_id
            ))
            .basic_auth(
                &self.config.github_client_id,
                Some(&self.config.github_client_secret),
            )
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&serde_json::json!({
                "access_token": access_token
            }))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to revoke GitHub access token: {}", e);
                AppError::InternalServerError
            })?;

        match response.status().as_u16() {
            204 => {
                info!("Successfully revoked GitHub access token");
                Ok(())
            }
            404 => {
                info!("GitHub access token was already revoked or doesn't exist");
                Ok(())
            }
            _ => {
                warn!(
                    "Unexpected status from GitHub token revocation: {}",
                    response.status()
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    /// Get rate limit information
    pub async fn get_rate_limit(&self, access_token: &str) -> AppResult<serde_json::Value> {
        let response = self
            .http_client
            .get("https://api.github.com/rate_limit")
            .bearer_auth(access_token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch GitHub rate limit: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            error!(
                "GitHub rate limit request failed with status: {}",
                response.status()
            );
            return Err(AppError::InternalServerError);
        }

        let rate_limit: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse GitHub rate limit response: {}", e);
            AppError::InternalServerError
        })?;

        Ok(rate_limit)
    }

    /// Generate a secure state parameter for OAuth flow
    pub fn generate_state() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                 abcdefghijklmnopqrstuvwxyz\
                                 0123456789";
        const STATE_LEN: usize = 32;

        let mut rng = rand::thread_rng();
        (0..STATE_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_state_generation() {
        let state1 = GitHubOAuthService::generate_state();
        let state2 = GitHubOAuthService::generate_state();

        assert_eq!(state1.len(), 32);
        assert_eq!(state2.len(), 32);
        assert_ne!(state1, state2); // Should be different

        // Should only contain alphanumeric characters
        assert!(state1.chars().all(|c| c.is_ascii_alphanumeric()));
        assert!(state2.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_github_user_profile_parsing() {
        let json_data = r#"
        {
            "id": 12345,
            "login": "testuser",
            "name": "Test User",
            "email": "test@example.com",
            "avatar_url": "https://github.com/images/error/testuser_happy.gif",
            "bio": "Test bio",
            "company": "Test Company",
            "location": "Test Location",
            "blog": "https://testuser.dev",
            "twitter_username": "testuser",
            "public_repos": 10,
            "public_gists": 5,
            "followers": 100,
            "following": 50,
            "created_at": "2021-01-01T00:00:00Z",
            "updated_at": "2021-01-01T00:00:00Z",
            "html_url": "https://github.com/testuser",
            "type": "User",
            "site_admin": false,
            "hireable": true
        }
        "#;

        let user: GitHubUser = serde_json::from_str(json_data).unwrap();

        assert_eq!(user.id, 12345);
        assert_eq!(user.login, "testuser");
        assert_eq!(user.name, Some("Test User".to_string()));
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.public_repos, 10);
        assert!(!user.site_admin);
        assert_eq!(user.hireable, Some(true));
    }
}
