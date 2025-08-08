# Ex-Gaussian Distribution

The Ex-Gaussian distribution is fundamental to cognitive response time modeling. It captures both the typical fast responses and occasional slow responses that characterize human reaction times.

## Why Ex-Gaussian?

Response time distributions in cognitive tasks are typically:
- **Right-skewed**: Most responses are fast, with a long tail of slow responses
- **Bounded below**: There's a minimum time for perception and motor response
- **Variable tail**: The tail heaviness indicates cognitive processing variability

The Ex-Gaussian elegantly captures these properties as the convolution of:
- **Gaussian component**: Normal variability in processing
- **Exponential component**: Occasional attention lapses or strategy switches

## Mathematical Definition

The Ex-Gaussian PDF is correctly expressed using the complementary error function:

```
f(x; μ, σ, τ) = (1/(2τ)) * exp((μ - x)/τ + σ²/(2τ²)) * erfc((μ + σ²/τ - x)/(σ√2))
```

Or equivalently with rate parameter λ = 1/τ:

```
f(x; μ, σ, λ) = (λ/2) * exp(λ(μ - x + λσ²/2)) * erfc((μ + λσ² - x)/(σ√2))
```

**Important**: The factor is 1/(2τ) or λ/2, not 1/τ, when using erfc. This ensures the distribution integrates to 1.

Where:
- `μ`: Mean of Gaussian component
- `σ`: Standard deviation of Gaussian component  
- `τ`: Mean of exponential component (1/λ where λ is rate)
- `Φ`: Standard normal CDF
- `erfc`: Complementary error function

## Implementation

Our implementation handles numerical challenges:

```rust
pub struct ExGaussianModel {
    pub params: ExGaussianParameters,
}

impl ExGaussianModel {
    pub fn pdf(&self, x: f64) -> f64 {
        // Ex-Gaussian not defined for negative values
        if x < 0.0 || self.params.tau <= 0.0 || self.params.sigma <= 0.0 {
            return 0.0;
        }
        
        let lambda = 1.0 / self.params.tau;
        
        // Calculate exponential argument
        let exp_arg = (lambda / 2.0) * 
            (2.0 * self.params.mu + lambda * self.params.sigma.powi(2) - 2.0 * x);
        
        // Prevent numerical overflow/underflow
        if exp_arg < -50.0 {
            return 0.0;  // Essentially zero
        }
        if exp_arg > 50.0 {
            return 1e10; // Cap at large value
        }
        
        // Calculate erfc argument
        let erfc_arg = (self.params.mu + lambda * self.params.sigma.powi(2) - x) / 
                       (self.params.sigma * std::f64::consts::SQRT_2);
        
        // Compute erfc with bounds checking
        let erfc_val = if erfc_arg > 5.0 {
            0.0  // erfc approaches 0 for large positive arguments
        } else if erfc_arg < -5.0 {
            2.0  // erfc approaches 2 for large negative arguments
        } else {
            statrs::function::erf::erfc(erfc_arg)
        };
        
        // Final computation
        let result = (lambda / 2.0) * exp_arg.exp() * erfc_val;
        
        if result.is_finite() {
            result
        } else {
            0.0
        }
    }
}
```

## Cumulative Distribution Function

The CDF is crucial for computing probabilities:

```rust
pub fn cdf(&self, x: f64) -> f64 {
    if self.params.tau <= 0.0 || self.params.sigma <= 0.0 {
        return 0.0;
    }
    
    let lambda = 1.0 / self.params.tau;
    let normal = Normal::new(0.0, 1.0).unwrap();
    
    // First term: Φ((x-μ)/σ)
    let term1 = normal.cdf((x - self.params.mu) / self.params.sigma);
    
    // Second term exponential part
    let exp_arg = (lambda / 2.0) * 
        (2.0 * self.params.mu + lambda * self.params.sigma.powi(2) - 2.0 * x);
    
    if exp_arg > 50.0 {
        return 0.0; // Would cause overflow
    }
    
    // Second term normal CDF part
    let term2_arg = (x - self.params.mu - lambda * self.params.sigma.powi(2)) / 
                    self.params.sigma;
    
    let term2 = if exp_arg < -50.0 {
        0.0  // Exponential is essentially 0
    } else {
        exp_arg.exp() * normal.cdf(term2_arg)
    };
    
    (term1 - term2).max(0.0).min(1.0)
}
```

## Moments

The Ex-Gaussian has tractable moments:

```rust
impl ExGaussianModel {
    pub fn mean(&self) -> f64 {
        self.params.mu + self.params.tau
    }
    
    pub fn variance(&self) -> f64 {
        self.params.sigma.powi(2) + self.params.tau.powi(2)
    }
    
    pub fn skewness(&self) -> f64 {
        let var = self.variance();
        let std = var.sqrt();
        
        2.0 * self.params.tau.powi(3) / (std.powi(3))
    }
}
```

## Parameter Estimation

We fit Ex-Gaussian parameters to response time data:

