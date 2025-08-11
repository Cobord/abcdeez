# Node Embeddings

Node embeddings represent one of the most sophisticated components of ABCDeez Core's learning model. They capture the latent spatial relationships between items in a knowledge domain, allowing the system to understand not just what learners know, but how they organize and relate different concepts in mental space.

## Conceptual Foundation

### What are Node Embeddings?

Node embeddings are continuous vector representations that encode the position of knowledge items in a learned latent space. Unlike fixed orderings, these embeddings:

1. **Adapt to individual learners**: Each person may organize concepts differently
2. **Capture similarity**: Items close in embedding space are psychologically similar
3. **Reflect learning progress**: Embeddings become more accurate as evidence accumulates
4. **Enable prediction**: Spatial relationships predict task difficulty and performance patterns

### Theoretical Background

The embedding model is based on several key psychological principles:

- **Spatial representation of knowledge**: Humans often organize abstract concepts spatially
- **Individual differences**: People vary in how they structure knowledge domains
- **Multidimensional scaling**: Psychological distances can be represented in Euclidean space
- **Uncertainty quantification**: Confidence in spatial positions varies with experience

## Core Data Structures

### Latent Node Embedding

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentNodeEmbedding {
    pub node_id: String,
    pub position: f64,           // 1D position in latent space
    pub uncertainty: f64,        // Standard deviation of position estimate
    pub last_updated: DateTime<Utc>,
    pub update_count: usize,     // Number of times updated
    pub evidence_strength: f64,  // How much evidence supports this position
}

impl LatentNodeEmbedding {
    pub fn new(node_id: String, initial_position: f64, initial_uncertainty: f64) -> Self {
        Self {
            node_id,
            position: initial_position,
            uncertainty: initial_uncertainty,
            last_updated: Utc::now(),
            update_count: 0,
            evidence_strength: 0.1,
        }
    }
    
    pub fn confidence_interval(&self, z_score: f64) -> (f64, f64) {
        let margin = z_score * self.uncertainty;
        (self.position - margin, self.position + margin)
    }
    
    pub fn is_well_calibrated(&self) -> bool {
        // Well-calibrated if uncertainty is reasonable and we have sufficient evidence
        self.uncertainty < 2.0 && self.evidence_strength > 0.5 && self.update_count >= 5
    }
}
```

### Multi-Dimensional Embeddings

For complex domains, we support multi-dimensional embeddings:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiDimEmbedding {
    pub node_id: String,
    pub positions: Vec<f64>,          // Position in each dimension
    pub uncertainties: Vec<f64>,      // Uncertainty in each dimension
    pub correlation_matrix: Vec<Vec<f64>>, // Cross-dimensional correlations
    pub dimension_labels: Vec<String>, // Semantic meaning of dimensions
}

impl MultiDimEmbedding {
    pub fn distance_to(&self, other: &MultiDimEmbedding) -> f64 {
        self.positions.iter()
            .zip(other.positions.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
    
    pub fn uncertainty_weighted_distance(&self, other: &MultiDimEmbedding) -> f64 {
        let mut weighted_distance = 0.0;
        
        for i in 0..self.positions.len().min(other.positions.len()) {
            let position_diff = (self.positions[i] - other.positions[i]).abs();
            let combined_uncertainty = (self.uncertainties[i] + other.uncertainties[i]) / 2.0;
            
            // Weight differences more heavily when we're more certain
            let weight = 1.0 / (1.0 + combined_uncertainty);
            weighted_distance += weight * position_diff.powi(2);
        }
        
        weighted_distance.sqrt()
    }
}
```

## Embedding Learning Algorithms

### Maximum Likelihood Estimation

The core algorithm updates embeddings based on observed performance patterns:

