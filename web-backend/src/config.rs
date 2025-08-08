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
                .unwrap_or_else(|_| "development_secret_change_in_production".to_string()),
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
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
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