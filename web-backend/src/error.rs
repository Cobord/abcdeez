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

    // Core library errors
    CoreError(graph_learning_core::Error),
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
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::RedisError(e) => {
                tracing::error!("Redis error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Cache error".to_string())
            }
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
            ),

            // Core library errors
            AppError::CoreError(e) => {
                tracing::error!("Core library error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Core library error".to_string(),
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
                tracing::error!("Numerical error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Numerical computation error".to_string(),
                )
            }
            AppError::StatisticalError(msg) => {
                tracing::error!("Statistical error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Statistical computation error".to_string(),
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

impl From<graph_learning_core::Error> for AppError {
    fn from(err: graph_learning_core::Error) -> Self {
        AppError::CoreError(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::ValidationError(format!("JSON parsing error: {}", err))
    }
}

pub type AppResult<T> = Result<T, AppError>;