```rust
pub struct EmbeddingLearner {
    pub learning_rate: f64,
    pub uncertainty_decay: f64,
    pub min_uncertainty: f64,
    pub regularization_strength: f64,
    pub identifiability_constraints: bool,
}

impl EmbeddingLearner {
    pub fn new() -> Self {
        Self {
            learning_rate: 0.1,
            uncertainty_decay: 0.95,
            min_uncertainty: 0.05,
            regularization_strength: 0.01,
            identifiability_constraints: true,
        }
    }
    
    pub fn update_from_task_performance(
        &self,
        embeddings: &mut HashMap<String, LatentNodeEmbedding>,
        task_response: &TaskResponse,
    ) -> Result<(), String> {
        match &task_response.task.task_type {
            TaskType::PairwiseOrder { a, b } => {
                self.update_from_pairwise_comparison(embeddings, a, b, task_response)
            }
            TaskType::KJump { start, k } => {
                self.update_from_k_jump(embeddings, start, *k, task_response)
            }
            TaskType::Successor { item } => {
                self.update_from_successor(embeddings, item, task_response)
            }
            _ => Ok(()), // Other task types don't directly inform embeddings
        }
    }
    
    fn update_from_pairwise_comparison(
        &self,
        embeddings: &mut HashMap<String, LatentNodeEmbedding>,
        item_a: &str,
        item_b: &str,
        response: &TaskResponse,
    ) -> Result<(), String> {
        let embedding_a = embeddings.get_mut(item_a).ok_or("Item A not found")?;
        let embedding_b = embeddings.get_mut(item_b).ok_or("Item B not found")?;
        
        // Calculate current predicted order
        let predicted_a_before_b = embedding_a.position < embedding_b.position;
        let actual_correct = response.correct;
        
        // If prediction was wrong, adjust positions
        if predicted_a_before_b != actual_correct {
            let position_diff = (embedding_b.position - embedding_a.position).abs();
            let adjustment = self.learning_rate * (1.0 / (1.0 + position_diff));
            
            if actual_correct {
                // A should come before B - move A left, B right
                embedding_a.position -= adjustment;
                embedding_b.position += adjustment;
            } else {
                // B should come before A - move A right, B left  
                embedding_a.position += adjustment;
                embedding_b.position -= adjustment;
            }
        }
        
        // Update uncertainties and metadata
        let uncertainty_reduction = self.calculate_uncertainty_reduction(response);
        embedding_a.uncertainty *= self.uncertainty_decay;
        embedding_b.uncertainty *= self.uncertainty_decay;
        embedding_a.uncertainty = embedding_a.uncertainty.max(self.min_uncertainty);
        embedding_b.uncertainty = embedding_b.uncertainty.max(self.min_uncertainty);
        
        embedding_a.update_count += 1;
        embedding_b.update_count += 1;
        embedding_a.last_updated = Utc::now();
        embedding_b.last_updated = Utc::now();
        
        Ok(())
    }
    
    fn update_from_k_jump(
        &self,
        embeddings: &mut HashMap<String, LatentNodeEmbedding>,
        start_item: &str,
        k: i32,
        response: &TaskResponse,
    ) -> Result<(), String> {
        let start_embedding = embeddings.get(start_item).ok_or("Start item not found")?;
        
        // Extract target item from response
        if let Some(target_item) = self.extract_target_from_response(response) {
            let target_embedding = embeddings.get_mut(&target_item).ok_or("Target not found")?;
            
            // Expected position based on k-jump
            let expected_position = start_embedding.position + k as f64;
            let position_error = target_embedding.position - expected_position;
            
            // Update target position towards expected position
            let adjustment = self.learning_rate * position_error;
            target_embedding.position -= adjustment;
            
            // Update uncertainty based on accuracy
            let uncertainty_reduction = if response.correct { 0.1 } else { -0.05 };
            target_embedding.uncertainty *= (1.0 - uncertainty_reduction).max(0.5);
            target_embedding.uncertainty = target_embedding.uncertainty.max(self.min_uncertainty);
            
            target_embedding.update_count += 1;
            target_embedding.last_updated = Utc::now();
        }
        
        Ok(())
    }
    
    fn calculate_uncertainty_reduction(&self, response: &TaskResponse) -> f64 {
        // More evidence from correct responses, especially fast ones
        let base_reduction = if response.correct { 0.1 } else { 0.05 };
        let speed_bonus = if response.response_time_ms < 2000.0 { 0.05 } else { 0.0 };
        base_reduction + speed_bonus
    }
}
```

### Identifiability Constraints

To prevent gauge freedom (arbitrary rotations/translations), we apply constraints:

```rust
impl EmbeddingLearner {
    pub fn apply_identifiability_constraints(
        &self,
        embeddings: &mut HashMap<String, LatentNodeEmbedding>,
    ) {
        if !self.identifiability_constraints {
            return;
        }
        
        let positions: Vec<f64> = embeddings.values().map(|e| e.position).collect();
        if positions.is_empty() {
            return;
        }
        
        // Fix first and last items to prevent rotation/scaling
        let min_pos = positions.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_pos = positions.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        // Find items at extremes
        let min_item = embeddings
            .iter()
            .min_by(|a, b| a.1.position.partial_cmp(&b.1.position).unwrap())
            .map(|(id, _)| id.clone());
            
        let max_item = embeddings
            .iter()
            .max_by(|a, b| a.1.position.partial_cmp(&b.1.position).unwrap())
            .map(|(id, _)| id.clone());
        
        // Anchor extremes
        if let Some(min_id) = min_item {
            if let Some(embedding) = embeddings.get_mut(&min_id) {
                embedding.position = 0.0;
                embedding.uncertainty = self.min_uncertainty;
            }
        }
        
        if let Some(max_id) = max_item {
            if let Some(embedding) = embeddings.get_mut(&max_id) {
                let n = embeddings.len() as f64;
                embedding.position = n - 1.0;
                embedding.uncertainty = self.min_uncertainty;
            }
        }
        
        // Center all embeddings to prevent drift
        self.center_embeddings(embeddings);
    }
    
    fn center_embeddings(&self, embeddings: &mut HashMap<String, LatentNodeEmbedding>) {
        let mean_position: f64 = embeddings.values()
            .map(|e| e.position)
            .sum::<f64>() / embeddings.len() as f64;
        
        let target_mean = (embeddings.len() as f64 - 1.0) / 2.0; // Center at middle
        let shift = target_mean - mean_position;
        
        for embedding in embeddings.values_mut() {
            embedding.position += shift;
        }
    }
}
```

## Advanced Embedding Techniques

### Bayesian Embedding Updates

For more principled uncertainty handling:

```rust
pub struct BayesianEmbeddingLearner {
    pub prior_mean: f64,
    pub prior_variance: f64,
    pub observation_noise: f64,
    pub dimension_count: usize,
}

impl BayesianEmbeddingLearner {
    pub fn bayesian_update(
        &self,
        embedding: &mut LatentNodeEmbedding,
        observed_position: f64,
        observation_weight: f64,
    ) {
        // Prior: N(μ₀, σ₀²)
        let prior_mean = embedding.position;
        let prior_variance = embedding.uncertainty.powi(2);
        
        // Likelihood: N(observed_position, observation_noise²)
        let likelihood_variance = self.observation_noise.powi(2) / observation_weight;
        
        // Posterior: N(μ₁, σ₁²)
        let precision_prior = 1.0 / prior_variance;
        let precision_likelihood = 1.0 / likelihood_variance;
        let precision_posterior = precision_prior + precision_likelihood;
        
        let posterior_variance = 1.0 / precision_posterior;
        let posterior_mean = posterior_variance * (
            precision_prior * prior_mean + precision_likelihood * observed_position
        );
        
        // Update embedding
        embedding.position = posterior_mean;
        embedding.uncertainty = posterior_variance.sqrt();
        embedding.evidence_strength += observation_weight;
        embedding.update_count += 1;
        embedding.last_updated = Utc::now();
    }
    
    pub fn predict_position_distribution(
        &self,
        embedding: &LatentNodeEmbedding,
    ) -> NormalDistribution {
        NormalDistribution {
            mean: embedding.position,
            std_dev: embedding.uncertainty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NormalDistribution {
    pub mean: f64,
    pub std_dev: f64,
}

impl NormalDistribution {
    pub fn sample(&self) -> f64 {
        // Box-Muller transformation for normal sampling
        let u1: f64 = rand::random();
        let u2: f64 = rand::random();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        self.mean + self.std_dev * z0
    }
    
    pub fn probability_density(&self, x: f64) -> f64 {
        let coefficient = 1.0 / (self.std_dev * (2.0 * std::f64::consts::PI).sqrt());
        let exponent = -0.5 * ((x - self.mean) / self.std_dev).powi(2);
        coefficient * exponent.exp()
    }
}
```

