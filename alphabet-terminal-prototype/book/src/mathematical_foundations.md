# Mathematical Foundations

This chapter establishes the mathematical framework underlying the cognitive modeling system. We build from basic probability through Bayesian inference to information theory.

## Probability Distributions

### Discrete Distributions

For categorical outcomes (correct/incorrect responses), we use Bernoulli and Categorical distributions:

```rust
// Bernoulli: probability of correct response
P(correct | θ) = θ^k * (1-θ)^(1-k)
where k ∈ {0,1}
```

### Continuous Distributions  

For response times, we use the Ex-Gaussian distribution, which captures both the normal decision process and occasional slow responses:

```rust
// Ex-Gaussian: convolution of Gaussian and Exponential
f(t; μ, σ, τ) = (1/τ) * φ((t-μ)/σ) * Φ((t-μ-τ)/σ)
```

Where:
- `μ`: mean of Gaussian component (central tendency)
- `σ`: standard deviation of Gaussian (variability)  
- `τ`: rate parameter of exponential (tail heaviness)
- `φ`: standard normal PDF
- `Φ`: standard normal CDF

## Bayesian Framework

### Prior Distributions

We encode initial uncertainty about learner parameters using priors:

```rust
#[derive(Debug, Clone)]
pub struct GaussianPosterior {
    pub mean: f64,      // Current best estimate
    pub variance: f64,  // Uncertainty about estimate
}

impl GaussianPosterior {
    pub fn new_uninformative() -> Self {
        GaussianPosterior {
            mean: 0.0,      // No initial bias
            variance: 10.0, // High uncertainty
        }
    }
}
```

### Likelihood Functions

The likelihood links parameters to observations:

```rust
// Likelihood of observing response time t given parameters
pub fn likelihood(t: f64, params: &ExGaussianParameters) -> f64 {
    let model = ExGaussianModel::new(params.clone());
    model.pdf(t)
}
```

### Posterior Updates

Bayes' theorem provides the update rule:

```
P(θ | data) ∝ P(data | θ) * P(θ)
```

In code:

```rust
pub fn update_posterior(
    prior: &GaussianPosterior,
    observation: f64,
    observation_variance: f64,
) -> GaussianPosterior {
    // Kalman filter update for Gaussian-Gaussian conjugacy
    let gain = prior.variance / (prior.variance + observation_variance);
    let new_mean = prior.mean + gain * (observation - prior.mean);
    let new_variance = (1.0 - gain) * prior.variance;
    
    GaussianPosterior {
        mean: new_mean,
        variance: new_variance,
    }
}
```

## Information Theory

### Entropy

Entropy quantifies uncertainty in a probability distribution:

```rust
// Shannon entropy for discrete distribution
H(P) = -Σ p_i * log(p_i)

// Differential entropy for continuous Gaussian
H(N(μ,σ²)) = 0.5 * log(2πeσ²)
```

Implementation:

```rust
impl GaussianPosterior {
    pub fn entropy(&self) -> f64 {
        0.5 * (2.0 * PI * E * self.variance).ln()
    }
}
```

### Kullback-Leibler Divergence

KL divergence measures the "distance" between distributions:

```rust
// KL divergence from P to Q (discrete)
D_KL(P || Q) = Σ p_i * log(p_i / q_i)

// For Gaussians N₁ ~ N(μ₁, σ₁²) and N₂ ~ N(μ₂, σ₂²):
D_KL(N₁ || N₂) = 0.5 * [log(σ₂²/σ₁²) + σ₁²/σ₂² + (μ₁-μ₂)²/σ₂² - 1]

// Breaking down the terms:
// - log(σ₂²/σ₁²): variance ratio penalty
// - σ₁²/σ₂²: trace term (spread difference)
// - (μ₁-μ₂)²/σ₂²: normalized squared distance between means
```

### Mutual Information

Mutual information quantifies the reduction in uncertainty:

```rust
I(X;Y) = H(X) - H(X|Y) = H(Y) - H(Y|X)
```

## Expected Information Gain

The crown jewel of our framework is Expected Information Gain (EIG), which quantifies how much a task is expected to reduce our uncertainty:

```rust
EIG = E_y[D_KL(P(θ|y) || P(θ))]
    = ∫ P(y) * D_KL(P(θ|y) || P(θ)) dy
```

Since this integral is often intractable, we use Monte Carlo estimation:

```rust
pub fn monte_carlo_eig(&self, task: &Task, n_samples: usize) -> f64 {
    let mut total_kl = 0.0;
    
    for _ in 0..n_samples {
        // Sample from current posterior
        let theta_sample = self.sample_from_posterior();
        
        // Simulate response given sampled parameters
        let response = self.simulate_response(&theta_sample, task);
        
        // Compute posterior after hypothetical response
        let posterior = self.hypothetical_update(task, response);
        
        // Compute KL divergence
        let kl = self.kl_divergence(&posterior);
        total_kl += kl;
    }
    
    total_kl / n_samples as f64
}
```

## Practical Considerations

### Numerical Stability

Many calculations involve logarithms and exponentials that can overflow:

```rust
// Bad: can overflow
let prob = exp(log_a + log_b);

// Good: use log-sum-exp trick
let max_val = log_a.max(log_b);
let prob = max_val + ((log_a - max_val).exp() + (log_b - max_val).exp()).ln();
```

### Conjugate Priors

When possible, we use conjugate prior-likelihood pairs for closed-form updates:

| Likelihood | Conjugate Prior | Posterior |
|------------|----------------|-----------|
| Gaussian (known σ²) | Gaussian | Gaussian |
| Bernoulli | Beta | Beta |
| Categorical | Dirichlet | Dirichlet |

### Approximations

For non-conjugate cases, we use approximations:

1. **Laplace Approximation**: Approximate posterior with Gaussian at MAP
2. **Variational Inference**: Approximate with tractable family
3. **MCMC**: Sample from true posterior

## Summary

The mathematical foundations provide:
- **Probability distributions** for modeling uncertainty
- **Bayesian inference** for learning from data
- **Information theory** for quantifying knowledge
- **EIG** for optimal experiment design

These tools combine to create an adaptive system that efficiently explores a learner's knowledge state.