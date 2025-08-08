# Memory and Forgetting

Memory dynamics are central to learning. The system models how memories form, strengthen, and decay over time, enabling optimal spacing and review scheduling.

## Memory Strength Model

```rust
#[derive(Debug, Clone)]
pub struct MemoryStrength {
    pub item_id: String,
    pub strength: f64,              // Current activation [0, 1]
    pub stability: f64,             // Decay rate parameter
    pub retrievability: f64,        // Probability of successful retrieval
    pub last_practice: DateTime<Utc>,
    pub practice_history: Vec<PracticeEvent>,
}

#[derive(Debug, Clone)]
pub struct PracticeEvent {
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub response_time: f64,
    pub practice_quality: f64,
}
```

## Forgetting Curves

### Ebbinghaus Model

The classic exponential decay:

```rust
impl MemoryStrength {
    pub fn ebbinghaus_retention(&self, time: Duration) -> f64 {
        let hours = time.num_hours() as f64;
        (-hours / self.stability).exp()
    }
    
    pub fn update_after_practice(&mut self, success: bool, quality: f64) {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.last_practice);
        
        // Compute current strength after decay
        let decayed = self.strength * self.ebbinghaus_retention(elapsed);
        
        if success {
            // Strengthen memory
            self.strength = (decayed + quality * 0.2).min(1.0);
            // Increase stability (slower forgetting)
            self.stability *= 1.0 + quality * 0.1;
        } else {
            // Weaken memory
            self.strength = (decayed - 0.1).max(0.0);
            // Decrease stability (faster forgetting)
            self.stability *= 0.9;
        }
        
        self.last_practice = now;
        self.practice_history.push(PracticeEvent {
            timestamp: now,
            success,
            response_time: 0.0, // Set externally
            practice_quality: quality,
        });
    }
}
```

### ACT-R Memory Model

Activation-based memory with power law decay:

```rust
pub struct ACTRMemory {
    pub base_activation: f64,
    pub decay_parameter: f64,  // Typically 0.5
    pub activation_noise: f64,
}

impl ACTRMemory {
    pub fn compute_activation(&self, item: &MemoryItem) -> f64 {
        // Base-level activation from practice history
        let base = self.base_level_activation(&item.practice_times);
        
        // Spreading activation from context
        let spreading = self.spreading_activation(&item.associations);
        
        // Partial matching
        let partial = self.partial_matching_penalty(&item.similarity);
        
        // Add noise
        let noise = sample_logistic(0.0, self.activation_noise);
        
        base + spreading - partial + noise
    }
    
    fn base_level_activation(&self, practice_times: &[f64]) -> f64 {
        let now = current_time();
        
        // B = ln(sum(t_i^-d))
        let sum: f64 = practice_times.iter()
            .map(|t| (now - t).powf(-self.decay_parameter))
            .sum();
        
        sum.ln()
    }
    
    pub fn retrieval_probability(&self, activation: f64, threshold: f64) -> f64 {
        // Logistic function
        let tau = 0.5; // Temperature parameter
        1.0 / (1.0 + ((threshold - activation) / tau).exp())
    }
    
    pub fn retrieval_time(&self, activation: f64) -> f64 {
        // Exponential relationship
        let F = 1.0; // Latency factor
        F * (-activation).exp()
    }
}
```

## Spacing Effects

### Optimal Spacing Algorithm

```rust
pub struct SpacingOptimizer {
    pub target_retention: f64,
    pub model: SpacingModel,
}

impl SpacingOptimizer {
    pub fn compute_optimal_spacing(&self, item: &MemoryItem) -> Duration {
        match self.model {
            SpacingModel::FixedRatio => {
                // Each interval is k times the previous
                let k = 2.5;
                let last_interval = item.last_interval();
                Duration::hours((last_interval.num_hours() as f64 * k) as i64)
            }
            
            SpacingModel::AdaptiveRatio => {
                // Adjust ratio based on performance
                let k = self.compute_adaptive_ratio(item);
                let last_interval = item.last_interval();
                Duration::hours((last_interval.num_hours() as f64 * k) as i64)
            }
            
            SpacingModel::TargetRetention => {
                // Solve for interval that yields target retention
                self.solve_for_target_retention(item)
            }
            
            SpacingModel::OptimalLearning => {
                // Maximize long-term retention
                self.optimize_long_term_retention(item)
            }
        }
    }
    
    fn solve_for_target_retention(&self, item: &MemoryItem) -> Duration {
        // Given R(t) = exp(-t/S), solve for t when R(t) = target
        let stability = item.memory_strength.stability;
        let hours = -stability * self.target_retention.ln();
        Duration::hours(hours as i64)
    }
    
    fn optimize_long_term_retention(&self, item: &MemoryItem) -> Duration {
        // Find spacing that maximizes area under retention curve
        let mut best_spacing = Duration::hours(24);
        let mut best_value = 0.0;
        
        for hours in [1, 2, 4, 8, 16, 24, 48, 96, 168] {
            let spacing = Duration::hours(hours);
            let value = self.evaluate_spacing_value(item, spacing);
            
            if value > best_value {
                best_value = value;
                best_spacing = spacing;
            }
        }
        
        best_spacing
    }
}
```

