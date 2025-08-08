# Numerical Stability

Numerical stability is critical in cognitive modeling systems. Small numerical errors can compound through Bayesian updates, leading to incorrect inferences about learner knowledge. This chapter covers the numerical challenges and solutions implemented throughout the system.

## Floating Point Challenges

### The Precision Problem

```rust
// Problematic: Accumulating small values
let mut sum = 0.0;
for i in 0..1000000 {
    sum += 1e-10;  // Small values get lost
}
// sum might be much less than 0.0001 due to precision loss

// Better: Kahan summation
fn kahan_sum(values: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut c = 0.0;  // Compensation for lost digits
    
    for &value in values {
        let y = value - c;
        let t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }
    
    sum
}
```

### Overflow and Underflow

Common in exponential calculations:

```rust
// Problematic: Direct exponential
let prob = exp(700.0);  // Overflow!
let tiny = exp(-700.0); // Underflow to 0

// Better: Work in log space with machine epsilon-based thresholds
let log_prob = 700.0;
let log_tiny = -700.0;

// Derive thresholds from machine limits
const MAX_EXP: f64 = 709.78; // ln(f64::MAX)
const MIN_EXP: f64 = -708.39; // ln(f64::MIN_POSITIVE)

// When you need actual probability:
let safe_prob = if log_prob > MIN_EXP.ln() {
    1.0  // Would overflow, cap at 1
} else if log_prob < MAX_EXP.ln() {
    0.0  // Would underflow, set to 0
} else {
    log_prob.exp()
};
```

## Log-Space Computations

### Log-Sum-Exp Trick

Essential for combining probabilities:

```rust
// Problematic: Naive sum of exponentials
let sum = values.iter().map(|x| x.exp()).sum();
let result = sum.ln();  // Can overflow/underflow

// Better: Log-sum-exp trick
fn log_sum_exp(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NEG_INFINITY;
    }
    
    let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    
    if !max_val.is_finite() {
        return max_val;
    }
    
    let sum_exp = values.iter()
        .map(|&x| (x - max_val).exp())
        .sum::<f64>();
    
    max_val + sum_exp.ln()
}
```

### Log-Space Gaussian Operations

Working with Gaussian PDFs in log space:

```rust
// Log PDF of Gaussian
fn log_gaussian_pdf(x: f64, mean: f64, variance: f64) -> f64 {
    let two_pi = 2.0 * std::f64::consts::PI;
    
    -0.5 * (two_pi * variance).ln() 
    - 0.5 * (x - mean).powi(2) / variance
}

// Product of Gaussians in log space
fn log_gaussian_product(
    mean1: f64, var1: f64,
    mean2: f64, var2: f64
) -> (f64, f64, f64) {
    // New variance
    let new_var = 1.0 / (1.0 / var1 + 1.0 / var2);
    
    // New mean
    let new_mean = new_var * (mean1 / var1 + mean2 / var2);
    
    // Log normalization constant
    let log_norm = -0.5 * (
        (2.0 * PI * (var1 + var2)).ln() +
        (mean1 - mean2).powi(2) / (var1 + var2)
    );
    
    (new_mean, new_var, log_norm)
}
```

## Ex-Gaussian Numerical Stability

The Ex-Gaussian implementation faces several challenges:

### Complementary Error Function

