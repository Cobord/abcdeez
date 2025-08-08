# Bayesian Inference

This chapter explores how the system uses Bayesian inference to model and update beliefs about learner knowledge.

## The Bayesian Learner Model

The `BayesianLearnerModel` maintains probability distributions over all unknown parameters:

```rust
pub struct BayesianLearnerModel {
    // Position of each node in mental space
    pub node_positions: HashMap<String, GaussianPosterior>,
    
    // Proficiency with each operation type  
    pub operation_proficiencies: HashMap<String, GaussianPosterior>,
    
    // Chunk boundaries in the sequence
    pub chunk_boundaries: Vec<f64>,
    
    // Reference to topology structure
    topology: Topology,
}
```

## Node Position Inference

We model each node as having a latent position in mental space:

```rust
#[derive(Debug, Clone)]
pub struct GaussianPosterior {
    pub mean: f64,      // Best estimate of position
    pub variance: f64,  // Uncertainty about position
}
```

The key insight: if a learner knows the sequence well, adjacent items should have consistent spacing in their mental representation.

### Initialization

```rust
impl BayesianLearnerModel {
    pub fn new(topology: &Topology) -> Self {
        let mut node_positions = HashMap::new();
        
        for node in &topology.nodes {
            // Initialize with prior based on true position plus uncertainty
            node_positions.insert(
                node.id.clone(),
                GaussianPosterior {
                    mean: node.position as f64,
                    variance: 4.0, // Initial uncertainty
                },
            );
        }
        
        // ... initialize other components
    }
}
```

### Updating from Responses

When we observe a response, we update our beliefs:

```rust
pub fn update_with_response(&mut self, response: ResponseData) {
    match &response.task.task_type {
        TaskType::Successor { item } => {
            self.update_successor_beliefs(item, response.correct, response.response_time);
        }
        TaskType::PairwiseOrder { a, b } => {
            self.update_pairwise_beliefs(a, b, response.correct, response.response_time);
        }
        // ... other task types
    }
}
```

## Observation Models

Different task types provide different information:

### Successor Tasks

For "What comes after X?" tasks:

```rust
fn update_successor_beliefs(&mut self, item: &str, correct: bool, rt: f64) {
    // Get node IDs
    let current_id = self.topology.get_node_by_label(item).unwrap().id;
    let next_id = self.topology.get_successor(&item).unwrap();
    
    // Observation model: if correct, positions should differ by ~1
    let expected_diff = 1.0;
    let observed_diff = if correct { expected_diff } else { -expected_diff };
    
    // Observation variance increases with response time (uncertainty)
    let obs_variance = self.calculate_observation_variance(rt, correct);
    
    // Update both positions to be consistent
    self.update_relative_positions(current_id, next_id, observed_diff, obs_variance);
}
```

### Pairwise Order Tasks  

For "Does A come before B?" tasks:

```rust
fn update_pairwise_beliefs(&mut self, a: &str, b: &str, correct: bool, rt: f64) {
    let pos_a = &self.node_positions[&self.get_node_id(a)];
    let pos_b = &self.node_positions[&self.get_node_id(b)];
    
    // The observation is about relative ordering
    let observed_order_correct = 
        (pos_a.mean < pos_b.mean) == correct;
    
    if !observed_order_correct {
        // Need to adjust positions
        let midpoint = (pos_a.mean + pos_b.mean) / 2.0;
        // Push positions apart or together based on response
        self.adjust_positions_around_midpoint(a, b, midpoint, correct);
    }
}
```

## Adaptive Observation Variance

A key innovation: observation variance adapts based on response characteristics:

```rust
fn calculate_observation_variance(&self, response: &ResponseData) -> f64 {
    let base_variance = 1.0;
    
    // Fast, correct responses → low variance (confident)
    // Slow, incorrect responses → high variance (uncertain)
    
    let rt_factor = (response.response_time / 1000.0).min(3.0) / 3.0;
    let accuracy_factor = if response.correct { 1.0 } else { 0.5 };
    
    base_variance * (1.0 + 2.0 * (1.0 - accuracy_factor) + 0.5 * rt_factor)
}
```

