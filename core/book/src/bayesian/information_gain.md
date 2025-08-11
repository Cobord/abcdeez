# Expected Information Gain

Expected Information Gain (EIG) is the cornerstone of adaptive assessment in ABCDeez Core. It quantifies how much uncertainty about a learner's state would be reduced by observing their response to a specific task.

## Mathematical Foundation

### Information Theory Basics

Information gain measures the reduction in entropy:

```math
IG = H(θ) - H(θ|response)
```

Where:
- **H(θ)**: Current entropy (uncertainty) about learner parameters
- **H(θ|response)**: Expected posterior entropy after observing response

### Expected Information Gain

Since we don't know the response beforehand, we take the expectation:

```math
EIG = ∫ P(response|task) · [H(θ) - H(θ|response, task)] dresponse
```

## Implementation in ABCDeez Core

### Monte Carlo Approximation

Computing EIG analytically is often intractable, so we use Monte Carlo methods:

```rust
impl BayesianLearnerModel {
    pub fn monte_carlo_eig(&self, task: &Task, n_samples: usize) -> f64 {
        let mut total_ig = 0.0;
        
        // Current entropy
        let current_entropy = self.total_entropy();
        
        for _ in 0..n_samples {
            // Sample from current posterior
            let theta_sample = self.sample_from_posterior();
            
            // Simulate response given sampled parameters
            let response = self.simulate_response(&theta_sample, task);
            
            // Create hypothetical updated model
            let mut hypothetical = self.clone();
            hypothetical.update_with_response(&response);
            
            // Calculate entropy reduction
            let posterior_entropy = hypothetical.total_entropy();
            let ig = current_entropy - posterior_entropy;
            
            total_ig += ig;
        }
        
        total_ig / n_samples as f64
    }
}
```

### Efficient Computation Strategies

#### 1. Importance Sampling

Focus samples on high-probability regions:

```rust
pub fn importance_sampled_eig(&self, task: &Task) -> f64 {
    // Use proposal distribution centered on MAP estimate
    let map_estimate = self.get_map_estimate();
    let proposal = Gaussian::new(map_estimate, self.uncertainty);
    
    let mut weighted_ig = 0.0;
    let mut total_weight = 0.0;
    
    for _ in 0..N_SAMPLES {
        let theta = proposal.sample();
        let weight = self.posterior_density(theta) / proposal.density(theta);
        
        let response = self.simulate_response(&theta, task);
        let ig = self.calculate_ig_for_response(&response, task);
        
        weighted_ig += weight * ig;
        total_weight += weight;
    }
    
    weighted_ig / total_weight
}
```

#### 2. Gradient-Based Optimization

For continuous parameter spaces:

```rust
pub fn gradient_eig(&self, task: &Task) -> Vec<f64> {
    // Compute gradient of EIG with respect to task parameters
    let eps = 1e-6;
    let base_eig = self.monte_carlo_eig(task, 100);
    
    let mut gradients = Vec::new();
    for param_idx in 0..task.param_count() {
        let mut perturbed_task = task.clone();
        perturbed_task.perturb_param(param_idx, eps);
        
        let perturbed_eig = self.monte_carlo_eig(&perturbed_task, 100);
        gradients.push((perturbed_eig - base_eig) / eps);
    }
    
    gradients
}
```

## Task Selection Strategies

### 1. Maximum EIG

Select the task with highest expected information gain:

```rust
pub fn select_max_eig_task(&self, candidates: &[Task]) -> Task {
    candidates
        .iter()
        .map(|task| (task, self.monte_carlo_eig(task, 500)))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(task, _)| task.clone())
        .unwrap()
}
```

### 2. EIG with Constraints

Balance information gain with other objectives:

```rust
pub fn select_balanced_task(&self, candidates: &[Task]) -> Task {
    candidates
        .iter()
        .map(|task| {
            let eig = self.monte_carlo_eig(task, 200);
            let difficulty_match = self.difficulty_alignment(task);
            let recency_penalty = self.recency_penalty(task);
            
            // Weighted combination
            let score = 0.6 * eig + 
                       0.3 * difficulty_match - 
                       0.1 * recency_penalty;
            (task, score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(task, _)| task.clone())
        .unwrap()
}
```

### 3. Myopic vs. Non-Myopic

Myopic selection considers only immediate gain:

```rust
pub fn myopic_selection(&self, candidates: &[Task]) -> Task {
    // Standard EIG calculation
    self.select_max_eig_task(candidates)
}
```

Non-myopic looks ahead multiple steps:

```rust
pub fn non_myopic_selection(
    &self, 
    candidates: &[Task], 
    horizon: usize
) -> Task {
    candidates
        .iter()
        .map(|task| {
            let immediate_eig = self.monte_carlo_eig(task, 100);
            
            // Simulate future trajectories
            let future_value = self.simulate_future_value(task, horizon);
            
            (task, immediate_eig + 0.9 * future_value)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(task, _)| task.clone())
        .unwrap()
}
```

## Entropy Calculations

### Node-Level Entropy

```rust
impl BayesianLearnerModel {
    pub fn node_entropy(&self, node_id: &str) -> f64 {
        if let Some(embedding) = self.node_embeddings.get(node_id) {
            // Entropy of Gaussian distribution
            let variance = embedding.uncertainty.powi(2);
            0.5 * (2.0 * PI * E * variance).ln()
        } else {
            0.0
        }
    }
}
```