## Memory Consolidation

### Sleep-Dependent Consolidation

```rust
pub struct ConsolidationModel {
    pub sleep_cycles: Vec<SleepCycle>,
    pub consolidation_rate: f64,
}

impl ConsolidationModel {
    pub fn simulate_consolidation(&self, memories: &mut [MemoryItem]) {
        for cycle in &self.sleep_cycles {
            match cycle.stage {
                SleepStage::REM => {
                    // Consolidate procedural memories
                    self.consolidate_procedural(memories, cycle.duration);
                }
                SleepStage::SWS => {
                    // Consolidate declarative memories
                    self.consolidate_declarative(memories, cycle.duration);
                }
                _ => {}
            }
        }
    }
    
    fn consolidate_declarative(&self, memories: &mut [MemoryItem], duration: Duration) {
        for memory in memories {
            if memory.memory_type == MemoryType::Declarative {
                // Strengthen memory trace
                let boost = self.consolidation_rate * duration.num_hours() as f64;
                memory.strength *= 1.0 + boost;
                
                // Increase resistance to interference
                memory.interference_resistance += boost * 0.5;
                
                // Form associations
                self.strengthen_associations(memory);
            }
        }
    }
    
    fn strengthen_associations(&self, memory: &mut MemoryItem) {
        // Replay-based consolidation
        for association in &mut memory.associations {
            association.strength *= 1.0 + self.consolidation_rate;
        }
    }
}
```

## Interference Effects

### Proactive and Retroactive Interference

```rust
pub struct InterferenceModel {
    pub similarity_threshold: f64,
    pub interference_decay: f64,
}

impl InterferenceModel {
    pub fn compute_interference(&self, target: &MemoryItem, context: &[MemoryItem]) -> f64 {
        let mut proactive = 0.0;
        let mut retroactive = 0.0;
        
        for item in context {
            let similarity = self.compute_similarity(target, item);
            
            if similarity > self.similarity_threshold {
                let time_diff = target.timestamp - item.timestamp;
                
                if time_diff > 0 {
                    // Item learned before target (proactive)
                    proactive += similarity * item.strength * 
                                (-time_diff.num_hours() as f64 * self.interference_decay).exp();
                } else {
                    // Item learned after target (retroactive)
                    retroactive += similarity * item.strength * 
                                  (time_diff.num_hours() as f64 * self.interference_decay).exp();
                }
            }
        }
        
        proactive + retroactive
    }
    
    fn compute_similarity(&self, item1: &MemoryItem, item2: &MemoryItem) -> f64 {
        // Feature-based similarity
        let feature_overlap = item1.features.intersection(&item2.features).count();
        let total_features = item1.features.union(&item2.features).count();
        
        let feature_sim = feature_overlap as f64 / total_features as f64;
        
        // Contextual similarity
        let context_sim = self.context_similarity(&item1.context, &item2.context);
        
        0.7 * feature_sim + 0.3 * context_sim
    }
}
```

## Retrieval Practice

### Testing Effect

```rust
pub struct RetrievalPractice {
    pub difficulty_threshold: f64,
    pub feedback_delay: Duration,
}

impl RetrievalPractice {
    pub fn design_retrieval_practice(&self, item: &MemoryItem) -> PracticeTask {
        let current_strength = item.memory_strength.strength;
        
        // Desirable difficulty principle
        let optimal_difficulty = if current_strength < 0.3 {
            0.7  // Easy for weak memories
        } else if current_strength < 0.7 {
            0.5  // Medium difficulty
        } else {
            0.3  // Hard for strong memories
        };
        
        PracticeTask {
            task_type: self.select_task_type(optimal_difficulty),
            cue_strength: 1.0 - optimal_difficulty,
            feedback_timing: self.determine_feedback_timing(current_strength),
            elaboration_required: current_strength > 0.5,
        }
    }
    
    fn determine_feedback_timing(&self, strength: f64) -> FeedbackTiming {
        if strength < 0.3 {
            FeedbackTiming::Immediate  // Weak memories need immediate feedback
        } else if strength < 0.6 {
            FeedbackTiming::Delayed(Duration::seconds(5))
        } else {
            FeedbackTiming::Delayed(self.feedback_delay)  // Strong memories benefit from delay
        }
    }
    
    pub fn update_after_retrieval(&self, item: &mut MemoryItem, success: bool, effort: f64) {
        if success {
            // Successful retrieval strengthens memory more than restudy
            let boost = 0.3 * effort;  // Higher effort = greater benefit
            item.memory_strength.strength = (item.memory_strength.strength + boost).min(1.0);
            
            // Increase stability
            item.memory_strength.stability *= 1.0 + 0.2 * effort;
        } else {
            // Failed retrieval followed by feedback is still beneficial
            item.memory_strength.strength += 0.1;
        }
    }
}
```

