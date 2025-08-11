# Learner Model

The Learner Model is the heart of ABCDeez Core's cognitive assessment system. It maintains a comprehensive representation of an individual's learning state, including their knowledge, skills, memory strengths, and strategic preferences. This model adapts continuously as new evidence is gathered, providing increasingly accurate predictions of performance.

## Core Architecture

### Complete Learner State

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerModel {
    pub learner_id: String,
    
    // Cognitive components
    pub node_embeddings: HashMap<String, LatentNodeEmbedding>,
    pub operation_proficiencies: HashMap<String, OperationProficiency>,
    pub memory_strengths: HashMap<String, MemoryStrength>,
    pub confusability_matrix: HashMap<(String, String), f64>,
    pub chunk_boundaries: Vec<ChunkBoundary>,
    
    // Meta-learning
    pub total_practice_time: Duration,
    pub session_count: usize,
    pub config: LearnerConfig,
}
```

This comprehensive state captures multiple aspects of learning:
- **Declarative knowledge**: What facts are known
- **Procedural knowledge**: How to perform operations
- **Memory dynamics**: Strength and decay of memories
- **Confusion patterns**: Which items are easily mixed up
- **Structural knowledge**: How information is organized

## Node Embeddings: Spatial Knowledge Representation

### Latent Space Model

Each item in the knowledge domain is represented as a point in latent space:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentNodeEmbedding {
    pub node_id: String,
    pub position: f64,        // Position in latent space
    pub uncertainty: f64,     // How confident we are about this position
}
```

### Updating Node Positions

As learners demonstrate knowledge, node positions are refined:

```rust
impl LearnerModel {
    pub fn update_node_embedding(
        &mut self,
        node_id: &str,
        new_position: f64,
        reduce_uncertainty: f64,
    ) {
        if let Some(embedding) = self.node_embeddings.get_mut(node_id) {
            // Weighted update of position
            let old_weight = self.config.position_update_weight;
            let new_weight = 1.0 - old_weight;
            
            embedding.position = old_weight * embedding.position + 
                               new_weight * new_position;
            
            // Reduce uncertainty with learning
            embedding.uncertainty *= (1.0 - reduce_uncertainty)
                .max(self.config.min_uncertainty);
        }
    }
}
```

### Identifiability Constraints

To prevent gauge freedom in the embedding space, we apply constraints:

```rust
pub fn apply_identifiability_constraints(&mut self) {
    // Fix first and last nodes to prevent rotation/translation
    if let Some(first_node) = self.node_embeddings.values()
        .min_by_key(|n| n.position as i64) {
        let first_id = first_node.node_id.clone();
        if let Some(first) = self.node_embeddings.get_mut(&first_id) {
            first.position = 0.0;
            first.uncertainty = 0.01;  // Very certain about anchor
        }
    }
    
    if let Some(last_node) = self.node_embeddings.values()
        .max_by_key(|n| n.position as i64) {
        let last_id = last_node.node_id.clone();
        let n_nodes = self.node_embeddings.len() as f64;
        if let Some(last) = self.node_embeddings.get_mut(&last_id) {
            last.position = n_nodes - 1.0;
            last.uncertainty = 0.01;
        }
    }
    
    // Center the embeddings to prevent drift
    self.center_embeddings();
}
```

## Operation Proficiency: Skill Development

### Proficiency Representation

Each cognitive operation has associated skill parameters:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProficiency {
    pub operation: OperationType,
    pub theta: f64,           // Logit-scale proficiency [-∞, ∞]
    pub practice_count: usize, // Number of practice attempts
}
```

### Adaptive Learning Rate

Learning rates adapt based on current proficiency and experience:

```rust
pub fn update_operation_proficiency(&mut self, operation: &OperationType, success: bool) {
    let key = format!("{:?}", operation);
    if let Some(prof) = self.operation_proficiencies.get_mut(&key) {
        prof.practice_count += 1;

        // Base learning rate decreases with practice (power law)
        let base_rate = self.config.learning_rate_base /
                       (1.0 + prof.practice_count as f64)
                       .powf(self.config.learning_rate_decay);

        // Adjust based on current proficiency level
        let theta_range = self.config.theta_bounds.1 - self.config.theta_bounds.0;
        let proficiency_factor = 1.0 - (prof.theta.abs() / (theta_range / 2.0))
                                      .min(1.0);

        // Calculate adaptive learning rate
        let learning_rate = (base_rate * (0.5 + proficiency_factor))
                           .max(0.01).min(0.5);

        if success {
            // Diminishing returns as proficiency increases
            prof.theta += learning_rate * (1.0 - sigmoid(prof.theta));
        } else {
            // Larger penalty for errors at high proficiency
            let error_weight = if prof.theta > 1.0 { 1.5 } else { 1.0 };
            prof.theta -= learning_rate * sigmoid(prof.theta) * error_weight;
        }

        // Keep theta within bounds
        prof.theta = prof.theta
            .max(self.config.theta_bounds.0)
            .min(self.config.theta_bounds.1);
    }
}
```

### Operation Types

The system tracks proficiency across multiple cognitive operations:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    Successor,                    // A → B
    Predecessor,                  // B → A  
    PairwiseOrder,               // A < B?
    KJump(i32),                  // A → C (skip B)
    Segment(usize, bool),        // ABC... (count, reverse)
    Index,                       // Position of A?
}
```