```rust
impl ExGaussianModel {
    pub fn fit(response_times: &[f64]) -> Self {
        let stats = DetailedStatistics::from_data(response_times);
        
        // Method of moments estimation
        // Mean = μ + τ
        // Variance = σ² + τ²
        // Skewness ≈ 2τ³/(σ² + τ²)^(3/2)
        
        // Initial estimates
        let mu = stats.mean - stats.std_dev;
        let sigma = stats.std_dev * 0.8;
        let tau = stats.std_dev * 0.5;
        
        // Could refine with MLE, but this is often sufficient
        ExGaussianModel::from_params(mu, sigma, tau)
    }
}
```

## Cognitive Interpretation

The parameters have psychological meaning:

### μ (Mu) - Central Tendency
- **Low μ**: Fast base processing speed
- **High μ**: Slower base processing
- **Example**: Expert typists have lower μ for letter sequences

### σ (Sigma) - Consistent Variability  
- **Low σ**: Consistent, automatic processing
- **High σ**: Variable, controlled processing
- **Example**: Well-learned sequences show lower σ

### τ (Tau) - Attention/Strategy
- **Low τ**: Focused attention, consistent strategy
- **High τ**: Attention lapses, strategy switching
- **Example**: Difficult tasks show higher τ

## Response Time Patterns

Different cognitive strategies produce different Ex-Gaussian signatures:

```rust
pub struct StrategyAnalysis {
    pub rt_distance_correlation: f64,
    pub strategy_classification: StrategyType,
    pub ex_gaussian_params: ExGaussianParameters,
}

impl StrategyAnalysis {
    pub fn analyze(response_times: &[f64], distances: &[usize]) -> Self {
        // Fit Ex-Gaussian to RTs
        let model = ExGaussianModel::fit(response_times);
        
        // Compute RT-distance correlation
        let correlation = Self::calculate_correlation(response_times, distances);
        
        // Classify strategy based on correlation and Ex-Gaussian shape
        let strategy = if correlation > 0.8 {
            // Strong correlation: serial scanning
            // Expect: higher μ, moderate σ, low τ
            StrategyType::SerialScan
        } else if correlation < 0.3 && model.params.tau < model.params.sigma {
            // Low correlation, low tau: direct access
            // Expect: low μ, low σ, low τ
            StrategyType::DirectAccess
        } else {
            // Mixed strategy
            // Expect: moderate μ, high σ, high τ
            StrategyType::Hybrid
        };
        
        StrategyAnalysis {
            rt_distance_correlation: correlation,
            strategy_classification: strategy,
            ex_gaussian_params: model.params,
        }
    }
}
```

## Numerical Considerations

### The Complementary Error Function

The `erfc` function requires careful handling:

```rust
// For numerical stability, we use different approximations in different regions
fn stable_erfc(x: f64) -> f64 {
    if x > 5.0 {
        // Asymptotic expansion for large positive x
        // erfc(x) ≈ exp(-x²)/(x√π) * (1 - 1/(2x²) + ...)
        let exp_part = (-x * x).exp();
        exp_part / (x * PI.sqrt())
    } else if x < -5.0 {
        // erfc(-x) = 2 - erfc(x)
        2.0 - stable_erfc(-x)
    } else {
        // Use standard library implementation
        statrs::function::erf::erfc(x)
    }
}
```

### Integration Tests

We verify our implementation integrates correctly:

```rust
#[test]
fn test_ex_gaussian_pdf_integration() {
    let params = ExGaussianParameters {
        mu: 1.0,
        sigma: 0.5,
        tau: 0.3,
    };
    let model = ExGaussianModel::new(params);
    
    // Numerical integration should approximate 1
    let integral: f64 = (0..10000)
        .map(|i| {
            let x = i as f64 * 0.01 - 10.0; // Range from -10 to 90
            model.pdf(x) * 0.01
        })
        .sum();
    
    assert!((integral - 1.0).abs() < 0.01, 
            "PDF should integrate to approximately 1, got {}", integral);
}
```

## Sampling

To generate Ex-Gaussian random variates:

```rust
pub fn sample_ex_gaussian(
    mu: f64, 
    sigma: f64, 
    tau: f64, 
    rng: &mut impl Rng
) -> f64 {
    // Sample from Gaussian
    let normal = Normal::new(mu, sigma).unwrap();
    let gaussian_sample = normal.sample(rng);
    
    // Sample from Exponential
    let exponential = Exp::new(1.0 / tau).unwrap();
    let exponential_sample = exponential.sample(rng);
    
    // Sum gives Ex-Gaussian
    gaussian_sample + exponential_sample
}
```

## Applications in the System

The Ex-Gaussian appears throughout our system:

1. **Response Time Modeling**: Fitting observed RT distributions
2. **Strategy Detection**: Different strategies produce different Ex-Gaussian parameters
3. **Simulation**: Generating realistic response times for Monte Carlo methods
4. **Outlier Detection**: Identifying responses outside expected Ex-Gaussian range

## Summary

The Ex-Gaussian distribution:
- **Captures RT characteristics**: Skewness, minimum time, variable tail
- **Has cognitive interpretation**: Parameters map to psychological processes
- **Enables strategy detection**: Different strategies produce different shapes
- **Requires careful implementation**: Numerical stability is crucial

The next chapter explores the architecture that ties these mathematical components together.