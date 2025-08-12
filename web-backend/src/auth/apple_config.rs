// Apple Sign In Configuration for ABCDEEZ
//
// Bundle ID: 992772WV7K.online.fg-goose.abcdeez
// Team ID: 992772WV7K
// App ID: online.fg-goose.abcdeez

use serde::{Deserialize, Serialize};
use std::env;

/// Apple Sign In configuration for different environments
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppleSignInConfig {
    /// Team ID from Apple Developer Account
    pub team_id: String,
    /// Bundle ID / App ID
    pub bundle_id: String,
    /// Service ID for web authentication (if different from bundle ID)
    pub service_id: String,
    /// Key ID for Sign in with Apple
    pub key_id: String,
    /// Private key for generating client secrets
    pub private_key: String,
    /// Redirect URIs for OAuth flow (web authentication)
    pub redirect_uris: Vec<String>,
    /// Server-to-server notification endpoint
    pub notification_endpoint: String,
    /// Native app signin endpoint (receives tokens from iOS/Android apps)
    pub native_signin_endpoint: String,
    /// Environment (staging/production)
    pub environment: Environment,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Environment {
    Staging,
    Production,
}

impl AppleSignInConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let environment = match env::var("ENVIRONMENT").unwrap_or_else(|_| "staging".to_string()).as_str() {
            "production" | "prod" => Environment::Production,
            _ => Environment::Staging,
        };

        let (redirect_uris, notification_endpoint, native_signin_endpoint) = match environment {
            Environment::Production => (
                vec![
                    // OAuth callback for web authentication flow
                    "https://api.abcdeez.fg-goose.online/v1/auth/oauth/callback?provider=apple".to_string(),
                ],
                "https://api.abcdeez.fg-goose.online/v1/auth/apple/notifications".to_string(),
                // Native app posts tokens here (not an OAuth redirect)
                "https://api.abcdeez.fg-goose.online/v1/auth/apple/signin".to_string(),
            ),
            Environment::Staging => (
                vec![
                    // OAuth callback for web authentication flow
                    "https://api.staging.abcdeez.fg-goose.online/v1/auth/oauth/callback?provider=apple".to_string(),
                ],
                "https://api.staging.abcdeez.fg-goose.online/v1/auth/apple/notifications".to_string(),
                // Native app posts tokens here (not an OAuth redirect)
                "https://api.staging.abcdeez.fg-goose.online/v1/auth/apple/signin".to_string(),
            ),
        };

        Ok(Self {
            team_id: env::var("APPLE_TEAM_ID").unwrap_or_else(|_| "992772WV7K".to_string()),
            bundle_id: "online.fg-goose.abcdeez".to_string(),
            service_id: env::var("APPLE_SERVICE_ID").unwrap_or_else(|_| "online.fg-goose.abcdeez".to_string()),
            key_id: env::var("APPLE_KEY_ID")?,
            private_key: env::var("APPLE_PRIVATE_KEY")?,
            redirect_uris,
            notification_endpoint,
            native_signin_endpoint,
            environment,
        })
    }

    /// Get the base API URL for the current environment
    pub fn api_base_url(&self) -> &str {
        match self.environment {
            Environment::Production => "https://api.abcdeez.fg-goose.online",
            Environment::Staging => "https://api.staging.abcdeez.fg-goose.online",
        }
    }

    /// Get the web app URL for the current environment
    pub fn web_app_url(&self) -> &str {
        match self.environment {
            Environment::Production => "https://abcdeez.fg-goose.online",
            Environment::Staging => "https://staging.abcdeez.fg-goose.online",
        }
    }

    /// Generate client secret for Apple Sign In
    /// This needs to be generated dynamically as it expires after 6 months
    pub fn generate_client_secret(&self) -> Result<String, Box<dyn std::error::Error>> {
        use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
        use chrono::{Utc, Duration};
        
        #[derive(Debug, Serialize)]
        struct Claims {
            iss: String,  // Team ID
            iat: i64,     // Issued at
            exp: i64,     // Expiration (max 6 months)
            aud: String,  // https://appleid.apple.com
            sub: String,  // Service ID or Bundle ID
        }

        let now = Utc::now();
        let exp = now + Duration::days(180); // 6 months

        let claims = Claims {
            iss: self.team_id.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            aud: "https://appleid.apple.com".to_string(),
            sub: self.service_id.clone(),
        };

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());

        let encoding_key = EncodingKey::from_ec_pem(self.private_key.as_bytes())?;
        let token = encode(&header, &claims, &encoding_key)?;

        Ok(token)
    }
}

/// URLs for Apple Sign In
pub struct AppleUrls;

impl AppleUrls {
    /// Apple's authorization endpoint
    pub const AUTHORIZE: &'static str = "https://appleid.apple.com/auth/authorize";
    
    /// Apple's token endpoint
    pub const TOKEN: &'static str = "https://appleid.apple.com/auth/token";
    
    /// Apple's public keys for JWT verification
    pub const PUBLIC_KEYS: &'static str = "https://appleid.apple.com/auth/keys";
    
    /// Apple's token validation endpoint
    pub const VALIDATE: &'static str = "https://appleid.apple.com/auth/token/validate";
    
    /// Apple's revoke token endpoint
    pub const REVOKE: &'static str = "https://appleid.apple.com/auth/revoke";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_urls() {
        let config = AppleSignInConfig {
            team_id: "992772WV7K".to_string(),
            bundle_id: "online.fg-goose.abcdeez".to_string(),
            service_id: "online.fg-goose.abcdeez".to_string(),
            key_id: "TEST_KEY".to_string(),
            private_key: "TEST_PRIVATE_KEY".to_string(),
            redirect_uris: vec![],
            notification_endpoint: String::new(),
            environment: Environment::Production,
        };

        assert_eq!(config.api_base_url(), "https://api.abcdeez.fg-goose.online");
        assert_eq!(config.web_app_url(), "https://abcdeez.fg-goose.online");
    }

    #[test]
    fn test_staging_environment() {
        let config = AppleSignInConfig {
            team_id: "992772WV7K".to_string(),
            bundle_id: "online.fg-goose.abcdeez".to_string(),
            service_id: "online.fg-goose.abcdeez".to_string(),
            key_id: "TEST_KEY".to_string(),
            private_key: "TEST_PRIVATE_KEY".to_string(),
            redirect_uris: vec![],
            notification_endpoint: String::new(),
            environment: Environment::Staging,
        };

        assert_eq!(config.api_base_url(), "https://api.staging.abcdeez.fg-goose.online");
        assert_eq!(config.web_app_url(), "https://staging.abcdeez.fg-goose.online");
    }
}