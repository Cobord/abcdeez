use std::fmt;
use std::error::Error as StdError;

/// Result type for alphabet-terminal operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the alphabet-terminal system
#[derive(Debug)]
pub enum Error {
    /// Invalid topology configuration
    InvalidTopology(String),
    
    /// Task generation failed
    TaskGenerationError(String),
    
    /// Numerical computation error
    NumericalError(String),
    
    /// Invalid parameters
    InvalidParameters(String),
    
    /// Data not found
    NotFound(String),
    
    /// Insufficient data for operation
    InsufficientData { 
        required: usize, 
        actual: usize 
    },
    
    /// Statistical test failed assumptions
    StatisticalAssumptionViolation(String),
    
    /// Convergence failure
    ConvergenceFailure {
        iterations: usize,
        tolerance: f64,
        final_error: f64,
    },
    
    /// IO error
    IoError(std::io::Error),
    
    /// Serialization error  
    SerializationError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidTopology(msg) => write!(f, "Invalid topology: {}", msg),
            Error::TaskGenerationError(msg) => write!(f, "Task generation failed: {}", msg),
            Error::NumericalError(msg) => write!(f, "Numerical error: {}", msg),
            Error::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::InsufficientData { required, actual } => {
                write!(f, "Insufficient data: required {}, got {}", required, actual)
            }
            Error::StatisticalAssumptionViolation(msg) => {
                write!(f, "Statistical assumption violated: {}", msg)
            }
            Error::ConvergenceFailure { iterations, tolerance, final_error } => {
                write!(f, "Failed to converge after {} iterations (tolerance: {}, error: {})", 
                       iterations, tolerance, final_error)
            }
            Error::IoError(e) => write!(f, "IO error: {}", e),
            Error::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerializationError(e.to_string())
    }
}