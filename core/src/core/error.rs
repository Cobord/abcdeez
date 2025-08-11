use std::{error::Error as StdError, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidTopology(String),
    TaskGenerationError(String),
    NumericalError(String),
    InvalidParameters(String),
    NotFound(String),
    InsufficientData { required: usize, actual: usize },
    StatisticalAssumptionViolation(String),
    ConvergenceFailure {
        iterations: usize,
        tolerance: f64,
        final_error: f64,
    },
    Io(std::io::Error),
    Serialization(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidTopology(msg) => write!(f, "Invalid topology: {msg}"),
            Error::TaskGenerationError(msg) => write!(f, "Task generation failed: {msg}"),
            Error::NumericalError(msg) => write!(f, "Numerical error: {msg}"),
            Error::InvalidParameters(msg) => write!(f, "Invalid parameters: {msg}"),
            Error::NotFound(msg) => write!(f, "Not found: {msg}"),
            Error::InsufficientData { required, actual } => {
                write!(f, "Insufficient data: required {required}, got {actual}")
            }
            Error::StatisticalAssumptionViolation(msg) => {
                write!(f, "Statistical assumption violated: {msg}")
            }
            Error::ConvergenceFailure { iterations, tolerance, final_error } => {
                write!(f, "Failed to converge after {iterations} iterations (tolerance: {tolerance}, error: {final_error})")
            }
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::Serialization(e) => write!(f, "Serialization error: {e}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e)
    }
}

