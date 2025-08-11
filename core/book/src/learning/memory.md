# Memory & Forgetting

Memory and forgetting are fundamental to human learning. ABCDeez Core implements sophisticated models based on decades of cognitive science research to accurately simulate how knowledge is retained and lost over time.

## Theoretical Foundation

### The Forgetting Curve

Hermann Ebbinghaus discovered that memory decays exponentially over time:

```math
R(t) = R₀ · e^(-t/τ)
```

Where:
- **R(t)**: Retention at time t
- **R₀**: Initial retention strength
- **τ**: Time constant (decay rate)

### Modern Extensions

Our implementation extends the basic model with:
1. **Strength-dependent decay**: Stronger memories decay slower
2. **Practice effects**: Each review strengthens memory
3. **Spacing effects**: Distributed practice is more effective
4. **Individual differences**: Personalized decay rates

## Implementation

### Memory Strength Representation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStrength {
    pub node_id: String,
    pub strength: f64,  // Current strength [0, 1]
    pub last_practice: chrono::DateTime<chrono::Utc>,
}
```

### Core Memory Update Algorithm

```rust
impl LearnerModel {
    pub fn update_memory_strength(&mut self, node_id: &str, correct: bool) {
        if let Some(mem) = self.memory_strengths.get_mut(node_id) {
            let now = chrono::Utc::now();
            let time_since = now.signed_duration_since(mem.last_practice);
            let hours_since = time_since.num_hours() as f64;
            
            // Apply forgetting before update
            let decay_rate = self.calculate_decay_rate(mem.strength);
            let decayed_strength = self.apply_forgetting_curve(
                mem.strength, 
                hours_since, 
                decay_rate
            );
            
            // Update based on response
            if correct {
                // Successful retrieval strengthens memory
                mem.strength = (decayed_strength + self.config.memory_update_correct)
                    .min(1.0);
            } else {
                // Failure weakens memory but provides learning opportunity
                mem.strength = (decayed_strength + self.config.memory_update_incorrect)
                    .max(0.0);
            }
            
            mem.last_practice = now;
        }
    }
}
```

### Adaptive Decay Rate

Decay rate varies with memory strength:

```rust
fn calculate_decay_rate(&self, current_strength: f64) -> f64 {
    // Stronger memories decay slower (memory consolidation effect)
    self.config.memory_decay_rate * (2.0 - current_strength)
}
```

This implements the finding that well-learned material is forgotten more slowly.

### Forgetting Curve Application

```rust
fn apply_forgetting_curve(
    &self, 
    strength: f64, 
    hours_elapsed: f64, 
    decay_rate: f64
) -> f64 {
    // Exponential decay with floor to prevent complete forgetting
    let decayed = strength * (-decay_rate * hours_elapsed).exp();
    decayed.max(self.config.minimum_retention)
}
```

## Spacing Effects

The spacing effect shows that distributed practice leads to better retention:

### Optimal Spacing Calculation

```rust
impl LearnerModel {
    pub fn get_optimal_review_time(
        &self,
        node_id: &str,
        target_retention: f64,
    ) -> Option<chrono::DateTime<chrono::Utc>> {
        let mem = self.memory_strengths.get(node_id)?;
        
        // If already below target, review immediately
        if mem.strength <= target_retention {
            return Some(chrono::Utc::now());
        }
        
        // Calculate when retention will drop to target
        let decay_rate = self.calculate_decay_rate(mem.strength);
        let hours_until_review = -(target_retention / mem.strength).ln() / decay_rate;
        
        Some(mem.last_practice + chrono::Duration::hours(hours_until_review as i64))
    }
}
```

### Expanding Intervals

Successful reviews lead to expanding review intervals:

```rust
pub struct SpacedRepetitionSchedule {
    pub initial_interval: Duration,
    pub multiplier: f64,
    pub max_interval: Duration,
}

impl SpacedRepetitionSchedule {
    pub fn next_interval(&self, success_count: usize) -> Duration {
        let days = self.initial_interval.num_days() as f64 
                   * self.multiplier.powi(success_count as i32);
        
        let interval_days = days.min(self.max_interval.num_days() as f64);
        Duration::days(interval_days as i64)
    }
}
```

## Retrieval Practice

Retrieval practice (testing) strengthens memory more than passive review:

### Testing Effect Implementation

```rust
pub struct RetrievalPractice {
    pub boost_successful: f64,  // Larger boost for retrieval
    pub boost_passive: f64,     // Smaller boost for review
}

