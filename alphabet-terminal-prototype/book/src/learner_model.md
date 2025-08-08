# Learner Model

The learner model represents an individual's knowledge state, learning history, and cognitive characteristics. It integrates multiple sources of information to build a comprehensive picture of what someone knows and how they think.

## Core Structure

```rust
pub struct LearnerModel {
    pub learner_id: String,
    
    // Knowledge representations
    pub node_embeddings: HashMap<String, LatentNodeEmbedding>,
    pub operation_proficiencies: HashMap<String, OperationProficiency>,
    pub memory_strengths: HashMap<String, MemoryStrength>,
    
    // Error patterns
    pub confusability_matrix: HashMap<(String, String), f64>,
    
    // Cognitive characteristics  
    pub response_time_distribution: ExGaussianParameters,
    pub strategy: StrategyType,
    
    // Learning dynamics
    pub learning_rate: f64,
    pub forgetting_rate: f64,
}
```

## Node Embeddings

Each node has a learned position in latent space:

```rust
#[derive(Debug, Clone)]
pub struct LatentNodeEmbedding {
    pub node_id: String,
    pub position: f64,      // 1D position (can extend to higher dimensions)
    pub uncertainty: f64,   // Confidence in position estimate
}

impl LearnerModel {
    pub fn update_node_embedding(&mut self, node_id: &str, observation: f64) {
        if let Some(embedding) = self.node_embeddings.get_mut(node_id) {
            // Exponential moving average update
            let alpha = 0.1 / (1.0 + embedding.uncertainty);
            embedding.position = (1.0 - alpha) * embedding.position + alpha * observation;
            
            // Reduce uncertainty with each observation
            embedding.uncertainty *= 0.95;
        }
    }
}
```

## Operation Proficiency

Different cognitive operations have different difficulty:

```rust
#[derive(Debug, Clone)]
pub enum OperationType {
    Successor,              // "What comes after X?"
    Predecessor,            // "What comes before X?"
    PairwiseOrder,         // "Does A come before B?"
    KJump(i32),            // "What is K steps after X?"
    Segment(usize, bool),  // "List N items starting from X"
    Index,                 // "What is the Nth item?"
}

#[derive(Debug, Clone)]
pub struct OperationProficiency {
    pub operation: OperationType,
    pub theta: f64,         // Logit-scale proficiency
    pub practice_count: usize,
}
```

### Proficiency Updates

Learning follows a logistic growth curve:

```rust
impl LearnerModel {
    pub fn update_operation_proficiency(
        &mut self, 
        operation: &OperationType, 
        success: bool
    ) {
        let key = format!("{:?}", operation);
        if let Some(prof) = self.operation_proficiencies.get_mut(&key) {
            prof.practice_count += 1;
            
            let learning_rate = 0.1 / (1.0 + prof.practice_count as f64 * 0.1);
            
            if success {
                // Increase proficiency (but with diminishing returns)
                prof.theta += learning_rate * (1.0 - sigmoid(prof.theta));
            } else {
                // Decrease proficiency (but not below floor)
                prof.theta -= learning_rate * sigmoid(prof.theta);
            }
        }
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}
```

## Memory Strength

Memory for individual items follows forgetting curves:

```rust
#[derive(Debug, Clone)]
pub struct MemoryStrength {
    pub node_id: String,
    pub strength: f64,              // Current memory strength [0, 1]
    pub last_practice: DateTime<Utc>,
}

impl LearnerModel {
    pub fn update_memory_strength(&mut self, node_id: &str, correct: bool) {
        let key = self.normalize_node_id(node_id);
        
        if let Some(mem) = self.memory_strengths.get_mut(&key) {
            let now = Utc::now();
            let time_since = now.signed_duration_since(mem.last_practice);
            let hours_since = time_since.num_hours() as f64;
            
            // Apply forgetting curve
            let decay_rate = self.calculate_decay_rate(mem.strength);
            let decayed_strength = mem.strength * (-decay_rate * hours_since).exp();
            
            // Update based on response
            if correct {
                mem.strength = (decayed_strength + 0.2).min(1.0);
            } else {
                mem.strength = (decayed_strength - 0.1).max(0.0);
            }
            
            mem.last_practice = now;
        }
    }
    
    fn calculate_decay_rate(&self, current_strength: f64) -> f64 {
        // Stronger memories decay slower
        0.1 * (2.0 - current_strength)
    }
}
```

