/// Mathematical operation validation utilities
/// Provides safe wrappers for common mathematical operations that can fail

use crate::core::error::Error;

/// Safe logarithm that checks for non-positive values
pub fn safe_ln(x: f64) -> Result<f64, Error> {
    if x <= 0.0 {
        Err(Error::NumericalError(format!(
            "Cannot take logarithm of non-positive value: {}",
            x
        )))
    } else {
        Ok(x.ln())
    }
}

/// Safe square root that checks for negative values
pub fn safe_sqrt(x: f64) -> Result<f64, Error> {
    if x < 0.0 {
        Err(Error::NumericalError(format!(
            "Cannot take square root of negative value: {}",
            x
        )))
    } else {
        Ok(x.sqrt())
    }
}

/// Safe division that checks for division by zero
pub fn safe_div(numerator: f64, denominator: f64) -> Result<f64, Error> {
    if denominator.abs() < f64::EPSILON {
        Err(Error::NumericalError(
            "Division by zero or near-zero value".to_string()
        ))
    } else {
        Ok(numerator / denominator)
    }
}

/// Safe exponential that checks for overflow
pub fn safe_exp(x: f64) -> Result<f64, Error> {
    if x > 700.0 {
        Err(Error::NumericalError(format!(
            "Exponential would overflow: exp({})",
            x
        )))
    } else if x < -700.0 {
        Ok(0.0) // exp(-700) is effectively 0
    } else {
        Ok(x.exp())
    }
}

/// Validate that a correlation coefficient is in [-1, 1]
pub fn validate_correlation(r: f64) -> Result<f64, Error> {
    if r < -1.0 || r > 1.0 {
        Err(Error::StatisticalAssumptionViolation(format!(
            "Invalid correlation coefficient: {} (must be in [-1, 1])",
            r
        )))
    } else {
        Ok(r)
    }
}

/// Validate that a probability is in [0, 1]
pub fn validate_probability(p: f64) -> Result<f64, Error> {
    if p < 0.0 || p > 1.0 {
        Err(Error::StatisticalAssumptionViolation(format!(
            "Invalid probability: {} (must be in [0, 1])",
            p
        )))
    } else {
        Ok(p)
    }
}

/// Validate that a variance is non-negative
pub fn validate_variance(var: f64) -> Result<f64, Error> {
    if var < 0.0 {
        Err(Error::StatisticalAssumptionViolation(format!(
            "Invalid variance: {} (must be non-negative)",
            var
        )))
    } else {
        Ok(var)
    }
}

/// Validate sample size is positive
pub fn validate_sample_size(n: usize) -> Result<usize, Error> {
    if n == 0 {
        Err(Error::StatisticalAssumptionViolation(
            "Sample size must be positive".to_string()
        ))
    } else {
        Ok(n)
    }
}

/// Check if a float value is finite (not NaN or infinite)
pub fn ensure_finite(value: f64, context: &str) -> Result<f64, Error> {
    if !value.is_finite() {
        Err(Error::NumericalError(format!(
            "Non-finite value in {}: {}",
            context, value
        )))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_ln() {
        assert!(safe_ln(1.0).is_ok());
        assert!(safe_ln(0.0).is_err());
        assert!(safe_ln(-1.0).is_err());
    }

    #[test]
    fn test_safe_sqrt() {
        assert!(safe_sqrt(4.0).is_ok());
        assert_eq!(safe_sqrt(4.0).unwrap(), 2.0);
        assert!(safe_sqrt(-1.0).is_err());
    }

    #[test]
    fn test_safe_div() {
        assert!(safe_div(10.0, 2.0).is_ok());
        assert_eq!(safe_div(10.0, 2.0).unwrap(), 5.0);
        assert!(safe_div(10.0, 0.0).is_err());
    }

    #[test]
    fn test_safe_exp() {
        assert!(safe_exp(0.0).is_ok());
        assert_eq!(safe_exp(0.0).unwrap(), 1.0);
        assert!(safe_exp(800.0).is_err());
        assert_eq!(safe_exp(-800.0).unwrap(), 0.0);
    }

    #[test]
    fn test_validate_correlation() {
        assert!(validate_correlation(0.5).is_ok());
        assert!(validate_correlation(-1.0).is_ok());
        assert!(validate_correlation(1.0).is_ok());
        assert!(validate_correlation(1.5).is_err());
        assert!(validate_correlation(-1.5).is_err());
    }

    #[test]
    fn test_validate_probability() {
        assert!(validate_probability(0.5).is_ok());
        assert!(validate_probability(0.0).is_ok());
        assert!(validate_probability(1.0).is_ok());
        assert!(validate_probability(-0.1).is_err());
        assert!(validate_probability(1.1).is_err());
    }
}