## Memory Dynamics: Forgetting and Retention

### Memory Strength with Decay

Memory strength follows empirically-validated forgetting curves:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStrength {
    pub node_id: String,
    pub strength: f64,                           // Current strength [0, 1]
    pub last_practice: DateTime<Utc>,            // When last practiced
}
```

### Strength-Dependent Decay

Stronger memories decay more slowly:

```rust
fn calculate_decay_rate(&self, current_strength: f64) -> f64 {
    // Stronger memories are more resistant to decay
    self.config.memory_decay_rate * (2.0 - current_strength)
}

fn apply_forgetting_curve(&self, strength: f64, hours_elapsed: f64, decay_rate: f64) -> f64 {
    strength * (-decay_rate * hours_elapsed).exp()
}
```

### Memory Updates

Memory updates consider both decay and new practice:

```rust
pub fn update_memory_strength(&mut self, node_id: &str, correct: bool) {
    if let Some(mem) = self.memory_strengths.get_mut(node_id) {
        let now = Utc::now();
        let time_since = now.signed_duration_since(mem.last_practice);
        let hours_since = time_since.num_hours() as f64;

        // Apply decay first
        let current_strength = mem.strength;
        let decay_rate = self.calculate_decay_rate(current_strength);
        let decayed_strength = self.apply_forgetting_curve(
            current_strength, hours_since, decay_rate
        );

        // Update based on performance
        if correct {
            mem.strength = (decayed_strength + self.config.memory_update_correct)
                          .min(1.0);
        } else {
            mem.strength = (decayed_strength + self.config.memory_update_incorrect)
                          .max(0.0);
        }

        mem.last_practice = now;
    }
}
```

### Optimal Review Scheduling

The model can predict when items should be reviewed:

```rust
pub fn get_optimal_review_time(
    &self,
    node_id: &str,
    target_retention: f64,
) -> Option<DateTime<Utc>> {
    let mem = self.memory_strengths.get(node_id)?;

    if mem.strength <= target_retention {
        return Some(Utc::now());  // Review immediately
    }

    // Calculate when strength will drop to target
    let decay_rate = self.calculate_decay_rate(mem.strength);
    let hours_until_review = -(target_retention / mem.strength).ln() / decay_rate;

    Some(mem.last_practice + Duration::hours(hours_until_review as i64))
}
```

## Confusability Matrix: Error Patterns

### Tracking Confusion Patterns

The system learns which items are frequently confused:

```rust
pub fn update_confusability(&mut self, node_a: &str, node_b: &str, confused: bool) {
    let key = if node_a < node_b {
        (node_a.to_string(), node_b.to_string())
    } else {
        (node_b.to_string(), node_a.to_string())
    };

    let current = self.confusability_matrix.get(&key).unwrap_or(&0.0);
    let new_value = if confused {
        (current + 0.1).min(1.0)    // Increase confusion
    } else {
        (current * 0.9).max(0.0)    // Decrease confusion
    };

    self.confusability_matrix.insert(key, new_value);
}
```

### Using Confusion for Prediction

Confusion patterns inform difficulty predictions:

```rust
pub fn predict_confusion_difficulty(&self, item_a: &str, item_b: &str) -> f64 {
    let key = if item_a < item_b {
        (item_a.to_string(), item_b.to_string())
    } else {
        (item_b.to_string(), item_a.to_string())
    };
    
    self.confusability_matrix.get(&key).unwrap_or(&0.0) * 0.5
}
```

## Chunk Boundaries: Structural Knowledge

### Detecting Natural Chunks

The model discovers how learners segment knowledge:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkBoundary {
    pub position: usize,     // Where the boundary occurs
    pub strength: f64,       // How strong the boundary is
}

pub fn update_chunk_boundaries(&mut self, crossed_boundary: Option<usize>) {
    if let Some(pos) = crossed_boundary {
        if let Some(boundary) = self.chunk_boundaries.iter_mut()
            .find(|b| b.position == pos) {
            // Strengthen existing boundary
            boundary.strength = (boundary.strength + 0.1).min(1.0);
        } else {
            // Create new boundary
            self.chunk_boundaries.push(ChunkBoundary {
                position: pos,
                strength: 0.1,
            });
        }
    }
}
```

