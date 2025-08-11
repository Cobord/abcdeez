# Ex-Gaussian Distribution

The Ex-Gaussian distribution is fundamental to modeling response times in cognitive tasks. It captures both the normal variability in processing and the occasional slow responses due to lapses in attention or strategy shifts.

## Mathematical Foundation

The Ex-Gaussian is a convolution of a Gaussian (normal) distribution and an exponential distribution:

### Probability Density Function

```math
f(x; μ, σ, τ) = \frac{1}{τ} \exp\left(\frac{μ}{τ} + \frac{σ²}{2τ²} - \frac{x}{τ}\right) \cdot \Phi\left(\frac{x - μ - σ²/τ}{σ}\right)
```

Where:
- **μ** (mu): Mean of the Gaussian component
- **σ** (sigma): Standard deviation of the Gaussian component  
- **τ** (tau): Rate parameter of the exponential component
- **Φ**: Cumulative distribution function of the standard normal

### Implementation Details

Our implementation includes numerical stability improvements:

```rust
pub fn pdf(&self, x: f64) -> f64 {
    if x < self.mu - 5.0 * self.sigma {
        return 0.0; // Negligible probability
    }
    
    // Stability checks for exponential term
    let exp_arg = self.mu / self.tau 
                  + self.sigma.powi(2) / (2.0 * self.tau.powi(2)) 
                  - x / self.tau;
    
    if exp_arg > 700.0 {
        return 0.0; // Prevent overflow
    } else if exp_arg < -700.0 {
        return 0.0; // Underflow to zero
    }
    
    // Compute with validated inputs
    let exp_term = exp_arg.exp();
    let z = (x - self.mu - self.sigma.powi(2) / self.tau) / self.sigma;
    let normal_cdf = normal_cdf(z);
    
    (1.0 / self.tau) * exp_term * normal_cdf
}
```

## Psychological Interpretation

### Components and Cognitive Processes

1. **Gaussian Component (μ, σ)**
   - Represents the core cognitive processing time
   - Captures normal variability in motor and decision processes
   - Typically 400-800ms for simple tasks

2. **Exponential Component (τ)**
   - Represents attention lapses and strategy shifts
   - Creates the characteristic right-skewed tail
   - Larger τ indicates more variable attention or strategy exploration

### Typical Parameter Values

For different cognitive tasks:

| Task Type | μ (ms) | σ (ms) | τ (ms) |
|-----------|--------|--------|--------|
| Simple RT | 250-350 | 20-50 | 50-150 |
| Choice RT | 400-600 | 50-100 | 100-300 |
| Memory Retrieval | 600-1000 | 100-200 | 200-500 |
| Problem Solving | 1000-3000 | 200-500 | 500-2000 |

## Fitting Ex-Gaussian to Data

### Maximum Likelihood Estimation

```rust
pub fn fit_mle(data: &[f64]) -> ExGaussianDistribution {
    // Initial parameter estimates
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / data.len() as f64;
    let skewness = calculate_skewness(data);
    
    // Method of moments for initial values
    let tau_init = (variance.powf(1.5) * skewness / 2.0).abs().powf(1.0/3.0);
    let sigma_init = (variance - tau_init.powi(2)).max(1.0).sqrt();
    let mu_init = mean - tau_init;
    
    // Optimize using gradient descent or other methods
    optimize_parameters(data, mu_init, sigma_init, tau_init)
}
```

### Goodness of Fit Testing

```rust
pub fn kolmogorov_smirnov_test(&self, data: &[f64]) -> GoodnessOfFitResult {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let n = sorted.len() as f64;
    let mut max_diff = 0.0;
    
    for (i, &x) in sorted.iter().enumerate() {
        let empirical = (i + 1) as f64 / n;
        let theoretical = self.cdf(x);
        let diff = (empirical - theoretical).abs();
        max_diff = max_diff.max(diff);
    }
    
    // Critical value for 95% confidence
    let critical = 1.36 / n.sqrt();
    
    GoodnessOfFitResult {
        statistic: max_diff,
        critical_value: critical,
        p_value: calculate_ks_p_value(max_diff, n),
        reject_null: max_diff > critical,
    }
}
```