### Embedding Regularization

Prevent overfitting and maintain reasonable structure:

```rust
pub struct EmbeddingRegularizer {
    pub l2_strength: f64,
    pub smoothness_strength: f64,
    pub ordering_strength: f64,
}

impl EmbeddingRegularizer {
    pub fn apply_regularization(
        &self,
        embeddings: &mut HashMap<String, LatentNodeEmbedding>,
        topology: &Topology,
    ) {
        // L2 regularization toward expected positions
        self.apply_l2_regularization(embeddings, topology);
        
        // Smoothness regularization (neighboring items should be close)
        self.apply_smoothness_regularization(embeddings, topology);
        
        // Ordering regularization (preserve known orderings)
        self.apply_ordering_regularization(embeddings, topology);
    }
    
    fn apply_l2_regularization(
        &self,
        embeddings: &mut HashMap<String, LatentNodeEmbedding>,
        topology: &Topology,
    ) {
        for (node_id, embedding) in embeddings.iter_mut() {
            // Calculate expected position based on topology
            let expected_position = self.calculate_expected_position(node_id, topology);
            
            // Pull toward expected position
            let regularization_force = self.l2_strength * (expected_position - embedding.position);
            embedding.position += regularization_force;
        }
    }
    
    fn apply_smoothness_regularization(
        &self,
        embeddings: &mut HashMap<String, LatentNodeEmbedding>,
        topology: &Topology,
    ) {
        let mut position_updates: HashMap<String, f64> = HashMap::new();
        
        for embedding in embeddings.values() {
            let neighbors = topology.get_neighbors(&embedding.node_id);
            if neighbors.is_empty() {
                continue;
            }
            
            // Calculate mean position of neighbors
            let neighbor_positions: Vec<f64> = neighbors
                .iter()
                .filter_map(|neighbor_id| embeddings.get(neighbor_id))
                .map(|neighbor| neighbor.position)
                .collect();
            
            if !neighbor_positions.is_empty() {
                let mean_neighbor_position = neighbor_positions.iter().sum::<f64>() 
                    / neighbor_positions.len() as f64;
                
                // Pull toward neighbor average
                let smoothing_force = self.smoothness_strength * 
                    (mean_neighbor_position - embedding.position);
                
                position_updates.insert(embedding.node_id.clone(), smoothing_force);
            }
        }
        
        // Apply updates
        for (node_id, update) in position_updates {
            if let Some(embedding) = embeddings.get_mut(&node_id) {
                embedding.position += update;
            }
        }
    }
    
    fn calculate_expected_position(&self, node_id: &str, topology: &Topology) -> f64 {
        // For linear topologies, use index position
        if let Some(index) = topology.get_node_index(node_id) {
            index as f64
        } else {
            0.0 // Fallback
        }
    }
}
```

## Embedding-Based Predictions

### Distance-Based Difficulty Prediction

