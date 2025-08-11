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
    // Differential privacy configuration
    pub privacy_epsilon: f64,
    pub privacy_delta: f64,
    pub privacy_window_hours: i64,

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
    pub tls_enabled: bool,
    pub tls_domain: Option<String>,
    pub tls_use_letsencrypt: bool,
    pub tls_port: u16,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
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
    fn require_in_production(var_name: &str) -> String {
        env::var(var_name).unwrap_or_else(|_| {
            if env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .to_lowercase()
                == "production"
            {
                panic!("{} must be set in production environment", var_name);
            }
            String::new()
        })
    }

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
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "debug".to_string()),
            cors_origin: env::var("CORS_ORIGIN").unwrap_or_else(|_| {
                // Default to localhost in development, require explicit configuration otherwise
                if env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string())
                    .to_lowercase()
                    == "development"
                {
                    "http://localhost:3000".to_string()
                } else {
                    panic!("CORS_ORIGIN must be explicitly set in non-development environments");
                }
            }),
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
            privacy_epsilon: env::var("PRIVACY_EPSILON")
                .unwrap_or_else(|_| "1.0".to_string())
                .parse()
                .unwrap_or(1.0),
            privacy_delta: env::var("PRIVACY_DELTA")
                .unwrap_or_else(|_| "1e-9".to_string())
                .parse()
                .unwrap_or(1e-9),
            privacy_window_hours: env::var("PRIVACY_WINDOW_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),

            // OAuth Providers configuration
            // These values are optional in development but required in production
            apple_client_id: Self::require_in_production("APPLE_CLIENT_ID"),
            apple_team_id: Self::require_in_production("APPLE_TEAM_ID"),
            apple_key_id: Self::require_in_production("APPLE_KEY_ID"),
            apple_private_key_path: Self::require_in_production("APPLE_PRIVATE_KEY_PATH"),
            apple_redirect_uri: Self::require_in_production("APPLE_REDIRECT_URI"),

            // GitHub OAuth configuration
            github_client_id: Self::require_in_production("GITHUB_CLIENT_ID"),
            github_client_secret: Self::require_in_production("GITHUB_CLIENT_SECRET"),
            github_redirect_uri: Self::require_in_production("GITHUB_REDIRECT_URI"),

            // TLS/SSL configuration
            tls_enabled: env::var("TLS_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .expect("TLS_ENABLED must be a boolean"),
            tls_domain: env::var("TLS_DOMAIN").ok(),
            tls_use_letsencrypt: env::var("TLS_USE_LETSENCRYPT")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .expect("TLS_USE_LETSENCRYPT must be a boolean"),
            tls_port: env::var("TLS_PORT")
                .unwrap_or_else(|_| "443".to_string())
                .parse()
                .expect("TLS_PORT must be a number"),
            tls_cert_path: env::var("TLS_CERT_PATH").ok(),
            tls_key_path: env::var("TLS_KEY_PATH").ok(),
            admin_email: env::var("ADMIN_EMAIL").ok(),
            server_name: env::var("SERVER_NAME").unwrap_or_else(|_| "localhost".to_string()),
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

        if self.jwt_secret.len() < 64 {
            return Err("JWT secret must be at least 64 characters for production".to_string());
        }

        // Check for common weak patterns in JWT secret
        let jwt_lower = self.jwt_secret.to_lowercase();
        if jwt_lower.contains("secret")
            || jwt_lower.contains("password")
            || jwt_lower.contains("default")
            || jwt_lower.contains("admin")
            || jwt_lower.contains("test")
            || jwt_lower.contains("demo")
        {
            return Err("JWT secret contains weak patterns".to_string());
        }

        // Check for any placeholder or default values
        if self.jwt_secret.contains("placeholder")
            || self.jwt_secret.contains("change_me")
            || self.jwt_secret.contains("development")
            || self.jwt_secret == "development_secret_change_in_production"
        {
            return Err(
                "JWT secret contains placeholder or default values not allowed in production"
                    .to_string(),
            );
        }

        if self.database_url.starts_with("sqlite://") && !self.database_url.contains("?mode=ro") {
            return Err("SQLite databases should be read-only in production".to_string());
        }

        // Validate OAuth providers configuration in production
        // Check for empty or placeholder values
        if self.apple_client_id.is_empty()
            || self.apple_client_id.contains("placeholder")
            || self.apple_client_id.contains("test")
        {
            return Err("Apple Client ID must be properly configured for production".to_string());
        }

        if self.apple_team_id.is_empty()
            || self.apple_team_id.contains("placeholder")
            || self.apple_team_id.contains("test")
        {
            return Err("Apple Team ID must be properly configured for production".to_string());
        }

        if self.apple_key_id.is_empty()
            || self.apple_key_id.contains("placeholder")
            || self.apple_key_id.contains("test")
        {
            return Err("Apple Key ID must be properly configured for production".to_string());
        }

        if self.apple_private_key_path.is_empty()
            || self.apple_private_key_path.contains("placeholder")
            || self.apple_private_key_path.contains("/tmp/")
        {
            return Err(
                "Apple private key path must be properly configured for production".to_string(),
            );
        }

        // Verify the Apple private key file exists
        if !self.apple_private_key_path.is_empty()
            && !std::path::Path::new(&self.apple_private_key_path).exists()
        {
            return Err(format!(
                "Apple private key file not found: {}",
                self.apple_private_key_path
            ));
        }

        if !self.apple_redirect_uri.starts_with("https://") {
            return Err("Apple redirect URI must use HTTPS in production".to_string());
        }

        if self.github_client_id.is_empty()
            || self.github_client_id.contains("placeholder")
            || self.github_client_id.contains("test")
        {
            return Err("GitHub Client ID must be properly configured for production".to_string());
        }

        if self.github_client_secret.is_empty()
            || self.github_client_secret.contains("placeholder")
            || self.github_client_secret.contains("test")
            || self.github_client_secret.contains("secret")
        {
            return Err(
                "GitHub Client Secret must be properly configured for production".to_string(),
            );
        }

        // GitHub secrets should be at least 40 characters
        if self.github_client_secret.len() < 40 {
            return Err("GitHub Client Secret appears to be invalid (too short)".to_string());
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