## Strategy Classification Using Ex-Gaussian

Response time distributions reveal cognitive strategies:

### Serial Scanning
- Large τ parameter (>300ms)
- High variance in response times
- τ increases with set size

```rust
pub fn classify_strategy(params: &ExGaussianParams) -> Strategy {
    if params.tau > 300.0 && params.tau / params.mu > 0.5 {
        Strategy::SerialScanning
    } else if params.tau < 150.0 && params.sigma < 100.0 {
        Strategy::DirectAccess
    } else {
        Strategy::Hybrid
    }
}
```

### Direct Access
- Small τ parameter (<150ms)
- Low variance
- Parameters stable across set sizes

### Hybrid Strategy
- Intermediate τ values
- Bimodal response time distributions
- Transition period between strategies

## Practical Applications

### 1. Outlier Detection

```rust
pub fn detect_outliers(&self, data: &[f64], threshold: f64) -> Vec<usize> {
    data.iter()
        .enumerate()
        .filter(|(_, &x)| {
            let z_score = (x - self.mean()) / self.std_dev();
            z_score.abs() > threshold || x > self.mu + 3.0 * self.tau
        })
        .map(|(i, _)| i)
        .collect()
}
```

### 2. Performance Monitoring

```rust
pub fn track_performance_changes(
    baseline: &ExGaussianDistribution,
    current: &ExGaussianDistribution,
) -> PerformanceChange {
    let mu_change = (current.mu - baseline.mu) / baseline.mu;
    let tau_change = (current.tau - baseline.tau) / baseline.tau;
    
    if mu_change < -0.1 && tau_change < -0.2 {
        PerformanceChange::Improvement
    } else if mu_change > 0.1 || tau_change > 0.3 {
        PerformanceChange::Decline
    } else {
        PerformanceChange::Stable
    }
}
```

### 3. Individual Difference Analysis

```rust
pub fn analyze_individual_differences(
    participants: &[ParticipantData],
) -> IndividualDifferences {
    let mut params = Vec::new();
    
    for participant in participants {
        let fitted = ExGaussianDistribution::fit_mle(&participant.rts);
        params.push(fitted);
    }
    
    // Cluster analysis on parameters
    let clusters = cluster_by_parameters(&params);
    
    IndividualDifferences {
        fast_accurate: clusters[0].clone(),
        slow_careful: clusters[1].clone(),
        variable: clusters[2].clone(),
    }
}
```

## Numerical Considerations

### Stability Issues

1. **Extreme Parameter Values**
   - Very small σ can cause numerical instability
   - Very large τ relative to μ requires careful handling

2. **CDF Computation**
   - Use high-precision approximations for normal CDF
   - Implement series expansions for edge cases

### Optimization Tips

1. **Parameter Constraints**
   ```rust
   const MIN_SIGMA: f64 = 0.001;
   const MAX_TAU_RATIO: f64 = 10.0; // tau < 10 * mu
   ```

2. **Adaptive Sampling**
   - Use importance sampling for rare events
   - Adjust Monte Carlo samples based on parameter values

## Validation

Our implementation is validated against:
1. Analytical solutions for special cases
2. Published parameter values from cognitive psychology
3. Simulation studies with known ground truth
4. Cross-validation with real behavioral data

## References

- Matzke, D., & Wagenmakers, E. J. (2009). Psychological interpretation of the ex-Gaussian and shifted Wald parameters: A diffusion model analysis.
- Lacouture, Y., & Cousineau, D. (2008). How to use MATLAB to fit the ex-Gaussian and other probability functions to a distribution of response times.
- Palmer, E. M., Horowitz, T. S., Torralba, A., & Wolfe, J. M. (2011). What are the shapes of response time distributions in visual search?

## Next Steps

- Learn about [Response Time Analysis](./response_times.md)
- Explore [Strategy Classification](./strategies.md)
- Understand [Power Analysis](./power.md)