```rust
pub struct EmbeddingPredictor {
    pub distance_coefficient: f64,
    pub uncertainty_penalty: f64,
    pub base_difficulty: f64,
}

impl EmbeddingPredictor {
    pub fn predict_task_difficulty(
        &self,
        task: &Task,
        embeddings: &HashMap<String, LatentNodeEmbedding>,
    ) -> f64 {
        match &task.task_type {
            TaskType::PairwiseOrder { a, b } => {
                self.predict_pairwise_difficulty(a, b, embeddings)
            }
            TaskType::KJump { start, k } => {
                self.predict_k_jump_difficulty(start, *k, embeddings)
            }
            TaskType::Successor { item } => {
                self.predict_successor_difficulty(item, embeddings)
            }
            _ => self.base_difficulty,
        }
    }
    
    fn predict_pairwise_difficulty(
        &self,
        item_a: &str,
        item_b: &str,
        embeddings: &HashMap<String, LatentNodeEmbedding>,
    ) -> f64 {
        let embedding_a = embeddings.get(item_a);
        let embedding_b = embeddings.get(item_b);
        
        match (embedding_a, embedding_b) {
            (Some(emb_a), Some(emb_b)) => {
                // Distance-based difficulty
                let distance = (emb_a.position - emb_b.position).abs();
                let distance_difficulty = self.distance_coefficient / (1.0 + distance);
                
                // Uncertainty penalty
                let uncertainty_penalty = self.uncertainty_penalty * 
                    (emb_a.uncertainty + emb_b.uncertainty);
                
                (self.base_difficulty + distance_difficulty + uncertainty_penalty)
                    .max(0.1)
                    .min(1.0)
            }
            _ => self.base_difficulty,
        }
    }
    
    fn predict_k_jump_difficulty(
        &self,
        start_item: &str,
        k: i32,
        embeddings: &HashMap<String, LatentNodeEmbedding>,
    ) -> f64 {
        let base_k_difficulty = self.base_difficulty + (k.abs() as f64 * 0.1);
        
        if let Some(start_embedding) = embeddings.get(start_item) {
            let uncertainty_penalty = self.uncertainty_penalty * start_embedding.uncertainty;
            (base_k_difficulty + uncertainty_penalty).max(0.1).min(1.0)
        } else {
            base_k_difficulty
        }
    }
    
    fn predict_successor_difficulty(
        &self,
        item: &str,
        embeddings: &HashMap<String, LatentNodeEmbedding>,
    ) -> f64 {
        if let Some(embedding) = embeddings.get(item) {
            let uncertainty_penalty = self.uncertainty_penalty * embedding.uncertainty;
            (self.base_difficulty + uncertainty_penalty).max(0.1).min(1.0)
        } else {
            self.base_difficulty
        }
    }
}
```

### Response Time Prediction

```rust
impl EmbeddingPredictor {
    pub fn predict_response_time(
        &self,
        task: &Task,
        embeddings: &HashMap<String, LatentNodeEmbedding>,
        learner_proficiency: f64,
    ) -> f64 {
        let base_rt = 1000.0; // Base response time in ms
        
        let difficulty = self.predict_task_difficulty(task, embeddings);
        let difficulty_penalty = 500.0 * difficulty;
        
        let proficiency_bonus = 300.0 * learner_proficiency.sigmoid();
        
        // Uncertainty increases response time (less confident = slower)
        let uncertainty_penalty = match &task.task_type {
            TaskType::PairwiseOrder { a, b } => {
                let uncertainty_a = embeddings.get(a).map_or(1.0, |e| e.uncertainty);
                let uncertainty_b = embeddings.get(b).map_or(1.0, |e| e.uncertainty);
                200.0 * (uncertainty_a + uncertainty_b)
            }
            _ => 200.0, // Default uncertainty penalty
        };
        
        (base_rt + difficulty_penalty + uncertainty_penalty - proficiency_bonus)
            .max(300.0) // Minimum response time
    }
}

trait SigmoidFunction {
    fn sigmoid(self) -> f64;
}

impl SigmoidFunction for f64 {
    fn sigmoid(self) -> f64 {
        1.0 / (1.0 + (-self).exp())
    }
}
```

## Visualization and Analysis

### Embedding Visualization

```rust
pub struct EmbeddingVisualizer {
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub margin: f64,
}

impl EmbeddingVisualizer {
    pub fn generate_1d_visualization(
        &self,
        embeddings: &HashMap<String, LatentNodeEmbedding>,
    ) -> EmbeddingVisualization {
        let positions: Vec<f64> = embeddings.values().map(|e| e.position).collect();
        let min_pos = positions.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_pos = positions.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        let mut items = Vec::new();
        
        for (node_id, embedding) in embeddings {
            let normalized_x = if max_pos > min_pos {
                (embedding.position - min_pos) / (max_pos - min_pos)
            } else {
                0.5
            };
            
            let screen_x = self.margin + normalized_x * (self.canvas_width - 2.0 * self.margin);
            
            items.push(VisualizationItem {
                node_id: node_id.clone(),
                x: screen_x,
                y: self.canvas_height / 2.0,
                uncertainty_radius: embedding.uncertainty * 10.0,
                confidence_color: self.uncertainty_to_color(embedding.uncertainty),
            });
        }
        
        EmbeddingVisualization {
            items,
            width: self.canvas_width,
            height: self.canvas_height,
            min_position: min_pos,
            max_position: max_pos,
        }
    }
    
    fn uncertainty_to_color(&self, uncertainty: f64) -> Color {
        // Red = high uncertainty, Green = low uncertainty
        let normalized_uncertainty = (uncertainty / 2.0).min(1.0); // Assume max uncertainty of 2.0
        Color {
            r: (255.0 * normalized_uncertainty) as u8,
            g: (255.0 * (1.0 - normalized_uncertainty)) as u8,
            b: 0,
            a: 255,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddingVisualization {
    pub items: Vec<VisualizationItem>,
    pub width: f64,
    pub height: f64,
    pub min_position: f64,
    pub max_position: f64,
}

#[derive(Debug, Clone)]
pub struct VisualizationItem {
    pub node_id: String,
    pub x: f64,
    pub y: f64,
    pub uncertainty_radius: f64,
    pub confidence_color: Color,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
```