## Confusability Matrix

Tracks systematic errors between items:

```rust
impl LearnerModel {
    pub fn update_confusability(&mut self, presented: &str, responded: &str) {
        if presented != responded {
            let key = (presented.to_string(), responded.to_string());
            *self.confusability_matrix.entry(key).or_insert(0.0) += 1.0;
            
            // Normalize by total errors
            let total_errors: f64 = self.confusability_matrix.values().sum();
            for value in self.confusability_matrix.values_mut() {
                *value /= total_errors;
            }
        }
    }
    
    pub fn predict_likely_confusion(&self, item: &str) -> Vec<(String, f64)> {
        let mut confusions = Vec::new();
        
        for ((from, to), prob) in &self.confusability_matrix {
            if from == item {
                confusions.push((to.clone(), *prob));
            }
        }
        
        confusions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        confusions
    }
}
```

## Strategy Detection

The model infers cognitive strategy from response patterns:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StrategyType {
    SerialScan,     // Sequential search through list
    DirectAccess,   // Direct retrieval from memory
    Hybrid,         // Mix of strategies
}

impl LearnerModel {
    pub fn detect_strategy(&mut self, response_times: &[f64], distances: &[usize]) {
        // Compute correlation between RT and distance
        let correlation = calculate_correlation(response_times, distances);
        
        self.strategy = if correlation > 0.8 {
            StrategyType::SerialScan  // Strong linear relationship
        } else if correlation < 0.3 {
            StrategyType::DirectAccess // No relationship
        } else {
            StrategyType::Hybrid       // Mixed evidence
        };
        
        // Update RT distribution parameters based on strategy
        self.update_rt_distribution(response_times);
    }
    
    fn update_rt_distribution(&mut self, response_times: &[f64]) {
        let model = ExGaussianModel::fit(response_times);
        self.response_time_distribution = model.params;
    }
}
```

## Performance Prediction

The model predicts future performance:

```rust
impl LearnerModel {
    pub fn predict_performance(&self, task: &Task) -> PerformancePrediction {
        // Combine multiple factors
        let operation_factor = self.get_operation_proficiency(&task.operation);
        let memory_factor = self.get_memory_strength(&task.get_involved_nodes());
        let strategy_factor = self.get_strategy_efficiency(&task);
        
        // Weighted combination
        let p_correct = 
            0.4 * sigmoid(operation_factor) +
            0.4 * memory_factor +
            0.2 * strategy_factor;
        
        // Predict response time
        let base_rt = self.response_time_distribution.mu;
        let difficulty_multiplier = 1.0 + task.difficulty;
        let predicted_rt = base_rt * difficulty_multiplier;
        
        PerformancePrediction {
            probability_correct: p_correct,
            expected_rt: predicted_rt,
            confidence_interval: self.compute_confidence_interval(p_correct),
        }
    }
}
```

## Learning Dynamics

The model captures how learning changes over time:

```rust
impl LearnerModel {
    pub fn compute_learning_curve(&self) -> LearningCurve {
        let mut curve = Vec::new();
        
        // Reconstruct learning trajectory
        for (i, prof) in self.operation_proficiencies.values().enumerate() {
            for t in 0..prof.practice_count {
                // Approximate proficiency at time t
                let theta_t = self.reconstruct_proficiency_at_time(prof, t);
                curve.push((t, sigmoid(theta_t)));
            }
        }
        
        // Fit parametric model
        let model = self.fit_learning_model(&curve);
        
        LearningCurve {
            data_points: curve,
            fitted_model: model,
            plateau_point: self.find_plateau(&curve),
        }
    }
    