### Chunk-Based Response Time Prediction

Chunk boundaries affect processing time:

```rust
pub fn predict_response_time(&self, operation: &OperationType, distance: usize) -> f64 {
    let base_rt = 1000.0;                    // Base response time
    let distance_penalty = 100.0 * distance as f64;  // Linear distance effect
    
    // Proficiency reduces response time
    let proficiency_bonus = self.get_operation_proficiency(operation)
        .map(|p| 200.0 * sigmoid(p))
        .unwrap_or(0.0);
    
    // Chunk boundary crossing penalty
    let boundary_penalty = self.chunk_boundaries
        .iter()
        .filter(|b| b.position < distance)
        .map(|b| 200.0 * b.strength)
        .sum::<f64>();
    
    (base_rt + distance_penalty + boundary_penalty - proficiency_bonus)
        .max(300.0)  // Minimum response time
}
```

## Performance Predictions

### Probability of Correct Response

```rust
pub fn get_probability_correct(&self, operation: &OperationType, difficulty: f64) -> f64 {
    let key = format!("{:?}", operation);
    let prof = self.operation_proficiencies.get(&key);
    
    let theta = prof.map(|p| p.theta).unwrap_or(-1.0);
    
    // Logistic model: P(correct) = 1 / (1 + exp(-(θ - β)))
    sigmoid(theta - difficulty)
}
```

### Individual Difference Metrics

The model computes interpretable metrics:

```rust
pub fn get_bidirectionality_index(&self) -> f64 {
    let forward = self.operation_proficiencies
        .get(&format!("{:?}", OperationType::Successor));
    let backward = self.operation_proficiencies
        .get(&format!("{:?}", OperationType::Predecessor));
    
    match (forward, backward) {
        (Some(f), Some(b)) => (f.theta - b.theta).abs(),
        _ => 1.0,  // High asymmetry if only one direction learned
    }
}

pub fn get_symbolic_distance_slope(&self) -> f64 {
    let distances = vec![1, 2, 3, 4, 5];
    let rts: Vec<f64> = distances.iter()
        .map(|&d| self.predict_response_time(&OperationType::PairwiseOrder, d))
        .collect();
    
    // Linear regression slope
    calculate_slope(&distances, &rts)
}
```

## Regularization and Constraints

### Embedding Regularization

Prevent extreme or inconsistent embeddings:

```rust
pub fn regularize_embeddings(&mut self, lambda: f64) {
    // Sort nodes by position
    let mut sorted_nodes: Vec<_> = self.node_embeddings.values().cloned().collect();
    sorted_nodes.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());

    // Maintain minimum spacing
    let min_spacing = 0.1;
    for i in 1..sorted_nodes.len() {
        let prev_pos = sorted_nodes[i - 1].position;
        let curr_pos = sorted_nodes[i].position;

        if curr_pos - prev_pos < min_spacing {
            if let Some(node) = self.node_embeddings.get_mut(&sorted_nodes[i].node_id) {
                node.position = prev_pos + min_spacing;
            }
        }
    }

    // L2 regularization toward expected positions
    for embedding in self.node_embeddings.values_mut() {
        let expected_pos = calculate_expected_position(&embedding.node_id);
        embedding.position = (1.0 - lambda) * embedding.position + 
                           lambda * expected_pos;
    }
}
```

## Configuration and Profiles

### Learner Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerConfig {
    // Memory parameters
    pub initial_memory_strength: f64,    // Starting strength
    pub memory_decay_rate: f64,          // Base decay rate
    pub memory_update_correct: f64,      // Boost for correct response
    pub memory_update_incorrect: f64,    // Penalty for incorrect response
    
    // Learning parameters  
    pub learning_rate_base: f64,         // Initial learning rate
    pub learning_rate_decay: f64,        // Decay exponent
    pub initial_proficiency: f64,        // Starting θ value
    
    // Individual differences
    pub edge_emphasis: f64,              // Serial position effect strength
    pub min_uncertainty: f64,            // Floor for uncertainty
    
    // Constraints
    pub theta_bounds: (f64, f64),        // Valid θ range
}
```

### Age-Appropriate Profiles

```rust
impl LearnerConfig {
    pub fn child() -> Self {
        Self {
            initial_memory_strength: 0.3,   // Weaker initial memory
            memory_decay_rate: 0.08,        // Faster forgetting
            learning_rate_base: 0.3,        // Higher learning rate
            edge_emphasis: 0.2,             // Strong serial position effects
            ..Default::default()
        }
    }
    
