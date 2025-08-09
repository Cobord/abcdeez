use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub refresh_token_expiration_days: i64,
    pub environment: Environment,
    pub log_level: String,
    pub cors_origin: String,
    pub rate_limit_requests: u32,
    pub rate_limit_window_seconds: u64,
    pub max_failed_login_attempts: u32,
    pub login_lockout_duration_minutes: u32,
    pub session_timeout_hours: u32,
    pub require_strong_passwords: bool,
    pub metrics_enabled: bool,
    pub tracing_endpoint: Option<String>,
    pub health_check_interval_seconds: u32,
    pub performance_monitoring_enabled: bool,
    
    // OAuth Providers configuration
    pub apple_client_id: String,
    pub apple_team_id: String,
    pub apple_key_id: String,
    pub apple_private_key_path: String,
    pub apple_redirect_uri: String,
    
    // GitHub OAuth configuration
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    
    // TLS/SSL configuration
    pub tls_domain: Option<String>,
    pub tls_use_letsencrypt: bool,
    pub tls_port: u16,
    pub admin_email: Option<String>,
    pub server_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenvy::dotenv().ok();

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://graph_learning.db".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a number"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET environment variable is required"),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a number"),
            refresh_token_expiration_days: env::var("REFRESH_TOKEN_EXPIRATION_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .expect("REFRESH_TOKEN_EXPIRATION_DAYS must be a number"),
            environment: env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .parse()
                .expect("Invalid environment"),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "debug".to_string()),
            cors_origin: env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "*".to_string()),
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .expect("RATE_LIMIT_REQUESTS must be a number"),
            rate_limit_window_seconds: env::var("RATE_LIMIT_WINDOW_SECONDS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .expect("RATE_LIMIT_WINDOW_SECONDS must be a number"),
            max_failed_login_attempts: env::var("MAX_FAILED_LOGIN_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("MAX_FAILED_LOGIN_ATTEMPTS must be a number"),
            login_lockout_duration_minutes: env::var("LOGIN_LOCKOUT_DURATION_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .expect("LOGIN_LOCKOUT_DURATION_MINUTES must be a number"),
            session_timeout_hours: env::var("SESSION_TIMEOUT_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("SESSION_TIMEOUT_HOURS must be a number"),
            require_strong_passwords: env::var("REQUIRE_STRONG_PASSWORDS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .expect("REQUIRE_STRONG_PASSWORDS must be a boolean"),
            metrics_enabled: env::var("METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .expect("METRICS_ENABLED must be a boolean"),
            tracing_endpoint: env::var("TRACING_ENDPOINT").ok(),
            health_check_interval_seconds: env::var("HEALTH_CHECK_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .expect("HEALTH_CHECK_INTERVAL_SECONDS must be a number"),
            performance_monitoring_enabled: env::var("PERFORMANCE_MONITORING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .expect("PERFORMANCE_MONITORING_ENABLED must be a boolean"),
            
            // OAuth Providers configuration (optional)
            apple_client_id: env::var("APPLE_CLIENT_ID")
                .unwrap_or_else(|_| "placeholder_apple_client_id".to_string()),
            apple_team_id: env::var("APPLE_TEAM_ID")
                .unwrap_or_else(|_| "placeholder_apple_team_id".to_string()),
            apple_key_id: env::var("APPLE_KEY_ID")
                .unwrap_or_else(|_| "placeholder_apple_key_id".to_string()),
            apple_private_key_path: env::var("APPLE_PRIVATE_KEY_PATH")
                .unwrap_or_else(|_| "/tmp/placeholder_apple_key.p8".to_string()),
            apple_redirect_uri: env::var("APPLE_REDIRECT_URI")
                .unwrap_or_else(|_| "https://api.yourapp.com/api/auth/apple/callback".to_string()),
            
            // GitHub OAuth configuration (optional)
            github_client_id: env::var("GITHUB_CLIENT_ID")
                .unwrap_or_else(|_| "placeholder_github_client_id".to_string()),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET")
                .unwrap_or_else(|_| "placeholder_github_client_secret".to_string()),
            github_redirect_uri: env::var("GITHUB_REDIRECT_URI")
                .unwrap_or_else(|_| "https://api.yourapp.com/api/auth/github/callback".to_string()),
            
            // TLS/SSL configuration
            tls_domain: env::var("TLS_DOMAIN").ok(),
            tls_use_letsencrypt: env::var("TLS_USE_LETSENCRYPT")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .expect("TLS_USE_LETSENCRYPT must be a boolean"),
            tls_port: env::var("TLS_PORT")
                .unwrap_or_else(|_| "443".to_string())
                .parse()
                .expect("TLS_PORT must be a number"),
            admin_email: env::var("ADMIN_EMAIL").ok(),
            server_name: env::var("SERVER_NAME")
                .unwrap_or_else(|_| "localhost".to_string()),
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
    
    pub fn validate_production_safety(&self) -> Result<(), String> {
        if !self.is_production() {
            return Ok(());
        }

        if self.cors_origin == "*" {
            return Err("Wildcard CORS not allowed in production".to_string());
        }

        if self.jwt_secret.len() < 32 {
            return Err("JWT secret too weak for production".to_string());
        }

        if self.jwt_secret == "development_secret_change_in_production" {
            return Err("Default JWT secret not allowed in production".to_string());
        }

        if self.database_url.starts_with("sqlite://") && !self.database_url.contains("?mode=ro") {
            return Err("SQLite databases should be read-only in production".to_string());
        }

        // Validate OAuth providers configuration in production
        if self.apple_client_id.is_empty() {
            return Err("Apple Client ID must be configured for production".to_string());
        }
        
        if self.apple_team_id.len() != 10 {
            return Err("Apple Team ID must be exactly 10 characters".to_string());
        }
        
        if self.apple_key_id.len() != 10 {
            return Err("Apple Key ID must be exactly 10 characters".to_string());
        }
        
        if !std::path::Path::new(&self.apple_private_key_path).exists() {
            return Err("Apple private key file not found".to_string());
        }
        
        if !self.apple_redirect_uri.starts_with("https://") {
            return Err("Apple redirect URI must use HTTPS in production".to_string());
        }
        
        if self.github_client_id.is_empty() {
            return Err("GitHub Client ID must be configured for production".to_string());
        }
        
        if self.github_client_secret.is_empty() {
            return Err("GitHub Client Secret must be configured for production".to_string());
        }
        
        if !self.github_redirect_uri.starts_with("https://") {
            return Err("GitHub redirect URI must use HTTPS in production".to_string());
        }

        Ok(())
    }
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "staging" | "stage" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(format!("Unknown environment: {}", s)),
        }
    }
}