## Performance Optimization

### Efficient Embedding Updates

```rust
pub struct OptimizedEmbeddingLearner {
    pub embeddings: HashMap<String, LatentNodeEmbedding>,
    pub update_buffer: Vec<EmbeddingUpdate>,
    pub batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct EmbeddingUpdate {
    pub node_id: String,
    pub position_delta: f64,
    pub uncertainty_delta: f64,
    pub weight: f64,
}

impl OptimizedEmbeddingLearner {
    pub fn queue_update(&mut self, update: EmbeddingUpdate) {
        self.update_buffer.push(update);
        
        if self.update_buffer.len() >= self.batch_size {
            self.flush_updates();
        }
    }
    
    pub fn flush_updates(&mut self) {
        // Group updates by node_id
        let mut grouped_updates: HashMap<String, Vec<EmbeddingUpdate>> = HashMap::new();
        
        for update in self.update_buffer.drain(..) {
            grouped_updates.entry(update.node_id.clone()).or_default().push(update);
        }
        
        // Apply batched updates
        for (node_id, updates) in grouped_updates {
            if let Some(embedding) = self.embeddings.get_mut(&node_id) {
                // Weighted average of position deltas
                let total_weight: f64 = updates.iter().map(|u| u.weight).sum();
                if total_weight > 0.0 {
                    let weighted_position_delta: f64 = updates
                        .iter()
                        .map(|u| u.position_delta * u.weight)
                        .sum::<f64>() / total_weight;
                    
                    let weighted_uncertainty_delta: f64 = updates
                        .iter()
                        .map(|u| u.uncertainty_delta * u.weight)
                        .sum::<f64>() / total_weight;
                    
                    embedding.position += weighted_position_delta;
                    embedding.uncertainty += weighted_uncertainty_delta;
                    embedding.uncertainty = embedding.uncertainty.max(0.01).min(5.0);
                    embedding.update_count += updates.len();
                    embedding.last_updated = Utc::now();
                }
            }
        }
    }
}
```