### Operation-Level Entropy

```rust
pub fn operation_entropy(&self, op: &OperationType) -> f64 {
    if let Some(prof) = self.operation_proficiencies.get(&format!("{:?}", op)) {
        // Beta distribution entropy for proficiency
        let alpha = prof.success_count + 1.0;
        let beta = prof.failure_count + 1.0;
        
        beta_entropy(alpha, beta)
    } else {
        MAX_ENTROPY
    }
}
```

### Total Model Entropy

```rust
pub fn total_entropy(&self) -> f64 {
    let node_entropy: f64 = self.node_embeddings
        .values()
        .map(|e| self.node_entropy(&e.node_id))
        .sum();
    
    let op_entropy: f64 = self.operation_proficiencies
        .values()
        .map(|p| self.operation_entropy(&p.operation))
        .sum();
    
    node_entropy + op_entropy
}
```

## Practical Considerations

### Computational Budget

Balance accuracy vs. speed:

```rust
pub struct AdaptiveEIG {
    min_samples: usize,
    max_samples: usize,
    convergence_threshold: f64,
}

impl AdaptiveEIG {
    pub fn calculate(&self, model: &BayesianLearnerModel, task: &Task) -> f64 {
        let mut samples = Vec::new();
        let mut running_mean = 0.0;
        
        for n in 1..=self.max_samples {
            let ig = model.single_sample_ig(task);
            samples.push(ig);
            
            let new_mean = samples.iter().sum::<f64>() / n as f64;
            
            if n >= self.min_samples {
                let change = (new_mean - running_mean).abs();
                if change < self.convergence_threshold {
                    return new_mean;
                }
            }
            
            running_mean = new_mean;
        }
        
        running_mean
    }
}
```

### Numerical Stability

Handle edge cases gracefully:

```rust
pub fn safe_entropy(probability: f64) -> f64 {
    if probability <= 0.0 || probability >= 1.0 {
        0.0
    } else {
        -probability * probability.ln() - (1.0 - probability) * (1.0 - probability).ln()
    }
}
```

## Validation

### Synthetic Data

Verify EIG calculations with known ground truth:

```rust
#[test]
fn test_eig_reduces_uncertainty() {
    let model = BayesianLearnerModel::new(&Topology::alphabet());
    let task = Task::new(TaskType::Successor { item: "M".into() });
    
    let initial_entropy = model.total_entropy();
    let eig = model.monte_carlo_eig(&task, 1000);
    
    // Simulate actual response and update
    let response = ResponseData { task, correct: true, response_time: 1000.0 };
    let mut updated = model.clone();
    updated.update_with_response(&response);
    
    let actual_reduction = initial_entropy - updated.total_entropy();
    
    // EIG should approximate actual information gain
    assert!((eig - actual_reduction).abs() < 0.1);
}
```

### Empirical Validation

Compare adaptive vs. random selection:

```rust
#[test]
fn test_adaptive_outperforms_random() {
    let mut adaptive_model = BayesianLearnerModel::new(&topology);
    let mut random_model = BayesianLearnerModel::new(&topology);
    
    for _ in 0..20 {
        // Adaptive selection
        let adaptive_task = adaptive_model.select_max_eig_task(&all_tasks);
        simulate_and_update(&mut adaptive_model, adaptive_task);
        
        // Random selection
        let random_task = all_tasks.choose(&mut rng).unwrap();
        simulate_and_update(&mut random_model, random_task.clone());
    }
    
    // Adaptive should achieve lower entropy
    assert!(adaptive_model.total_entropy() < random_model.total_entropy());
}
```

## Applications

### 1. Computerized Adaptive Testing (CAT)

Minimize test length while maintaining precision:

```rust
pub fn adaptive_test(
    model: &mut BayesianLearnerModel,
    item_bank: &[Task],
    target_precision: f64,
) -> Vec<Task> {
    let mut selected_tasks = Vec::new();
    
    while model.total_entropy() > target_precision {
        let next_task = model.select_max_eig_task(item_bank);
        selected_tasks.push(next_task.clone());
        
        let response = collect_response(&next_task);
        model.update_with_response(&response);
        
        if selected_tasks.len() >= MAX_TEST_LENGTH {
            break;
        }
    }
    
    selected_tasks
}
```

### 2. Active Learning

Optimize learning efficiency:

```rust
pub fn active_learning_curriculum(
    learner: &LearnerModel,
    bayesian: &mut BayesianLearnerModel,
    learning_goals: &[LearningGoal],
) -> Curriculum {
    let mut curriculum = Curriculum::new();
    
    for goal in learning_goals {
        while !goal.is_achieved(learner) {
            // Select most informative task for goal
            let candidates = goal.relevant_tasks();
            let task = bayesian.select_max_eig_task(&candidates);
            
            curriculum.add_task(task);
            
            // Simulate learning
            let response = learner.predict_response(&task);
            bayesian.update_with_response(&response);
        }
    }
    
    curriculum
}
```

## Next Steps

- Understand [Posterior Updates](./updates.md)
- Learn about [Model Comparison](./comparison.md)
- Explore [Adaptive Task Selection](../tasks/adaptive.md)