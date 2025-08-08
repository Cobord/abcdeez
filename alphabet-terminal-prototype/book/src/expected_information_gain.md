# Expected Information Gain

Expected Information Gain (EIG) is the heart of adaptive task selection. It quantifies how much we expect to learn from a task before seeing the response.

## The Core Idea

Imagine you're trying to understand what someone knows about the alphabet. You could ask:
1. "What comes after A?" (probably too easy)
2. "What comes after Q?" (moderately informative)  
3. "What comes after ℵ?" (nonsensical)

EIG helps us pick the task that will tell us the most about what they know.

## Mathematical Definition

EIG is the expected reduction in entropy:

```
EIG(task) = H(θ) - E[H(θ|response)]
         = E[D_KL(P(θ|response) || P(θ))]
```

Where:
- `H(θ)`: Current entropy (uncertainty) about parameters
- `H(θ|response)`: Expected entropy after seeing response
- `D_KL`: Kullback-Leibler divergence

## Monte Carlo Estimation

The expectation integral is usually intractable, so we use Monte Carlo:

```rust
pub fn monte_carlo_eig(&self, task: &Task, n_samples: usize) -> f64 {
    let mut total_kl = 0.0;
    let mut rng = thread_rng();
    
    for _ in 0..n_samples {
        // 1. Sample a model from current posterior
        let sampled_model = self.sample_from_posterior(&mut rng);
        
        // 2. Simulate a response using sampled model
        let (response, response_time) = 
            sampled_model.simulate_response(task, &mut rng);
        
        // 3. Create hypothetical response data
        let response_data = ResponseData {
            task: task.clone(),
            correct: response,
            response_time,
        };
        
        // 4. Compute posterior after hypothetical update
        let mut hypothetical_posterior = self.clone();
        hypothetical_posterior.update_with_response(response_data);
        
        // 5. Compute KL divergence
        let kl = self.compute_kl_divergence(&hypothetical_posterior);
        total_kl += kl;
    }
    
    total_kl / n_samples as f64
}
```

## Step-by-Step Breakdown

### Step 1: Sample from Posterior

We sample a possible "true" model from our current beliefs:

```rust
fn sample_from_posterior(&self, rng: &mut impl Rng) -> SampledModel {
    let mut positions = HashMap::new();
    
    for (node_id, posterior) in &self.node_positions {
        // Sample from Gaussian posterior
        let position = Normal::new(
            posterior.mean, 
            posterior.variance.sqrt()
        ).unwrap().sample(rng);
        
        positions.insert(node_id.clone(), position);
    }
    
    SampledModel { positions, /* ... */ }
}
```

### Step 2: Simulate Response

Given the sampled model, simulate how a learner would respond:

```rust
impl SampledModel {
    fn simulate_response(&self, task: &Task, rng: &mut impl Rng) 
        -> (bool, f64) 
    {
        // Compute success probability for this task
        let p_success = self.compute_success_probability(task);
        
        // Sample correctness
        let correct = rng.gen_bool(p_success);
        
        // Sample response time from Ex-Gaussian
        let rt_params = self.compute_rt_parameters(task, correct);
        let response_time = self.sample_ex_gaussian(rt_params, rng);
        
        (correct, response_time)
    }
}
```

### Step 3: Hypothetical Update

We update a copy of our model with the simulated response:

```rust
let mut hypothetical_posterior = self.clone();
hypothetical_posterior.update_with_response(response_data);
```

### Step 4: Compute KL Divergence

For Gaussian posteriors, KL divergence has a closed form:

```rust
fn compute_kl_divergence(&self, other: &Self) -> f64 {
    let mut total_kl = 0.0;
    
    for (node_id, posterior) in &self.node_positions {
        let other_posterior = &other.node_positions[node_id];
        
        // KL divergence between two Gaussians
        let kl = gaussian_kl_divergence(
            posterior.mean, posterior.variance,
            other_posterior.mean, other_posterior.variance
        );
        
        total_kl += kl;
    }
    
    total_kl
}

fn gaussian_kl_divergence(
    mu1: f64, var1: f64,
    mu2: f64, var2: f64
) -> f64 {
    0.5 * (
        var2.ln() - var1.ln() 
        + var1 / var2 
        + (mu1 - mu2).powi(2) / var2 
        - 1.0
    )
}
```

## Task Selection Strategy

The system ranks tasks by EIG and selects the most informative:

```rust
pub fn rank_tasks_by_eig(&self, tasks: Vec<Task>) -> Vec<(Task, f64)> {
    let mut ranked = Vec::new();
    
    for task in tasks {
        let eig = self.monte_carlo_eig(&task, 1000);
        ranked.push((task, eig));
    }
    
    // Sort by EIG (descending)
    ranked.sort_by(|a, b| 
        b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal)
    );
    
    ranked
}
```

## Convergence Diagnostics

### Determining Sample Size

The number of Monte Carlo samples should be determined dynamically:

```rust
pub fn adaptive_monte_carlo_eig(&self, task: &Task) -> (f64, usize) {
    const MIN_SAMPLES: usize = 100;
    const MAX_SAMPLES: usize = 10000;
    const RELATIVE_ERROR_THRESHOLD: f64 = 0.01; // 1% relative error
    
    let mut samples = Vec::new();
    let mut running_mean = 0.0;
    let mut running_var = 0.0;
    
    for i in 0..MAX_SAMPLES {
        // Generate sample
        let kl = self.compute_single_sample_eig(task);
        samples.push(kl);
        
        // Update statistics
        let delta = kl - running_mean;
        running_mean += delta / (i + 1) as f64;
        running_var += delta * (kl - running_mean);
        
        // Check convergence after minimum samples
        if i >= MIN_SAMPLES {
            let std_error = (running_var / (i * (i + 1)) as f64).sqrt();
            let relative_error = std_error / running_mean.abs().max(1e-10);
            
            if relative_error < RELATIVE_ERROR_THRESHOLD {
                return (running_mean, i + 1);
            }
        }
    }
    
    // Warning: did not converge
    eprintln!("Warning: Monte Carlo EIG did not converge after {} samples", MAX_SAMPLES);
    (running_mean, MAX_SAMPLES)
}
```

### Effective Sample Size

Account for correlation between samples:

```rust
pub fn effective_sample_size(samples: &[f64]) -> f64 {
    let n = samples.len() as f64;
    let mean = samples.iter().sum::<f64>() / n;
    
    // Compute autocorrelation at lag 1
    let mut c0 = 0.0;
    let mut c1 = 0.0;
    
    for i in 0..samples.len() {
        c0 += (samples[i] - mean).powi(2);
        if i > 0 {
            c1 += (samples[i] - mean) * (samples[i-1] - mean);
        }
    }
    
    let autocorr = c1 / c0;
    
    // ESS = n / (1 + 2 * sum of autocorrelations)
    // For AR(1) approximation:
    n / (1.0 + 2.0 * autocorr / (1.0 - autocorr))
}
```

## Computational Optimizations

### Caching Sampled Models

Instead of resampling for each task:

```rust
pub fn batch_eig_computation(&self, tasks: &[Task]) -> Vec<f64> {
    let n_samples = 1000;
    let mut rng = thread_rng();
    
    // Pre-sample models once
    let sampled_models: Vec<_> = (0..n_samples)
        .map(|_| self.sample_from_posterior(&mut rng))
        .collect();
    
    tasks.iter().map(|task| {
        let total_kl: f64 = sampled_models.iter()
            .map(|model| {
                let (response, rt) = model.simulate_response(task, &mut rng);
                // ... compute KL
            })
            .sum();
        
        total_kl / n_samples as f64
    }).collect()
}
```

### Parallel Computation

EIG computations are embarrassingly parallel:

```rust
use rayon::prelude::*;

pub fn parallel_eig(&self, task: &Task, n_samples: usize) -> f64 {
    let kl_sum: f64 = (0..n_samples)
        .into_par_iter()
        .map(|_| {
            // Each thread computes one sample's KL
            let model = self.sample_from_posterior();
            // ... compute KL
        })
        .sum();
    
    kl_sum / n_samples as f64
}
```

## Theoretical Properties

### Upper Bound

EIG is bounded by the current entropy:

```rust
assert!(eig <= self.total_entropy());
```

### Monotonicity

More samples always improve the estimate:

```rust
let eig_100 = self.monte_carlo_eig(&task, 100);
let eig_1000 = self.monte_carlo_eig(&task, 1000);
// eig_1000 has lower variance than eig_100
```

### Submodularity

The marginal value of information decreases (diminishing returns):

```rust
// EIG of task A after observing B ≤ EIG of task A alone
let eig_a = model.monte_carlo_eig(&task_a, 1000);
model.update_with_response(response_b);
let eig_a_after_b = model.monte_carlo_eig(&task_a, 1000);
assert!(eig_a_after_b <= eig_a);
```

## Practical Considerations

### Sample Size Selection

How many Monte Carlo samples do we need?

```rust
fn determine_sample_size(desired_stderr: f64) -> usize {
    // Standard error decreases as 1/√n
    // For stderr < 0.01 with typical variance ~0.1:
    let variance_estimate = 0.1;
    let n = (variance_estimate / desired_stderr.powi(2)) as usize;
    n.max(100).min(10000) // Practical bounds
}
```

### Numerical Stability

Avoid numerical issues in KL computation:

```rust
fn stable_gaussian_kl(mu1: f64, var1: f64, mu2: f64, var2: f64) -> f64 {
    // Avoid log(0) and division by zero
    let var1 = var1.max(1e-10);
    let var2 = var2.max(1e-10);
    
    // Avoid overflow in squared difference
    let diff = (mu1 - mu2).min(100.0).max(-100.0);
    
    0.5 * (
        var2.ln() - var1.ln() 
        + (var1 + diff * diff) / var2 
        - 1.0
    ).max(0.0) // KL is always non-negative
}
```

## Example: Alphabet Learning

Let's trace through EIG computation for a specific task:

```rust
// Task: "What comes after M?"
let task = Task {
    task_type: TaskType::Successor { item: "M".to_string() },
    // ...
};

// Current beliefs about M and N positions
// M: mean=12.5, variance=2.0 (somewhat uncertain)
// N: mean=13.8, variance=1.5 (fairly confident)

// Sample 1: M=11.8, N=14.2 → gap too large → p(correct)=0.3
// Sample 2: M=12.9, N=13.5 → reasonable → p(correct)=0.8
// Sample 3: M=13.1, N=13.2 → too close → p(correct)=0.4

// Average EIG ≈ 0.35 bits (moderately informative)
```

## Summary

Expected Information Gain provides:
- **Principled task selection**: Maximizes learning efficiency
- **Adaptive difficulty**: Automatically finds the "sweet spot"
- **Quantitative comparison**: Ranks all possible tasks
- **Theoretical guarantees**: Bounded, monotonic, submodular

The next chapter explores the Ex-Gaussian distribution used for response time modeling.