```rust
impl ExGaussianModel {
    pub fn pdf(&self, x: f64) -> f64 {
        // Guard against invalid inputs
        if x < 0.0 || self.params.tau <= 0.0 || self.params.sigma <= 0.0 {
            return 0.0;
        }
        
        let lambda = 1.0 / self.params.tau;
        
        // Compute exponential argument with bounds
        let exp_arg = (lambda / 2.0) * 
            (2.0 * self.params.mu + lambda * self.params.sigma.powi(2) - 2.0 * x);
        
        // Prevent overflow/underflow using machine-derived limits
        const LN_MIN: f64 = -708.39; // ln(f64::MIN_POSITIVE)
        const LN_MAX: f64 = 709.78;  // ln(f64::MAX)
        
        if exp_arg < LN_MIN {
            return 0.0;  // Would underflow
        }
        if exp_arg > LN_MAX {
            // Would overflow, return max reasonable PDF value
            return 1.0 / (self.params.sigma * (2.0 * PI).sqrt());
        }
        
        // Compute erfc with special handling at extremes
        let erfc_arg = (self.params.mu + lambda * self.params.sigma.powi(2) - x) / 
                       (self.params.sigma * SQRT_2);
        
        let erfc_val = if erfc_arg > 5.0 {
            // erfc(5) ≈ 1.5e-12, effectively 0
            0.0
        } else if erfc_arg < -5.0 {
            // erfc(-5) ≈ 2, the maximum value
            2.0
        } else {
            // Safe to compute
            statrs::function::erf::erfc(erfc_arg)
        };
        
        // Final computation with sanity check
        let result = (lambda / 2.0) * exp_arg.exp() * erfc_val;
        
        if result.is_finite() {
            result
        } else {
            0.0
        }
    }
}
```

## KL Divergence Stability

KL divergence calculations are prone to numerical issues:

```rust
fn stable_kl_divergence(p: &[f64], q: &[f64]) -> f64 {
    assert_eq!(p.len(), q.len());
    
    let mut kl = 0.0;
    
    for (pi, qi) in p.iter().zip(q.iter()) {
        // Skip if pi is effectively 0 (contributes 0 to KL)
        // Use machine epsilon-based threshold
        let epsilon = f64::EPSILON;
        let threshold = epsilon.sqrt(); // sqrt(epsilon) is commonly used
        
        if *pi < threshold {
            continue;
        }
        
        // Handle qi = 0 (infinite KL)
        if *qi < threshold {
            return f64::INFINITY;
        }
        
        // Safe computation
        kl += pi * (pi.ln() - qi.ln());
    }
    
    kl.max(0.0)  // KL is always non-negative
}

// For Gaussians
fn gaussian_kl_divergence(
    mu1: f64, var1: f64,
    mu2: f64, var2: f64
) -> f64 {
    // Ensure variances are positive
    let var1 = var1.max(1e-10);
    let var2 = var2.max(1e-10);
    
    // Compute KL with bounds
    let ln_ratio = (var2 / var1).ln();
    let trace_term = var1 / var2;
    let quad_term = (mu1 - mu2).powi(2) / var2;
    
    0.5 * (ln_ratio + trace_term + quad_term - 1.0).max(0.0)
}
```

## Monte Carlo Numerical Issues

### Importance Sampling Weights

```rust
fn stable_importance_weights(log_weights: &[f64]) -> Vec<f64> {
    // Normalize in log space to avoid overflow
    let log_sum = log_sum_exp(log_weights);
    
    log_weights.iter()
        .map(|&log_w| {
            let normalized_log = log_w - log_sum;
            
            if normalized_log < -30.0 {
                0.0  // Too small to matter
            } else {
                normalized_log.exp()
            }
        })
        .collect()
}
```

### Effective Sample Size

Monitor Monte Carlo health:

```rust
fn effective_sample_size(weights: &[f64]) -> f64 {
    let sum_weights: f64 = weights.iter().sum();
    let sum_squared: f64 = weights.iter().map(|w| w * w).sum();
    
    if sum_squared > 0.0 {
        (sum_weights * sum_weights) / sum_squared
    } else {
        0.0
    }
}

// Resample if ESS too low
fn adaptive_resampling(samples: &mut Vec<Sample>, weights: &[f64]) {
    let ess = effective_sample_size(weights);
    let threshold = samples.len() as f64 / 2.0;
    
    if ess < threshold {
        resample_systematic(samples, weights);
    }
}
```

## Matrix Operations Stability

### Cholesky Decomposition

For sampling from multivariate Gaussians:

