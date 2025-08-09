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
    let sum_sq_diff = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>();

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

/// Error function approximation (needed for Ex-Gaussian)
pub fn error_function(x: f64) -> f64 {
    // Abramowitz and Stegun approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Complementary error function
pub fn erfc(x: f64) -> f64 {
    1.0 - error_function(x)
}

/// Inverse error function (approximation)
pub fn inv_error_function(x: f64) -> f64 {
    let x = clamp(x, -0.99999, 0.99999);

    // Rational approximation
    let a = (8.0 * (f64::consts::PI - 3.0)) / (3.0 * f64::consts::PI * (4.0 - f64::consts::PI));
    let ln_term = safe_log(1.0 - x * x);

    let sqrt_term = (2.0 / (f64::consts::PI * a) + ln_term / 2.0).powi(2) - ln_term / a;
    let result = if sqrt_term >= 0.0 {
        -2.0 / (f64::consts::PI * a) - ln_term / 2.0 + sqrt_term.sqrt()
    } else {
        0.0
    };

    if x >= 0.0 {
        result.sqrt()
    } else {
        -result.sqrt()
    }
}

/// Gamma function approximation (Stirling's approximation for large values)
pub fn gamma_function(x: f64) -> f64 {
    if x < 0.5 {
        // Use reflection formula: Γ(z)Γ(1-z) = π/sin(πz)
        f64::consts::PI / (f64::consts::PI * x).sin() / gamma_function(1.0 - x)
    } else if x < 1.5 {
        gamma_function(x + 1.0) / x
    } else if x < 12.0 {
        // Lanczos approximation coefficients
        let g = 7.0;
        let coeff = [
            0.99999999999980993,
            676.5203681218851,
            -1259.1392167224028,
            771.32342877765313,
            -176.61502916214059,
            12.507343278686905,
            -0.13857109526572012,
            9.9843695780195716e-6,
            1.5056327351493116e-7,
        ];

        let z = x - 1.0;
        let mut x = coeff[0];
        for i in 1..coeff.len() {
            x += coeff[i] / (z + i as f64);
        }

        let t = z + g + 0.5;
        (2.0 * f64::consts::PI).sqrt() * t.powf(z + 0.5) * (-t).exp() * x
    } else {
        // Stirling's approximation for large x
        (2.0 * f64::consts::PI / x).sqrt() * (x / f64::consts::E).powf(x)
    }
}

/// Natural logarithm of gamma function
pub fn ln_gamma(x: f64) -> f64 {
    if x <= 0.0 {
        f64::NAN
    } else {
        gamma_function(x).ln()
    }
}

/// Beta function: B(x,y) = Γ(x)Γ(y)/Γ(x+y)
pub fn beta_function(x: f64, y: f64) -> f64 {
    if x <= 0.0 || y <= 0.0 {
        f64::NAN
    } else {
        gamma_function(x) * gamma_function(y) / gamma_function(x + y)
    }
}

/// Regularized incomplete beta function (approximation)
pub fn regularized_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Use continued fraction approximation
    let bt = if x == 0.0 || x == 1.0 {
        0.0
    } else {
        (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp()
    };

    if x < (a + 1.0) / (a + b + 2.0) {
        bt * beta_continued_fraction(x, a, b) / a
    } else {
        1.0 - bt * beta_continued_fraction(1.0 - x, b, a) / b
    }
}

fn beta_continued_fraction(x: f64, a: f64, b: f64) -> f64 {
    const MAX_ITER: usize = 100;
    const EPS: f64 = 3.0e-7;

    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;

    if d.abs() < f64::MIN_POSITIVE {
        d = f64::MIN_POSITIVE;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=MAX_ITER {
        let m_f = m as f64;
        let m2 = 2.0 * m_f;
        let aa = m_f * (b - m_f) * x / ((qam + m2) * (a + m2));

        d = 1.0 + aa * d;
        if d.abs() < f64::MIN_POSITIVE {
            d = f64::MIN_POSITIVE;
        }
        c = 1.0 + aa / c;
        if c.abs() < f64::MIN_POSITIVE {
            c = f64::MIN_POSITIVE;
        }
        d = 1.0 / d;
        h *= d * c;

        let aa = -(a + m_f) * (qab + m_f) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < f64::MIN_POSITIVE {
            d = f64::MIN_POSITIVE;
        }
        c = 1.0 + aa / c;
        if c.abs() < f64::MIN_POSITIVE {
            c = f64::MIN_POSITIVE;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;

        if (del - 1.0).abs() < EPS {
            break;
        }
    }

    h
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