## Testing and Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_embedding_update_from_pairwise_comparison() {
        let mut embeddings = HashMap::new();
        embeddings.insert("A".to_string(), LatentNodeEmbedding::new("A".to_string(), 1.0, 0.5));
        embeddings.insert("B".to_string(), LatentNodeEmbedding::new("B".to_string(), 3.0, 0.5));
        
        let learner = EmbeddingLearner::new();
        let task = Task {
            task_type: TaskType::PairwiseOrder { 
                a: "A".to_string(), 
                b: "B".to_string() 
            },
            // ... other fields
        };
        
        let response = TaskResponse {
            task,
            correct: false, // A does not come before B (should be true)
            response_time_ms: 1500.0,
            // ... other fields
        };
        
        let initial_a_pos = embeddings["A"].position;
        let initial_b_pos = embeddings["B"].position;
        
        learner.update_from_task_performance(&mut embeddings, &response).unwrap();
        
        // A should move right, B should move left (since the order was wrong)
        assert!(embeddings["A"].position > initial_a_pos);
        assert!(embeddings["B"].position < initial_b_pos);
    }
    
    #[test]
    fn test_uncertainty_reduction() {
        let mut embedding = LatentNodeEmbedding::new("test".to_string(), 0.0, 1.0);
        let initial_uncertainty = embedding.uncertainty;
        
        let bayesian_learner = BayesianEmbeddingLearner {
            prior_mean: 0.0,
            prior_variance: 1.0,
            observation_noise: 0.1,
            dimension_count: 1,
        };
        
        // Multiple observations should reduce uncertainty
        for _ in 0..10 {
            bayesian_learner.bayesian_update(&mut embedding, 0.0, 1.0);
        }
        
        assert!(embedding.uncertainty < initial_uncertainty);
        assert!(embedding.uncertainty > 0.0); // Should not go to zero
    }
    
    #[test]
    fn test_identifiability_constraints() {
        let mut embeddings = HashMap::new();
        embeddings.insert("A".to_string(), LatentNodeEmbedding::new("A".to_string(), 10.0, 0.5));
        embeddings.insert("B".to_string(), LatentNodeEmbedding::new("B".to_string(), 20.0, 0.5));
        embeddings.insert("C".to_string(), LatentNodeEmbedding::new("C".to_string(), 30.0, 0.5));
        
        let learner = EmbeddingLearner::new();
        learner.apply_identifiability_constraints(&mut embeddings);
        
        // First item should be at 0, last at n-1
        let positions: Vec<f64> = embeddings.values().map(|e| e.position).collect();
        let min_pos = positions.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_pos = positions.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        assert!((min_pos - 0.0).abs() < 0.01);
        assert!((max_pos - 2.0).abs() < 0.01); // 3 items: 0, 1, 2
    }
    
    #[test]
    fn test_distance_based_difficulty_prediction() {
        let mut embeddings = HashMap::new();
        embeddings.insert("A".to_string(), LatentNodeEmbedding::new("A".to_string(), 0.0, 0.1));
        embeddings.insert("B".to_string(), LatentNodeEmbedding::new("B".to_string(), 1.0, 0.1));
        embeddings.insert("C".to_string(), LatentNodeEmbedding::new("C".to_string(), 5.0, 0.1));
        
        let predictor = EmbeddingPredictor {
            distance_coefficient: 1.0,
            uncertainty_penalty: 0.1,
            base_difficulty: 0.3,
        };
        
        let task_ab = Task {
            task_type: TaskType::PairwiseOrder { 
                a: "A".to_string(), 
                b: "B".to_string() 
            },
            // ... other fields
        };
        
        let task_ac = Task {
            task_type: TaskType::PairwiseOrder { 
                a: "A".to_string(), 
                b: "C".to_string() 
            },
            // ... other fields
        };
        
        let difficulty_ab = predictor.predict_task_difficulty(&task_ab, &embeddings);
        let difficulty_ac = predictor.predict_task_difficulty(&task_ac, &embeddings);
        
        // Closer items (A-B) should be harder than distant items (A-C)
        assert!(difficulty_ab > difficulty_ac);
    }
}
```

## Best Practices

1. **Initialize sensibly**: Start with reasonable positions based on domain knowledge
2. **Update incrementally**: Process one response at a time to avoid instability
3. **Apply constraints**: Use identifiability constraints to prevent gauge freedom
4. **Regularize appropriately**: Balance fitting data with maintaining structure
5. **Monitor convergence**: Track embedding stability over time
6. **Validate predictions**: Test embedding-based predictions against held-out data

## Common Pitfalls

- **Gauge freedom**: Without constraints, embeddings can rotate/translate arbitrarily
- **Overfitting**: Too many updates can make embeddings overspecific to individual responses
- **Initialization sensitivity**: Poor initial positions can lead to local minima
- **Uncertainty underestimation**: Failing to account for uncertainty in predictions
- **Scale inconsistency**: Different learners may use different embedding scales

## Applications

- **Adaptive testing**: Select items based on embedding distances
- **Knowledge mapping**: Visualize learner's conceptual organization
- **Personalized learning**: Tailor content to individual knowledge structures
- **Error analysis**: Understand patterns in confusable items
- **Curriculum design**: Optimize learning sequences based on embedding relationships

## Next Steps

- Learn about [Operation Proficiency](./proficiency.md) for skill development modeling
- Explore [Memory Dynamics](./memory.md) for forgetting and retention patterns  
- Understand [Chunk Detection](./chunking.md) for structural knowledge discovery
- See [Bayesian Updates](../bayesian/updates.md) for uncertainty quantification