    fn fit_learning_model(&self, data: &[(usize, f64)]) -> LearningModel {
        // Power law of practice: P(n) = a * n^(-b) + c
        // Or exponential: P(n) = a * (1 - exp(-b*n))
        // Implementation depends on domain
        LearningModel::PowerLaw { a: 0.3, b: 0.5, c: 0.7 }
    }
}
```

## Individual Differences

The model captures individual variation:

```rust
impl LearnerModel {
    pub fn estimate_cognitive_parameters(&mut self) {
        // Working memory capacity (from segment task performance)
        self.working_memory_capacity = self.estimate_wm_capacity();
        
        // Processing speed (from baseline RTs)
        self.processing_speed = self.estimate_processing_speed();
        
        // Learning style (from strategy preferences)
        self.learning_style = self.classify_learning_style();
    }
    
    fn estimate_wm_capacity(&self) -> f64 {
        // Based on performance degradation with segment length
        let segment_performances = self.get_segment_task_performance();
        
        // Find point where performance drops below threshold
        for (length, performance) in segment_performances {
            if performance < 0.75 {
                return length as f64 - 1.0;
            }
        }
        
        7.0 // Default to Miller's magic number
    }
}
```

## Metrics and Evaluation

The model provides comprehensive metrics:

```rust
#[derive(Debug, Clone)]
pub struct LearnerMetrics {
    pub total_trials: usize,
    pub overall_accuracy: f64,
    pub avg_response_time: f64,
    pub learning_rate: f64,
    pub current_mastery: f64,
    pub predicted_asymptote: f64,
}

impl LearnerModel {
    pub fn calculate_metrics(&self) -> LearnerMetrics {
        let total_practice: usize = self.operation_proficiencies
            .values()
            .map(|p| p.practice_count)
            .sum();
        
        let avg_proficiency: f64 = self.operation_proficiencies
            .values()
            .map(|p| sigmoid(p.theta))
            .sum::<f64>() / self.operation_proficiencies.len() as f64;
        
        let avg_memory: f64 = self.memory_strengths
            .values()
            .map(|m| m.strength)
            .sum::<f64>() / self.memory_strengths.len() as f64;
        
        LearnerMetrics {
            total_trials: total_practice,
            overall_accuracy: avg_proficiency,
            avg_response_time: self.response_time_distribution.mu,
            learning_rate: self.learning_rate,
            current_mastery: (avg_proficiency + avg_memory) / 2.0,
            predicted_asymptote: self.predict_asymptotic_performance(),
        }
    }
}
```

## Integration Example

Let's see how all components work together:

```rust
// Initialize learner
let mut learner = LearnerModel::new("student_001", &topology);

// Process a response
let response = ResponseData {
    task: Task {
        task_type: TaskType::Successor { item: "M".to_string() },
        operation: OperationType::Successor,
        // ...
    },
    correct: true,
    response_time: 1250.0,
};

// Update all relevant components
learner.update_operation_proficiency(&response.task.operation, response.correct);
learner.update_memory_strength("M", response.correct);
learner.update_memory_strength("N", response.correct);
learner.update_node_embedding("node_12", 12.8); // Refined position estimate
learner.update_response_time_distribution(response.response_time);

// Check for strategy shift
if learner.recent_responses.len() >= 10 {
    learner.detect_strategy(&learner.recent_rts, &learner.recent_distances);
}

// Get current metrics
let metrics = learner.calculate_metrics();
println!("Current mastery: {:.2}%", metrics.current_mastery * 100.0);
```

## Summary

The learner model provides:
- **Multi-faceted representation**: Embeddings, proficiencies, memory
- **Dynamic updates**: Learning from every response
- **Strategy detection**: Understanding how people think
- **Performance prediction**: Anticipating future behavior
- **Individual differences**: Capturing unique characteristics

This rich representation enables truly adaptive task selection, as explored in the next chapter.