```rust
fn stable_cholesky(covariance: &Matrix) -> Result<Matrix, Error> {
    // Add small diagonal to ensure positive definite
    let epsilon = 1e-10;
    let mut cov = covariance.clone();
    
    for i in 0..cov.nrows() {
        cov[(i, i)] += epsilon;
    }
    
    // Attempt decomposition
    match cov.cholesky() {
        Ok(l) => Ok(l),
        Err(_) => {
            // Fall back to eigendecomposition
            let (eigvals, eigvecs) = cov.symmetric_eigen();
            
            // Threshold negative eigenvalues
            let eigvals = eigvals.map(|x| x.max(epsilon));
            
            // Reconstruct L
            let sqrt_eigvals = eigvals.map(|x| x.sqrt());
            Ok(eigvecs * Matrix::from_diagonal(&sqrt_eigvals))
        }
    }
}
```

## Gradient Computations

For optimization routines:

```rust
fn stable_gradient(f: impl Fn(f64) -> f64, x: f64, h: f64) -> f64 {
    // Use central differences for better accuracy
    let f_plus = f(x + h);
    let f_minus = f(x - h);
    
    // Check for numerical issues
    if !f_plus.is_finite() || !f_minus.is_finite() {
        // Fall back to one-sided difference
        let f_x = f(x);
        if f_plus.is_finite() {
            return (f_plus - f_x) / h;
        } else if f_minus.is_finite() {
            return (f_x - f_minus) / h;
        } else {
            return 0.0;  // Gradient undefined
        }
    }
    
    (f_plus - f_minus) / (2.0 * h)
}
```

## Testing Numerical Stability

### Property-Based Tests

```rust
#[cfg(test)]
mod numerical_tests {
    use quickcheck::{quickcheck, TestResult};
    
    quickcheck! {
        fn test_log_sum_exp_stability(values: Vec<f64>) -> TestResult {
            // Filter out NaN/Inf
            let clean: Vec<f64> = values.into_iter()
                .filter(|x| x.is_finite())
                .collect();
            
            if clean.is_empty() {
                return TestResult::discard();
            }
            
            let result = log_sum_exp(&clean);
            
            TestResult::from_bool(
                result.is_finite() || 
                clean.iter().any(|&x| x > 700.0)  // Overflow expected
            )
        }
    }
}
```

### Stress Tests

```rust
#[test]
fn test_extreme_values() {
    // Test with very small probabilities
    let tiny_probs = vec![1e-300; 1000];
    let sum = stable_sum(&tiny_probs);
    assert!((sum - 1e-297).abs() < 1e-298);
    
    // Test with very large values
    let log_probs = vec![700.0, 699.0, 698.0];
    let log_sum = log_sum_exp(&log_probs);
    assert!(log_sum > 700.0 && log_sum < 701.0);
    
    // Test with mixed scales
    let mixed = vec![1e-10, 1.0, 1e10];
    let result = process_mixed_scales(&mixed);
    assert!(result.is_finite());
}
```

## Best Practices Summary

1. **Work in log space** when dealing with probabilities
2. **Bound exponential arguments** to prevent overflow
3. **Use stable algorithms** (Kahan summation, log-sum-exp)
4. **Add small constants** to prevent division by zero
5. **Check for special cases** (NaN, Inf, very small/large values)
6. **Test with extreme inputs** to verify stability
7. **Monitor numerical health** (condition numbers, ESS)
8. **Provide fallbacks** for numerical failures

## Performance Impact

Numerical stability often comes with performance costs:

```rust
// Fast but unstable
fn fast_sum(values: &[f64]) -> f64 {
    values.iter().sum()
}

// Stable but slower
fn stable_sum(values: &[f64]) -> f64 {
    kahan_sum(values)  // ~2x slower
}

// Choose based on context
fn adaptive_sum(values: &[f64]) -> f64 {
    let range = values.iter().max() - values.iter().min();
    
    if range < 1e6 {
        fast_sum(values)  // Precision loss negligible
    } else {
        stable_sum(values)  // Need stability
    }
}
```

## Summary

Numerical stability is not optional in scientific computing:
- **Silent failures** can corrupt entire analyses
- **Defensive programming** prevents catastrophic errors
- **Testing** must include extreme cases
- **Performance tradeoffs** are usually worth it

The implementation prioritizes correctness over speed, ensuring reliable cognitive modeling even with challenging numerical conditions.