/// Mathematical utilities with numerical stability guarantees
use std::f64;

/// Safe division that handles edge cases
pub fn safe_divide(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator.abs() < f64::EPSILON {
        None
    } else {
        Some(numerator / denominator)
    }
}

/// Safe division with fallback value
pub fn safe_divide_or(numerator: f64, denominator: f64, fallback: f64) -> f64 {
    safe_divide(numerator, denominator).unwrap_or(fallback)
}

/// Safe floating point equality comparison
pub fn float_eq(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

/// Safe floating point comparison with default epsilon
pub fn float_eq_default(a: f64, b: f64) -> bool {
    float_eq(a, b, f64::EPSILON)
}

/// Safe logarithm that clamps input to avoid NaN/infinity
pub fn safe_log(x: f64) -> f64 {
    x.max(f64::EPSILON).ln()
}

/// Safe logarithm base 2
pub fn safe_log2(x: f64) -> f64 {
    x.max(f64::EPSILON).log2()
}

/// Clamp a value to a safe range
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// Safe accuracy calculation (correct_count / total_count)
pub fn safe_accuracy(correct_count: usize, total_count: usize) -> f64 {
    if total_count == 0 {
        0.0
    } else {
        correct_count as f64 / total_count as f64
    }
}

/// Safe mean calculation
pub fn safe_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

/// Safe variance calculation (sample variance with Bessel's correction)
pub fn safe_variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    
    let mean = safe_mean(values);
    let sum_sq_diff = values.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>();
    
    sum_sq_diff / (values.len() - 1) as f64
}

/// Safe standard deviation
pub fn safe_std_dev(values: &[f64]) -> f64 {
    safe_variance(values).sqrt()
}

/// Safe confidence interval calculation (assuming normal distribution)
pub fn confidence_interval(values: &[f64], confidence: f64) -> (f64, f64) {
    if values.len() < 2 {
        let mean = safe_mean(values);
        return (mean, mean);
    }
    
    let mean = safe_mean(values);
    let std_dev = safe_std_dev(values);
    let n = values.len() as f64;
    let std_error = std_dev / n.sqrt();
    
    // Use t-distribution critical values (simplified approximation)
    let t_critical = match confidence {
        0.90 => 1.645,
        0.95 => 1.96,
        0.99 => 2.576,
        _ => 1.96, // Default to 95%
    };
    
    let margin = t_critical * std_error;
    (mean - margin, mean + margin)
}

/// Convert probability to odds (safe version)
pub fn prob_to_odds(p: f64) -> f64 {
    let safe_p = clamp(p, f64::EPSILON, 1.0 - f64::EPSILON);
    safe_p / (1.0 - safe_p)
}

/// Convert odds to probability (safe version)  
pub fn odds_to_prob(odds: f64) -> f64 {
    let safe_odds = odds.max(f64::EPSILON);
    safe_odds / (1.0 + safe_odds)
}

/// Safe sigmoid function that avoids overflow
pub fn safe_sigmoid(x: f64) -> f64 {
    if x > 500.0 {
        1.0
    } else if x < -500.0 {
        0.0
    } else {
        1.0 / (1.0 + (-x).exp())
    }
}

/// Safe logit function (inverse sigmoid)
pub fn safe_logit(p: f64) -> f64 {
    let safe_p = clamp(p, f64::EPSILON, 1.0 - f64::EPSILON);
    safe_log(safe_p / (1.0 - safe_p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64;

    #[test]
    fn test_safe_divide() {
        assert_eq!(safe_divide(10.0, 2.0), Some(5.0));
        assert_eq!(safe_divide(10.0, 0.0), None);
        assert_eq!(safe_divide_or(10.0, 0.0, 999.0), 999.0);
    }

    #[test]
    fn test_float_equality() {
        assert!(float_eq_default(0.1 + 0.2, 0.3));
        assert!(!float_eq_default(0.1, 0.2));
    }

    #[test]
    fn test_safe_accuracy() {
        assert_eq!(safe_accuracy(7, 10), 0.7);
        assert_eq!(safe_accuracy(0, 0), 0.0);
    }

    #[test]
    fn test_safe_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(safe_mean(&values), 3.0);
        assert!(safe_std_dev(&values) > 0.0);
        
        let empty: Vec<f64> = vec![];
        assert_eq!(safe_mean(&empty), 0.0);
        assert_eq!(safe_std_dev(&empty), 0.0);
    }

    #[test]
    fn test_safe_sigmoid() {
        assert!(safe_sigmoid(0.0) == 0.5);
        assert!(safe_sigmoid(1000.0) == 1.0);
        assert!(safe_sigmoid(-1000.0) == 0.0);
    }
}