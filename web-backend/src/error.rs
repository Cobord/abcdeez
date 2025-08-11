use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound(String),
    Conflict(String),
    ConflictError(String),
    UnprocessableEntity(String),
    InternalServerError,
    DatabaseError(sqlx::Error),
    RedisError(redis::RedisError),
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
            AppError::ConflictError(msg) => write!(f, "Conflict: {}", msg),
            AppError::UnprocessableEntity(msg) => write!(f, "Unprocessable entity: {}", msg),
            AppError::InternalServerError => write!(f, "Internal server error"),
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::RedisError(e) => write!(f, "Redis error: {}", e),
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
            AppError::ConflictError(msg) => (StatusCode::CONFLICT, msg),
            AppError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AppError::DatabaseError(e) => {
                // Log sanitized error information
                let error_id = uuid::Uuid::new_v4();
                tracing::error!(
                    error_id = %error_id,
                    error_type = "database",
                    "Database operation failed"
                );
                // Log full error details at debug level for troubleshooting
                tracing::debug!("Database error details for {}: {:?}", error_id, e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database operation failed. Error ID: {}", error_id),
                )
            }
            AppError::RedisError(e) => {
                // Log sanitized error information
                let error_id = uuid::Uuid::new_v4();
                tracing::error!(
                    error_id = %error_id,
                    error_type = "cache",
                    "Cache operation failed"
                );
                tracing::debug!("Redis error details for {}: {:?}", error_id, e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Cache operation failed. Error ID: {}", error_id),
                )
            }
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
            ),

            // Core library errors
            AppError::CoreError(e) => {
                let error_id = uuid::Uuid::new_v4();
                tracing::error!(
                    error_id = %error_id,
                    error_type = "core_library",
                    "Core library operation failed"
                );
                tracing::debug!("Core library error details for {}: {:?}", error_id, e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal operation failed. Error ID: {}", error_id),
                )
            }
            AppError::TaskGenerationError(msg) => {
                tracing::warn!("Task generation error: {}", msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("Task generation failed: {}", msg),
                )
            }
            AppError::NumericalError(msg) => {
                let error_id = uuid::Uuid::new_v4();
                tracing::error!(
                    error_id = %error_id,
                    error_type = "numerical",
                    "Numerical computation failed"
                );
                tracing::debug!("Numerical error details for {}: {}", error_id, msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Computation failed. Error ID: {}", error_id),
                )
            }
            AppError::StatisticalError(msg) => {
                let error_id = uuid::Uuid::new_v4();
                tracing::error!(
                    error_id = %error_id,
                    error_type = "statistical",
                    "Statistical computation failed"
                );
                tracing::debug!("Statistical error details for {}: {}", error_id, msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Analysis failed. Error ID: {}", error_id),
                )
            }
            AppError::ConvergenceError {
                iterations,
                tolerance,
                final_error,
            } => {
                tracing::warn!(
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

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::RedisError(err)
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