impl LearnerModel {
    pub fn apply_practice_effect(
        &mut self, 
        node_id: &str, 
        practice_type: PracticeType,
        success: bool
    ) {
        let boost = match (practice_type, success) {
            (PracticeType::Retrieval, true) => 0.3,   // Strong boost
            (PracticeType::Retrieval, false) => 0.1,  // Learning from error
            (PracticeType::Passive, _) => 0.05,       // Weak boost
        };
        
        if let Some(mem) = self.memory_strengths.get_mut(node_id) {
            mem.strength = (mem.strength + boost).min(1.0);
        }
    }
}
```

## Memory Consolidation

Sleep and time allow memories to consolidate:

### Consolidation Model

```rust
pub struct ConsolidationModel {
    pub fast_decay_rate: f64,    // Initial fast decay
    pub slow_decay_rate: f64,    // Long-term slow decay
    pub transition_hours: f64,   // When transition occurs
}

impl ConsolidationModel {
    pub fn retention_probability(&self, initial: f64, hours: f64) -> f64 {
        if hours < self.transition_hours {
            // Fast decay phase (working/short-term memory)
            initial * (-self.fast_decay_rate * hours).exp()
        } else {
            // Slow decay phase (long-term memory)
            let transition_strength = initial * 
                (-self.fast_decay_rate * self.transition_hours).exp();
            
            let hours_in_ltm = hours - self.transition_hours;
            transition_strength * (-self.slow_decay_rate * hours_in_ltm).exp()
        }
    }
}
```

## Individual Differences

People vary in memory abilities:

### Memory Profiles

```rust
#[derive(Debug, Clone)]
pub struct MemoryProfile {
    pub encoding_efficiency: f64,  // How well information is encoded
    pub decay_resistance: f64,     // Resistance to forgetting
    pub retrieval_efficiency: f64, // Ability to access memories
}

impl MemoryProfile {
    pub fn adult_typical() -> Self {
        Self {
            encoding_efficiency: 0.7,
            decay_resistance: 0.6,
            retrieval_efficiency: 0.8,
        }
    }
    
    pub fn child_typical() -> Self {
        Self {
            encoding_efficiency: 0.5,  // Less efficient encoding
            decay_resistance: 0.4,     // Faster forgetting
            retrieval_efficiency: 0.7, // Good retrieval when encoded
        }
    }
    
    pub fn expert_typical() -> Self {
        Self {
            encoding_efficiency: 0.9,  // Excellent encoding strategies
            decay_resistance: 0.8,     // Strong memory consolidation
            retrieval_efficiency: 0.9, // Efficient retrieval paths
        }
    }
}
```

### Applying Individual Differences

```rust
impl LearnerModel {
    pub fn apply_memory_profile(&mut self, profile: &MemoryProfile) {
        // Adjust encoding
        self.config.memory_update_correct *= profile.encoding_efficiency;
        
        // Adjust decay
        self.config.memory_decay_rate /= profile.decay_resistance;
        
        // Adjust retrieval threshold
        self.config.retrieval_threshold *= profile.retrieval_efficiency;
    }
}
```

## Interference Effects

New learning can interfere with old memories:

### Proactive Interference

Old learning interferes with new:

```rust
pub fn calculate_proactive_interference(
    &self,
    new_item: &str,
    existing_items: &[String]
) -> f64 {
    let mut interference = 0.0;
    
    for existing in existing_items {
        let similarity = self.calculate_similarity(new_item, existing);
        let strength = self.get_memory_strength(existing);
        
        // Stronger similar memories cause more interference
        interference += similarity * strength * 0.2;
    }
    
    interference.min(0.5) // Cap maximum interference
}
```

### Retroactive Interference

New learning interferes with old:

```rust
pub fn apply_retroactive_interference(
    &mut self,
    new_item: &str,
    strength_boost: f64
) {
    let new_node = self.get_node_embedding(new_item);
    
    for (id, mem) in self.memory_strengths.iter_mut() {
        if id != new_item {
            let similarity = self.calculate_similarity(new_item, id);
            
            // Reduce strength of similar items
            let interference = similarity * strength_boost * 0.1;
            mem.strength = (mem.strength - interference).max(0.0);
        }
    }
}
```

## Metamemory

Awareness of one's own memory:

### Feeling of Knowing

```rust
pub struct Metamemory {
    pub judgment_accuracy: f64,
    pub confidence_calibration: f64,
}