## Hierarchical Structure

The model captures hierarchical structure through chunk boundaries:

```rust
fn detect_chunk_boundary(&mut self, between: (&str, &str), rt: f64) {
    // Longer RTs suggest chunk boundaries
    let threshold = self.mean_rt * 1.5;
    
    if rt > threshold {
        let pos = (self.get_position(between.0) + self.get_position(between.1)) / 2.0;
        self.chunk_boundaries.push(pos);
        
        // Increase position variance near boundaries (less certainty)
        self.increase_boundary_uncertainty(pos);
    }
}
```

## Posterior Sampling

For Monte Carlo methods, we need to sample from posteriors:

```rust
fn sample_from_posterior(&self, rng: &mut impl Rng) -> SampledModel {
    let mut positions = HashMap::new();
    let mut proficiencies = HashMap::new();
    
    // Sample each node position
    for (node_id, posterior) in &self.node_positions {
        let sample = Normal::new(posterior.mean, posterior.variance.sqrt())
            .unwrap()
            .sample(rng);
        positions.insert(node_id.clone(), sample);
    }
    
    // Sample operation proficiencies
    for (op_name, posterior) in &self.operation_proficiencies {
        let sample = Normal::new(posterior.mean, posterior.variance.sqrt())
            .unwrap()
            .sample(rng);
        proficiencies.insert(op_name.clone(), sample);
    }
    
    SampledModel { positions, proficiencies }
}
```

## Predictive Distribution

Given our posterior, we can predict future performance:

```rust
pub fn predict_success_probability(&self, task: &Task) -> f64 {
    // Monte Carlo integration over posterior
    let n_samples = 100;
    let mut success_count = 0.0;
    
    for _ in 0..n_samples {
        let model_sample = self.sample_from_posterior();
        let p_success = model_sample.compute_success_probability(task);
        success_count += p_success;
    }
    
    success_count / n_samples as f64
}
```

## Convergence Properties

The Bayesian updates have desirable properties:

1. **Consistency**: As data increases, posterior concentrates on true value
2. **Efficiency**: Uses all available information optimally
3. **Uncertainty Quantification**: Naturally provides confidence intervals

```rust
// Posterior variance decreases with more observations
fn theoretical_variance_after_n_observations(
    prior_variance: f64,
    observation_variance: f64,
    n: usize,
) -> f64 {
    1.0 / (1.0 / prior_variance + n as f64 / observation_variance)
}
```

## Practical Implementation

The actual implementation in `bayesian.rs` handles several subtleties:

```rust
impl BayesianLearnerModel {
    pub fn update_with_response(&mut self, response: ResponseData) {
        // 1. Calculate observation variance based on response
        let obs_var = self.calculate_observation_variance(&response);
        
        // 2. Update relevant parameters
        match &response.task.task_type {
            TaskType::Successor { item } => {
                // Update positions of current and next node
                let current_id = self.get_node_id(item);
                if let Some(next) = self.topology.get_successor(item) {
                    let next_id = self.get_node_id(&next);
                    self.update_pairwise_positions(
                        &current_id, &next_id, 
                        response.correct, obs_var
                    );
                }
            }
            // ... handle other task types
        }
        
        // 3. Update operation proficiency
        self.update_operation_proficiency(&response.task.operation, response.correct);
        
        // 4. Check for chunk boundaries
        if response.response_time > self.mean_response_time * 1.5 {
            self.update_chunk_boundaries(&response.task);
        }
    }
}
```

## Summary

Bayesian inference provides:
- **Principled uncertainty**: Every belief has associated confidence
- **Optimal updates**: Uses all information efficiently
- **Predictive power**: Natural predictions with uncertainty
- **Adaptive behavior**: Adjusts to learner characteristics

The next chapter explores how we use these posteriors to compute Expected Information Gain.