## Memory Reconsolidation

```rust
pub struct Reconsolidation {
    pub vulnerability_window: Duration,
    pub update_threshold: f64,
}

impl Reconsolidation {
    pub fn trigger_reconsolidation(&mut self, memory: &mut MemoryItem, new_info: &Information) {
        // Check if memory is in vulnerable state
        let time_since_retrieval = Utc::now() - memory.last_retrieval;
        
        if time_since_retrieval < self.vulnerability_window {
            let prediction_error = self.compute_prediction_error(memory, new_info);
            
            if prediction_error > self.update_threshold {
                // Update memory trace
                self.update_memory_trace(memory, new_info, prediction_error);
                
                // Reset consolidation
                memory.consolidation_level = 0.0;
            }
        }
    }
    
    fn update_memory_trace(&self, memory: &mut MemoryItem, new_info: &Information, error: f64) {
        // Blend old and new information
        let update_rate = error.min(0.5);  // Cap update rate
        
        memory.content = self.blend_content(&memory.content, new_info, update_rate);
        memory.last_modified = Utc::now();
        
        // Temporary weakening during reconsolidation
        memory.memory_strength.strength *= 0.8;
    }
}
```

## Metamemory

Model learner's awareness of their memory:

```rust
pub struct Metamemory {
    pub judgments_of_learning: HashMap<String, f64>,
    pub feeling_of_knowing: HashMap<String, f64>,
    pub calibration_accuracy: f64,
}

impl Metamemory {
    pub fn predict_recall(&self, item: &MemoryItem) -> f64 {
        // Judgment of Learning (JOL)
        let jol = self.judgments_of_learning.get(&item.id).unwrap_or(&0.5);
        
        // Adjust based on calibration accuracy
        let actual_strength = item.memory_strength.strength;
        let weighted = self.calibration_accuracy * actual_strength + 
                      (1.0 - self.calibration_accuracy) * jol;
        
        weighted
    }
    
    pub fn update_metamemory(&mut self, item_id: &str, predicted: f64, actual: bool) {
        // Update JOL based on prediction accuracy
        let error = (predicted - if actual { 1.0 } else { 0.0 }).abs();
        
        self.judgments_of_learning.entry(item_id.to_string())
            .and_modify(|jol| *jol = *jol * 0.9 + (1.0 - error) * 0.1)
            .or_insert(1.0 - error);
        
        // Update calibration
        self.update_calibration(predicted, actual);
    }
    
    fn update_calibration(&mut self, predicted: f64, actual: bool) {
        let actual_value = if actual { 1.0 } else { 0.0 };
        let accuracy = 1.0 - (predicted - actual_value).abs();
        
        // Exponential moving average
        self.calibration_accuracy = 0.95 * self.calibration_accuracy + 0.05 * accuracy;
    }
}
```

## Distributed Practice Scheduling

```rust
pub struct DistributedScheduler {
    pub total_time: Duration,
    pub min_spacing: Duration,
    pub items: Vec<MemoryItem>,
}

impl DistributedScheduler {
    pub fn optimize_schedule(&self) -> Schedule {
        // Lagrangian optimization for distributed practice
        let n_items = self.items.len();
        let n_sessions = (self.total_time / self.min_spacing) as usize;
        
        // Binary variables: x[i][j] = 1 if item i practiced in session j
        let mut schedule = Matrix::zeros(n_items, n_sessions);
        
        // Constraints: spacing between practices
        for i in 0..n_items {
            let optimal_spacings = self.compute_optimal_spacings(&self.items[i]);
            
            for (session, spacing) in optimal_spacings.into_iter().enumerate() {
                if session < n_sessions {
                    schedule[(i, session)] = 1.0;
                }
            }
        }
        
        // Balance session loads
        self.balance_sessions(&mut schedule);
        
        Schedule::from_matrix(schedule, &self.items)
    }
    
    fn compute_optimal_spacings(&self, item: &MemoryItem) -> Vec<usize> {
        let mut spacings = Vec::new();
        let mut current_session = 0;
        let mut interval = 1;
        
        while current_session < n_sessions {
            spacings.push(current_session);
            current_session += interval;
            interval = (interval as f64 * 2.5) as usize;  // Expanding intervals
        }
        
        spacings
    }
}
```

## Summary

Memory and forgetting models provide:
- **Forgetting curves**: Ebbinghaus and ACT-R models
- **Spacing algorithms**: Optimal interval computation
- **Consolidation**: Sleep-dependent strengthening
- **Interference**: Proactive and retroactive effects
- **Retrieval practice**: Testing effect optimization
- **Reconsolidation**: Memory updating mechanisms
- **Metamemory**: Model awareness of memory
- **Distributed practice**: Optimal scheduling

These models enable the system to optimize review timing and practice scheduling for durable learning.