impl Metamemory {
    pub fn feeling_of_knowing(&self, actual_strength: f64) -> f64 {
        // People can somewhat judge what they know
        let perceived = actual_strength * self.judgment_accuracy 
                       + rand::random::<f64>() * (1.0 - self.judgment_accuracy);
        
        // Apply confidence calibration
        if perceived > 0.5 {
            perceived * self.confidence_calibration
        } else {
            perceived / self.confidence_calibration
        }
    }
}
```

## Practical Applications

### Review Scheduling System

```rust
pub struct ReviewScheduler {
    pub target_retention: f64,
    pub min_interval: Duration,
    pub max_interval: Duration,
}

impl ReviewScheduler {
    pub fn schedule_reviews(
        &self,
        learner: &LearnerModel,
        items: &[String],
    ) -> Vec<(String, DateTime<Utc>)> {
        let mut schedule = Vec::new();
        
        for item in items {
            if let Some(review_time) = learner.get_optimal_review_time(
                item,
                self.target_retention
            ) {
                schedule.push((item.clone(), review_time));
            }
        }
        
        // Sort by urgency
        schedule.sort_by_key(|(_, time)| *time);
        schedule
    }
}
```

### Memory Diagnostic

```rust
pub fn diagnose_memory_issues(learner: &LearnerModel) -> MemoryDiagnosis {
    let strengths: Vec<f64> = learner.memory_strengths
        .values()
        .map(|m| m.strength)
        .collect();
    
    let mean_strength = mean(&strengths);
    let strength_variance = variance(&strengths);
    
    let recent_items = learner.get_recently_practiced(24); // Last 24 hours
    let retention_rate = learner.calculate_retention_rate(&recent_items);
    
    MemoryDiagnosis {
        overall_retention: mean_strength,
        consistency: 1.0 - strength_variance,
        recent_performance: retention_rate,
        needs_review: learner.get_items_needing_review(0.6),
        at_risk: learner.get_items_below_threshold(0.3),
    }
}
```

## Validation

Our memory model is validated against classic findings:

| Phenomenon | Literature | Our Model |
|------------|-----------|-----------|
| Exponential forgetting | Ebbinghaus (1885) | ✓ Implemented |
| Spacing effect | Cepeda et al. (2006) | ✓ Reproduced |
| Testing effect | Roediger & Karpicke (2006) | ✓ Modeled |
| Strength-dependent decay | Wixted (2004) | ✓ Incorporated |
| Interference | Underwood (1957) | ✓ Simulated |

## Configuration Options

```rust
pub struct MemoryConfig {
    // Decay parameters
    pub memory_decay_rate: f64,        // Base decay rate
    pub minimum_retention: f64,        // Floor for retention
    
    // Update parameters
    pub memory_update_correct: f64,    // Boost for correct response
    pub memory_update_incorrect: f64,  // Penalty for incorrect
    
    // Consolidation
    pub consolidation_threshold: f64,  // When memories consolidate
    pub consolidation_bonus: f64,      // Boost from consolidation
    
    // Individual differences
    pub enable_individual_differences: bool,
    pub memory_profile: MemoryProfile,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            memory_decay_rate: 0.05,
            minimum_retention: 0.1,
            memory_update_correct: 0.2,
            memory_update_incorrect: -0.1,
            consolidation_threshold: 0.7,
            consolidation_bonus: 0.1,
            enable_individual_differences: false,
            memory_profile: MemoryProfile::adult_typical(),
        }
    }
}
```

## Next Steps

- Learn about [Operation Proficiency](./proficiency.md)
- Explore [Chunk Boundaries](./chunking.md)
- Understand [Node Embeddings](./embeddings.md)
- Read about [Spacing Effects](../psychology/spacing.md)