    pub fn adult() -> Self {
        Self {
            initial_memory_strength: 0.5,   // Moderate initial memory
            memory_decay_rate: 0.05,        // Moderate forgetting
            learning_rate_base: 0.2,        // Moderate learning rate
            edge_emphasis: 0.1,             // Moderate serial position effects
            ..Default::default()
        }
    }
    
    pub fn expert() -> Self {
        Self {
            initial_memory_strength: 0.7,   // Strong initial memory
            memory_decay_rate: 0.03,        // Slow forgetting
            learning_rate_base: 0.15,       // Lower learning rate
            edge_emphasis: 0.05,            // Weak serial position effects
            ..Default::default()
        }
    }
}
```

## Integration with Response Data

### Processing Responses

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub task: Task,
    pub correct: bool,
    pub response_time: f64,
}

impl LearnerModel {
    pub fn update_with_response(&mut self, response: &ResponseData) {
        // Update operation proficiency
        self.update_operation_proficiency(&response.task.operation, response.correct);
        
        // Update memory strength for relevant items
        for item in response.task.get_relevant_items() {
            self.update_memory_strength(&item, response.correct);
        }
        
        // Update confusion matrix if applicable
        if let Some((item_a, item_b)) = response.task.get_item_pair() {
            self.update_confusability(&item_a, &item_b, !response.correct);
        }
        
        // Update embeddings based on performance patterns
        self.update_embeddings_from_response(response);
        
        // Apply regularization periodically
        if self.session_count % 10 == 0 {
            self.apply_identifiability_constraints();
            self.regularize_embeddings(0.01);
        }
    }
}
```

## Model Export and Analysis

### Learner Metrics

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerMetrics {
    pub bidirectionality_index: f64,
    pub symbolic_distance_slope: f64,
    pub chunk_boundary_penalty: f64,
    pub avg_memory_strength: f64,
    pub operation_proficiencies: HashMap<String, f64>,
}

impl LearnerMetrics {
    pub fn from_model(model: &LearnerModel) -> Self {
        let avg_memory_strength = model.memory_strengths.values()
            .map(|m| m.strength)
            .sum::<f64>() / model.memory_strengths.len().max(1) as f64;

        let mut operation_proficiencies = HashMap::new();
        for (key, prof) in &model.operation_proficiencies {
            operation_proficiencies.insert(key.clone(), sigmoid(prof.theta));
        }

        Self {
            bidirectionality_index: model.get_bidirectionality_index(),
            symbolic_distance_slope: model.get_symbolic_distance_slope(),
            chunk_boundary_penalty: model.get_chunk_boundary_penalty(),
            avg_memory_strength,
            operation_proficiencies,
        }
    }
}
```

## Best Practices

1. **Initialize sensibly**: Use age-appropriate configurations
2. **Update incrementally**: Process responses one at a time
3. **Apply constraints**: Prevent pathological parameter values
4. **Monitor convergence**: Track learning curves and stability
5. **Export regularly**: Save model state for analysis
6. **Validate assumptions**: Check model fit to data

## Common Patterns

### Session Management

```rust
pub struct LearningSession {
    learner: LearnerModel,
    start_time: DateTime<Utc>,
    responses: Vec<ResponseData>,
}

impl LearningSession {
    pub fn process_response(&mut self, response: ResponseData) {
        self.learner.update_with_response(&response);
        self.responses.push(response);
    }
    
    pub fn get_performance_summary(&self) -> SessionSummary {
        SessionSummary {
            accuracy: self.calculate_accuracy(),
            avg_response_time: self.calculate_avg_rt(),
            learning_gains: self.calculate_learning_gains(),
            items_practiced: self.get_unique_items().len(),
        }
    }
}
```

### Adaptive Difficulty

```rust
pub fn get_adaptive_difficulty(&self, operation: &OperationType) -> f64 {
    let proficiency = self.get_operation_proficiency(operation)
        .unwrap_or(-1.0);
    
    // Target ~75% accuracy
    let target_logit = 1.1;  // log(3) ≈ 1.1 for 75% accuracy
    
    (proficiency - target_logit).max(0.1).min(1.0)
}
```

## Next Steps

- Learn about [Node Embeddings](./embeddings.md) in detail
- Explore [Operation Proficiency](./proficiency.md) mechanisms  
- Understand [Memory & Forgetting](./memory.md) dynamics
- See [Chunk Boundaries](./chunking.md) detection