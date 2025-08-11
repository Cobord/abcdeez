use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;
use tracing::{error, warn, debug};

/// Helper structure for consistent error logging with unique error IDs
#[derive(Debug)]
struct ErrorContext {
    id: uuid::Uuid,
    error_type: &'static str,
    message: String,
}

impl ErrorContext {
    fn new(error_type: &'static str, message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            error_type,
            message: message.into(),
        }
    }

    fn log_and_format(&self, user_message: impl Into<String>) -> (StatusCode, String) {
        error!(
            error_id = %self.id,
            error_type = self.error_type,
            "{}",
            self.message
        );
        debug!("Error details for {}: {}", self.id, self.message);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}. Error ID: {}", user_message.into(), self.id),
        )
    }
}

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound(String),
    Conflict(String),
    UnprocessableEntity(String),
    InternalServerError,
    DatabaseError(sqlx::Error),
    CacheError(String),
    ValidationError(String),
    AuthenticationError(String),
    RateLimitExceeded,
    InvalidUuid(uuid::Error),

    // Core library errors
    CoreError(abcdeez_core::Error),
    TaskGenerationError(String),
    NumericalError(String),
    StatisticalError(String),
    ConvergenceError {
        iterations: usize,
        tolerance: f64,
        final_error: f64,
    },
    InsufficientData {
        required: usize,
        actual: usize,
    },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Forbidden => write!(f, "Forbidden"),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::UnprocessableEntity(msg) => write!(f, "Unprocessable entity: {}", msg),
            AppError::InternalServerError => write!(f, "Internal server error"),
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::CacheError(e) => write!(f, "Cache error: {}", e),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            AppError::RateLimitExceeded => write!(f, "Rate limit exceeded"),

            // Core library errors
            AppError::CoreError(e) => write!(f, "Core library error: {}", e),
            AppError::TaskGenerationError(msg) => write!(f, "Task generation error: {}", msg),
            AppError::NumericalError(msg) => write!(f, "Numerical error: {}", msg),
            AppError::StatisticalError(msg) => write!(f, "Statistical error: {}", msg),
            AppError::ConvergenceError {
                iterations,
                tolerance,
                final_error,
            } => {
                write!(
                    f,
                    "Convergence error: failed after {} iterations (tolerance: {}, error: {})",
                    iterations, tolerance, final_error
                )
            }
            AppError::InsufficientData { required, actual } => {
                write!(
                    f,
                    "Insufficient data: required {}, got {}",
                    required, actual
                )
            }
            AppError::InvalidUuid(err) => {
                write!(f, "Invalid UUID: {}", err)
            }
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AppError::DatabaseError(e) => {
                let ctx = ErrorContext::new("database", format!("Database operation failed: {:?}", e));
                ctx.log_and_format("Database operation failed")
            }
            AppError::CacheError(e) => {
                let ctx = ErrorContext::new("cache", format!("Cache operation failed: {}", e));
                ctx.log_and_format("Cache operation failed")
            }
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
            ),

            // Core library errors
            AppError::CoreError(e) => {
                let ctx = ErrorContext::new("core_library", format!("Core library operation failed: {:?}", e));
                ctx.log_and_format("Internal operation failed")
            }
            AppError::TaskGenerationError(msg) => {
                warn!("Task generation error: {}", msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("Task generation failed: {}", msg),
                )
            }
            AppError::NumericalError(msg) => {
                let ctx = ErrorContext::new("numerical", format!("Numerical computation failed: {}", msg));
                ctx.log_and_format("Computation failed")
            }
            AppError::StatisticalError(msg) => {
                let ctx = ErrorContext::new("statistical", format!("Statistical computation failed: {}", msg));
                ctx.log_and_format("Analysis failed")
            }
            AppError::ConvergenceError {
                iterations,
                tolerance,
                final_error,
            } => {
                warn!(
                    "Convergence failure: {} iterations, tolerance: {}, error: {}",
                    iterations,
                    tolerance,
                    final_error
                );
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Algorithm failed to converge".to_string(),
                )
            }
            AppError::InsufficientData { required, actual } => (
                StatusCode::BAD_REQUEST,
                format!(
                    "Insufficient data: required {} samples, got {}",
                    required, actual
                ),
            ),
            AppError::InvalidUuid(err) => {
                (StatusCode::BAD_REQUEST, format!("Invalid UUID: {}", err))
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<crate::cache::CacheError> for AppError {
    fn from(err: crate::cache::CacheError) -> Self {
        AppError::CacheError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::AuthenticationError(err.to_string())
    }
}

impl From<abcdeez_core::Error> for AppError {
    fn from(err: abcdeez_core::Error) -> Self {
        AppError::CoreError(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::ValidationError(format!("JSON parsing error: {}", err))
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::InvalidUuid(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalServerError
    }
}

pub type AppResult<T> = Result<T, AppError>;
