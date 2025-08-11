# Operation Proficiency

Operation proficiency tracks learners' skill development across different cognitive operations. Unlike static knowledge representations, proficiency models capture the dynamic aspects of skill acquisition, including practice effects, automaticity development, and individual learning trajectories.

## Conceptual Foundation

### What is Operation Proficiency?

Operation proficiency quantifies how well a learner can execute specific cognitive operations. It encompasses:

1. **Accuracy**: The likelihood of performing the operation correctly
2. **Speed**: How quickly the operation can be executed
3. **Automaticity**: The degree to which the operation has become automatic
4. **Consistency**: How reliably the skill level is maintained over time
5. **Transfer**: How well the skill generalizes to related operations

### Theoretical Background

The proficiency model draws from several psychological theories:

- **Power Law of Practice**: Performance improves as a power function of practice
- **Adaptive Control of Thought (ACT-R)**: Skills develop from declarative to procedural knowledge
- **Instance Theory**: Repeated exposure creates stronger memory traces
- **Skill Acquisition Theory**: Learning progresses through cognitive, associative, and autonomous stages

## Core Data Structures

### Operation Proficiency

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProficiency {
    pub operation: OperationType,
    pub theta: f64,                    // Logit-scale proficiency [-∞, ∞]
    pub practice_count: usize,         // Number of practice attempts
    pub success_count: usize,          // Number of successful attempts
    pub last_practiced: DateTime<Utc>, // When last practiced
    pub learning_rate: f64,            // Current adaptive learning rate
    pub automaticity_level: f64,       // [0, 1] how automatic the skill is
    pub consistency_score: f64,        // [0, 1] how consistent performance is
}

impl OperationProficiency {
    pub fn new(operation: OperationType) -> Self {
        Self {
            operation,
            theta: -1.0, // Start slightly below average
            practice_count: 0,
            success_count: 0,
            last_practiced: Utc::now(),
            learning_rate: 0.3, // Initial learning rate
            automaticity_level: 0.0,
            consistency_score: 0.0,
        }
    }
    
    pub fn accuracy(&self) -> f64 {
        // Convert logit scale to probability
        1.0 / (1.0 + (-self.theta).exp())
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.practice_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.practice_count as f64
        }
    }
    
    pub fn is_well_learned(&self) -> bool {
        self.theta > 1.0 && self.practice_count >= 10 && self.automaticity_level > 0.7
    }
    
    pub fn needs_practice(&self) -> bool {
        self.theta < 0.0 || self.consistency_score < 0.5
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    // Sequential operations
    Successor,                      // A → B
    Predecessor,                    // B → A
    KJump(i32),                     // A → C (jump k positions)
    
    // Comparison operations
    PairwiseOrder,                  // A < B?
    Magnitude,                      // |A - B|
    
    // Structural operations
    Segment(usize, bool),           // Recite sequence (count, reverse)
    Index,                          // Position of item
    Search,                         // Find item in sequence
    
    // Complex operations
    Transpose,                      // Swap elements
    Insert,                         // Insert at position
    Delete,                         // Remove element
    
    // Meta-cognitive operations
    EstimateDifficulty,            // Judge task difficulty
    ConfidenceRating,              // Rate confidence in response
}
```

### Learning Curves and Models

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningCurve {
    pub operation: OperationType,
    pub data_points: Vec<LearningDataPoint>,
    pub fitted_parameters: PowerLawParameters,
    pub model_fit_r2: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningDataPoint {
    pub trial_number: usize,
    pub accuracy: f64,
    pub response_time: f64,
    pub timestamp: DateTime<Utc>,
    pub difficulty_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerLawParameters {
    pub initial_performance: f64,    // Performance at trial 1
    pub learning_rate: f64,          // How fast improvement occurs
    pub asymptotic_performance: f64, // Performance ceiling
    pub power_exponent: f64,         // Shape of the learning curve
}

impl LearningCurve {
    pub fn predict_performance(&self, trial_number: usize) -> f64 {
        let n = trial_number as f64;
        let p = &self.fitted_parameters;
        
        // Power law of practice: P(n) = P_initial + (P_asymptote - P_initial) * (1 - n^(-α))
        p.initial_performance + 
        (p.asymptotic_performance - p.initial_performance) * 
        (1.0 - (n * p.learning_rate).powf(-p.power_exponent))
    }
    
    pub fn fit_power_law(&mut self) {
        if self.data_points.len() < 5 {
            return; // Need minimum data for fitting
        }
        
        // Use non-linear least squares to fit parameters
        let (params, r2) = self.fit_power_law_parameters();
        self.fitted_parameters = params;
        self.model_fit_r2 = r2;
    }
    
    fn fit_power_law_parameters(&self) -> (PowerLawParameters, f64) {
        // Simplified fitting - in practice would use robust optimization
        let accuracies: Vec<f64> = self.data_points.iter().map(|p| p.accuracy).collect();
        
        let initial = accuracies.first().unwrap_or(&0.5);
        let final_avg = accuracies.iter().rev().take(3).sum::<f64>() / 3.0;
        
        let params = PowerLawParameters {
            initial_performance: *initial,
            learning_rate: 0.5,
            asymptotic_performance: final_avg.max(*initial + 0.1),
            power_exponent: 0.3,
        };
        
        // Calculate R² (simplified)
        let r2 = 0.75; // Would compute actual correlation
        
        (params, r2)
    }
}
```

## Proficiency Learning Algorithms

### Adaptive Learning Rate Updates

```rust
pub struct ProficiencyLearner {
    pub base_learning_rate: f64,
    pub decay_rate: f64,
    pub difficulty_sensitivity: f64,
    pub automaticity_threshold: f64,
    pub consistency_window_size: usize,
}

impl ProficiencyLearner {
    pub fn new() -> Self {
        Self {
            base_learning_rate: 0.3,
            decay_rate: 0.9,
            difficulty_sensitivity: 0.2,
            automaticity_threshold: 0.8,
            consistency_window_size: 10,
        }
    }
    
    pub fn update_proficiency(
        &self,
        proficiency: &mut OperationProficiency,
        task_response: &TaskResponse,
        task_difficulty: f64,
    ) {
        let correct = task_response.correct;
        let response_time = task_response.response_time_ms;
        
        // Update basic statistics
        proficiency.practice_count += 1;
        if correct {
            proficiency.success_count += 1;
        }
        proficiency.last_practiced = Utc::now();
        
        // Calculate adaptive learning rate
        let learning_rate = self.calculate_adaptive_learning_rate(proficiency, task_difficulty);
        proficiency.learning_rate = learning_rate;
        
        // Update theta (proficiency on logit scale)
        self.update_theta(proficiency, correct, task_difficulty, learning_rate);
        
        // Update automaticity
        self.update_automaticity(proficiency, response_time, correct);
        
        // Update consistency score
        self.update_consistency(proficiency, correct);
    }
    
    fn calculate_adaptive_learning_rate(
        &self,
        proficiency: &OperationProficiency,
        task_difficulty: f64,
    ) -> f64 {
        // Base rate decreases with practice (power law)
        let practice_factor = (proficiency.practice_count as f64 + 1.0).powf(-self.decay_rate);
        let base_rate = self.base_learning_rate * practice_factor;
        
        // Adjust based on current proficiency level
        let proficiency_factor = 1.0 - proficiency.accuracy();
        
        // Adjust based on task difficulty
        let difficulty_factor = 1.0 + self.difficulty_sensitivity * task_difficulty;
        
        (base_rate * proficiency_factor * difficulty_factor)
            .max(0.01)  // Minimum learning rate
            .min(0.5)   // Maximum learning rate
    }
    
    fn update_theta(
        &self,
        proficiency: &mut OperationProficiency,
        correct: bool,
        task_difficulty: f64,
        learning_rate: f64,
    ) {
        if correct {
            // Successful response increases theta
            let current_prob = proficiency.accuracy();
            let increment = learning_rate * (1.0 - current_prob) * (1.0 + task_difficulty);
            proficiency.theta += increment;
        } else {
            // Failed response decreases theta
            let current_prob = proficiency.accuracy();
            let decrement = learning_rate * current_prob * (2.0 - task_difficulty);
            proficiency.theta -= decrement;
        }
        
        // Keep theta within reasonable bounds
        proficiency.theta = proficiency.theta.max(-5.0).min(5.0);
    }
    
    fn update_automaticity(
        &self,
        proficiency: &mut OperationProficiency,
        response_time: f64,
        correct: bool,
    ) {
        if !correct {
            // Errors reduce automaticity
            proficiency.automaticity_level *= 0.95;
            return;
        }
        
        // Fast correct responses increase automaticity
        let target_time = 1500.0; // Target response time in ms
        let speed_factor = if response_time < target_time {
            (target_time / response_time).min(2.0)
        } else {
            (target_time / response_time).max(0.5)
        };
        
        let automaticity_increment = 0.05 * speed_factor;
        proficiency.automaticity_level = 
            (proficiency.automaticity_level + automaticity_increment).min(1.0);
    }
    
    fn update_consistency(
        &self,
        proficiency: &mut OperationProficiency,
        correct: bool,
    ) {
        // Simple consistency score based on recent performance
        // In practice, would track detailed performance history
        
        let current_success_rate = proficiency.success_rate();
        let expected_success = proficiency.accuracy();
        
        // Consistency is how close actual performance is to expected
        let performance_deviation = (current_success_rate - expected_success).abs();
        let new_consistency = 1.0 - performance_deviation;
        
        // Update with exponential moving average
        proficiency.consistency_score = 
            0.9 * proficiency.consistency_score + 0.1 * new_consistency.max(0.0);
    }
}
```

### Skill Transfer Modeling

```rust
pub struct SkillTransfer {
    pub source_operation: OperationType,
    pub target_operation: OperationType,
    pub transfer_strength: f64,      // [0, 1] how much skill transfers
    pub transfer_type: TransferType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    Positive,      // Source skill helps target skill
    Negative,      // Source skill interferes with target skill
    Neutral,       // No interaction between skills
}

pub struct TransferLearner {
    pub transfer_rules: Vec<SkillTransfer>,
    pub transfer_decay: f64,
}

impl TransferLearner {
    pub fn calculate_transfer_benefit(
        &self,
        target_operation: &OperationType,
        all_proficiencies: &HashMap<String, OperationProficiency>,
    ) -> f64 {
        let mut total_benefit = 0.0;
        
        for transfer_rule in &self.transfer_rules {
            if transfer_rule.target_operation == *target_operation {
                let source_key = format!("{:?}", transfer_rule.source_operation);
                
                if let Some(source_prof) = all_proficiencies.get(&source_key) {
                    let source_strength = source_prof.accuracy();
                    
                    let transfer_amount = match transfer_rule.transfer_type {
                        TransferType::Positive => {
                            transfer_rule.transfer_strength * source_strength
                        }
                        TransferType::Negative => {
                            -transfer_rule.transfer_strength * source_strength
                        }
                        TransferType::Neutral => 0.0,
                    };
                    
                    total_benefit += transfer_amount;
                }
            }
        }
        
        total_benefit.max(-0.5).min(0.5) // Bound transfer effects
    }
    
    pub fn create_default_transfer_rules() -> Vec<SkillTransfer> {
        vec![
            // Forward/backward transfer
            SkillTransfer {
                source_operation: OperationType::Successor,
                target_operation: OperationType::Predecessor,
                transfer_strength: 0.4,
                transfer_type: TransferType::Positive,
            },
            
            // Jump operations transfer to each other
            SkillTransfer {
                source_operation: OperationType::KJump(1),
                target_operation: OperationType::KJump(2),
                transfer_strength: 0.6,
                transfer_type: TransferType::Positive,
            },
            
            // Segment operations
            SkillTransfer {
                source_operation: OperationType::Segment(3, false),
                target_operation: OperationType::Segment(4, false),
                transfer_strength: 0.5,
                transfer_type: TransferType::Positive,
            },
            
            // Metacognitive skills
            SkillTransfer {
                source_operation: OperationType::ConfidenceRating,
                target_operation: OperationType::EstimateDifficulty,
                transfer_strength: 0.3,
                transfer_type: TransferType::Positive,
            },
            
            // Interference examples
            SkillTransfer {
                source_operation: OperationType::Segment(3, false),
                target_operation: OperationType::Segment(3, true),
                transfer_strength: 0.2,
                transfer_type: TransferType::Negative,
            },
        ]
    }
}
```

## Individual Difference Measures

### Proficiency Profiles

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProficiencyProfile {
    pub learner_id: String,
    pub proficiencies: HashMap<String, OperationProficiency>,
    pub learning_style: LearningStyle,
    pub cognitive_load_sensitivity: f64,
    pub error_recovery_rate: f64,
    pub metacognitive_awareness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningStyle {
    Sequential,    // Prefers step-by-step learning
    Global,        // Learns by seeing big picture first
    Active,        // Learns by doing
    Reflective,    // Learns by thinking about it
    Adaptive,      // Adjusts style based on content
}

impl ProficiencyProfile {
    pub fn calculate_skill_breadth(&self) -> f64 {
        // How many different operations are well-learned
        let well_learned_count = self.proficiencies
            .values()
            .filter(|p| p.is_well_learned())
            .count() as f64;
            
        well_learned_count / self.proficiencies.len().max(1) as f64
    }
    
    pub fn calculate_skill_depth(&self) -> f64 {
        // Average proficiency across all operations
        if self.proficiencies.is_empty() {
            return 0.0;
        }
        
        self.proficiencies
            .values()
            .map(|p| p.accuracy())
            .sum::<f64>() / self.proficiencies.len() as f64
    }
    
    pub fn get_strength_weaknesses(&self) -> (Vec<OperationType>, Vec<OperationType>) {
        let mut strengths = Vec::new();
        let mut weaknesses = Vec::new();
        
        let mean_accuracy = self.calculate_skill_depth();
        
        for proficiency in self.proficiencies.values() {
            if proficiency.accuracy() > mean_accuracy + 0.2 && proficiency.is_well_learned() {
                strengths.push(proficiency.operation.clone());
            } else if proficiency.accuracy() < mean_accuracy - 0.2 || proficiency.needs_practice() {
                weaknesses.push(proficiency.operation.clone());
            }
        }
        
        (strengths, weaknesses)
    }
    
    pub fn recommend_next_practice(&self) -> Option<OperationType> {
        // Find operation that would benefit most from practice
        let mut candidates: Vec<_> = self.proficiencies
            .values()
            .filter(|p| p.needs_practice())
            .collect();
            
        // Sort by potential improvement (low proficiency, high learning rate)
        candidates.sort_by(|a, b| {
            let score_a = a.learning_rate * (1.0 - a.accuracy());
            let score_b = b.learning_rate * (1.0 - b.accuracy());
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        candidates.first().map(|p| p.operation.clone())
    }
}
```

### Expertise Modeling

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertiseLevel {
    pub overall_level: ExpertiseCategory,
    pub domain_specific_levels: HashMap<OperationType, ExpertiseCategory>,
    pub development_trajectory: Vec<ExpertiseSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpertiseCategory {
    Novice,        // θ < -1.0, inconsistent performance
    Advanced,      // -1.0 ≤ θ < 0.5, developing skills  
    Proficient,    // 0.5 ≤ θ < 1.5, solid performance
    Expert,        // 1.5 ≤ θ < 2.5, highly skilled
    Master,        // θ ≥ 2.5, exceptional performance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertiseSnapshot {
    pub timestamp: DateTime<Utc>,
    pub overall_proficiency: f64,
    pub automaticity_score: f64,
    pub consistency_score: f64,
    pub breadth_score: f64,
}

impl ExpertiseLevel {
    pub fn assess_expertise(profile: &ProficiencyProfile) -> Self {
        let overall_level = Self::categorize_expertise(
            profile.calculate_skill_depth(),
            profile.calculate_skill_breadth(),
        );
        
        let mut domain_specific_levels = HashMap::new();
        for (op_name, prof) in &profile.proficiencies {
            let category = Self::categorize_operation_expertise(prof);
            if let Ok(op_type) = serde_json::from_str::<OperationType>(&format!("\"{}\"", op_name)) {
                domain_specific_levels.insert(op_type, category);
            }
        }
        
        Self {
            overall_level,
            domain_specific_levels,
            development_trajectory: Vec::new(),
        }
    }
    
    fn categorize_expertise(skill_depth: f64, skill_breadth: f64) -> ExpertiseCategory {
        // Combine depth and breadth for overall assessment
        let combined_score = (skill_depth + skill_breadth) / 2.0;
        
        match combined_score {
            x if x >= 0.9 => ExpertiseCategory::Master,
            x if x >= 0.8 => ExpertiseCategory::Expert,
            x if x >= 0.7 => ExpertiseCategory::Proficient,
            x if x >= 0.5 => ExpertiseCategory::Advanced,
            _ => ExpertiseCategory::Novice,
        }
    }
    
    fn categorize_operation_expertise(prof: &OperationProficiency) -> ExpertiseCategory {
        let theta = prof.theta;
        let automaticity = prof.automaticity_level;
        let consistency = prof.consistency_score;
        
        // Consider both accuracy and fluency
        let composite_score = prof.accuracy() * 0.5 + automaticity * 0.3 + consistency * 0.2;
        
        match (theta, composite_score) {
            (t, c) if t >= 2.5 && c >= 0.9 => ExpertiseCategory::Master,
            (t, c) if t >= 1.5 && c >= 0.8 => ExpertiseCategory::Expert,
            (t, c) if t >= 0.5 && c >= 0.7 => ExpertiseCategory::Proficient,
            (t, c) if t >= -1.0 && c >= 0.5 => ExpertiseCategory::Advanced,
            _ => ExpertiseCategory::Novice,
        }
    }
}
```

## Performance Prediction

### Response Time Modeling

```rust
pub struct ResponseTimePredictor {
    pub base_time: f64,                    // Base processing time
    pub proficiency_speedup: f64,          // How proficiency reduces time
    pub automaticity_speedup: f64,         // How automaticity reduces time
    pub difficulty_slowdown: f64,          // How difficulty increases time
    pub consistency_factor: f64,           // How consistency affects variability
}

impl ResponseTimePredictor {
    pub fn predict_response_time(
        &self,
        proficiency: &OperationProficiency,
        task_difficulty: f64,
        include_variability: bool,
    ) -> f64 {
        // Base time adjusted by proficiency
        let accuracy = proficiency.accuracy();
        let proficiency_multiplier = self.base_time * (2.0 - accuracy * self.proficiency_speedup);
        
        // Automaticity reduces time further
        let automaticity_multiplier = 1.0 - proficiency.automaticity_level * self.automaticity_speedup;
        
        // Difficulty increases time
        let difficulty_multiplier = 1.0 + task_difficulty * self.difficulty_slowdown;
        
        let predicted_time = proficiency_multiplier * automaticity_multiplier * difficulty_multiplier;
        
        if include_variability {
            // Add variability based on consistency
            let variability = (1.0 - proficiency.consistency_score) * self.consistency_factor;
            let noise = rand::thread_rng().gen_range(-variability..variability);
            (predicted_time * (1.0 + noise)).max(200.0) // Minimum 200ms
        } else {
            predicted_time.max(200.0)
        }
    }
    
    pub fn predict_accuracy(
        &self,
        proficiency: &OperationProficiency,
        task_difficulty: f64,
    ) -> f64 {
        // Adjust proficiency by task difficulty
        let effective_theta = proficiency.theta - task_difficulty;
        
        // Convert to probability with sigmoid
        1.0 / (1.0 + (-effective_theta).exp())
    }
}
```

### Learning Progress Prediction

```rust
pub struct ProgressPredictor {
    pub learning_curves: HashMap<OperationType, LearningCurve>,
    pub transfer_model: TransferLearner,
}

impl ProgressPredictor {
    pub fn predict_future_proficiency(
        &self,
        current_prof: &OperationProficiency,
        additional_practice: usize,
        practice_difficulty: f64,
    ) -> OperationProficiency {
        let mut predicted_prof = current_prof.clone();
        
        // Simulate additional practice sessions
        for trial in 1..=additional_practice {
            let current_accuracy = predicted_prof.accuracy();
            
            // Predict whether this trial would be successful
            let effective_difficulty = practice_difficulty - predicted_prof.theta;
            let success_probability = 1.0 / (1.0 + effective_difficulty.exp());
            let simulated_success = rand::thread_rng().gen::<f64>() < success_probability;
            
            // Apply learning update (simplified)
            let learning_rate = self.calculate_decay_adjusted_rate(&predicted_prof, trial);
            
            if simulated_success {
                predicted_prof.theta += learning_rate * (1.0 - current_accuracy);
                predicted_prof.success_count += 1;
            } else {
                predicted_prof.theta -= learning_rate * current_accuracy * 0.5;
            }
            
            predicted_prof.practice_count += 1;
            
            // Update automaticity and consistency (simplified)
            if simulated_success && trial % 5 == 0 {
                predicted_prof.automaticity_level = 
                    (predicted_prof.automaticity_level + 0.02).min(1.0);
            }
        }
        
        predicted_prof
    }
    
    fn calculate_decay_adjusted_rate(&self, prof: &OperationProficiency, trial: usize) -> f64 {
        let total_trials = prof.practice_count + trial;
        0.3 / (total_trials as f64).powf(0.3)
    }
    
    pub fn estimate_trials_to_mastery(
        &self,
        current_prof: &OperationProficiency,
        mastery_threshold: f64,
    ) -> Option<usize> {
        if current_prof.accuracy() >= mastery_threshold {
            return Some(0);
        }
        
        let mut test_prof = current_prof.clone();
        let mut trials = 0;
        const MAX_TRIALS: usize = 1000;
        
        while trials < MAX_TRIALS && test_prof.accuracy() < mastery_threshold {
            trials += 1;
            
            // Simulate a practice trial
            let learning_rate = self.calculate_decay_adjusted_rate(&test_prof, trials);
            let accuracy_gain = learning_rate * (mastery_threshold - test_prof.accuracy());
            
            test_prof.theta += accuracy_gain;
            test_prof.practice_count += 1;
        }
        
        if test_prof.accuracy() >= mastery_threshold {
            Some(trials)
        } else {
            None // May not reach mastery with current model
        }
    }
}
```

## Performance Optimization

### Efficient Proficiency Updates

```rust
pub struct BatchProficiencyUpdate {
    pub operations: Vec<OperationType>,
    pub responses: Vec<TaskResponse>,
    pub difficulties: Vec<f64>,
    pub batch_size: usize,
}

pub struct OptimizedProficiencyLearner {
    pub base_learner: ProficiencyLearner,
    pub update_buffer: HashMap<String, Vec<BatchProficiencyUpdate>>,
    pub batch_processing: bool,
}

impl OptimizedProficiencyLearner {
    pub fn queue_update(
        &mut self,
        operation: OperationType,
        response: TaskResponse,
        difficulty: f64,
    ) {
        let key = format!("{:?}", operation);
        
        self.update_buffer
            .entry(key)
            .or_default()
            .push(BatchProficiencyUpdate {
                operations: vec![operation],
                responses: vec![response],
                difficulties: vec![difficulty],
                batch_size: 1,
            });
    }
    
    pub fn flush_updates(&mut self, proficiencies: &mut HashMap<String, OperationProficiency>) {
        for (op_key, updates) in self.update_buffer.drain() {
            if let Some(prof) = proficiencies.get_mut(&op_key) {
                // Process all updates for this operation
                for batch in updates {
                    for ((response, difficulty)) in batch.responses.into_iter().zip(batch.difficulties) {
                        self.base_learner.update_proficiency(prof, &response, difficulty);
                    }
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
    fn test_proficiency_learning() {
        let mut prof = OperationProficiency::new(OperationType::Successor);
        let learner = ProficiencyLearner::new();
        
        let initial_accuracy = prof.accuracy();
        
        // Simulate successful practice
        for _ in 0..10 {
            let response = TaskResponse {
                correct: true,
                response_time_ms: 1200.0,
                // ... other fields
            };
            learner.update_proficiency(&mut prof, &response, 0.5);
        }
        
        assert!(prof.accuracy() > initial_accuracy);
        assert!(prof.practice_count == 10);
        assert!(prof.success_count == 10);
        assert!(prof.automaticity_level > 0.0);
    }
    
    #[test]
    fn test_learning_rate_adaptation() {
        let prof = OperationProficiency::new(OperationType::KJump(2));
        let learner = ProficiencyLearner::new();
        
        // High difficulty should increase learning rate
        let lr_hard = learner.calculate_adaptive_learning_rate(&prof, 0.8);
        let lr_easy = learner.calculate_adaptive_learning_rate(&prof, 0.2);
        
        assert!(lr_hard > lr_easy);
    }
    
    #[test]
    fn test_expertise_assessment() {
        let mut proficiencies = HashMap::new();
        
        // High-proficiency operation
        let mut high_prof = OperationProficiency::new(OperationType::Successor);
        high_prof.theta = 2.0;
        high_prof.automaticity_level = 0.9;
        high_prof.consistency_score = 0.8;
        proficiencies.insert("Successor".to_string(), high_prof);
        
        // Low-proficiency operation
        let low_prof = OperationProficiency::new(OperationType::Predecessor);
        proficiencies.insert("Predecessor".to_string(), low_prof);
        
        let profile = ProficiencyProfile {
            learner_id: "test".to_string(),
            proficiencies,
            learning_style: LearningStyle::Adaptive,
            cognitive_load_sensitivity: 0.5,
            error_recovery_rate: 0.7,
            metacognitive_awareness: 0.6,
        };
        
        let expertise = ExpertiseLevel::assess_expertise(&profile);
        
        // Should detect mixed expertise levels
        assert!(matches!(expertise.overall_level, ExpertiseCategory::Advanced | ExpertiseCategory::Proficient));
    }
    
    #[test]
    fn test_response_time_prediction() {
        let mut prof = OperationProficiency::new(OperationType::PairwiseOrder);
        prof.theta = 1.0;
        prof.automaticity_level = 0.6;
        prof.consistency_score = 0.8;
        
        let predictor = ResponseTimePredictor {
            base_time: 1000.0,
            proficiency_speedup: 0.5,
            automaticity_speedup: 0.3,
            difficulty_slowdown: 0.4,
            consistency_factor: 0.2,
        };
        
        let rt_easy = predictor.predict_response_time(&prof, 0.2, false);
        let rt_hard = predictor.predict_response_time(&prof, 0.8, false);
        
        assert!(rt_hard > rt_easy);
        assert!(rt_easy >= 200.0); // Minimum response time
    }
    
    #[test]
    fn test_skill_transfer() {
        let transfer_rules = TransferLearner::create_default_transfer_rules();
        let transfer_learner = TransferLearner {
            transfer_rules,
            transfer_decay: 0.9,
        };
        
        let mut proficiencies = HashMap::new();
        let mut successor_prof = OperationProficiency::new(OperationType::Successor);
        successor_prof.theta = 1.5; // High proficiency
        proficiencies.insert("Successor".to_string(), successor_prof);
        
        let benefit = transfer_learner.calculate_transfer_benefit(
            &OperationType::Predecessor,
            &proficiencies,
        );
        
        assert!(benefit > 0.0); // Should have positive transfer from Successor to Predecessor
    }
}
```

## Best Practices

1. **Start with reasonable priors**: Initialize proficiencies based on prior knowledge or population averages
2. **Adapt learning rates**: Use individual performance history to customize learning rates
3. **Track multiple metrics**: Monitor accuracy, speed, automaticity, and consistency together
4. **Model skill transfer**: Account for how skills support each other
5. **Regularize extreme values**: Prevent proficiency estimates from becoming unrealistic
6. **Validate predictions**: Test proficiency-based predictions against observed performance

## Common Pitfalls

- **Overfitting to recent performance**: Weight historical data appropriately
- **Ignoring skill interactions**: Consider how different operations relate to each other
- **Static learning rates**: Adapt rates based on individual progress patterns
- **Binary skill assessment**: Use continuous measures rather than pass/fail categories
- **Neglecting automaticity**: Track both accuracy and fluency development

## Applications

- **Adaptive testing**: Select questions that match current proficiency levels
- **Personalized practice**: Recommend operations that need development
- **Progress monitoring**: Track skill development over time
- **Expertise assessment**: Identify learners ready for advanced content
- **Intervention targeting**: Focus remediation on specific skill deficits

## Next Steps

- Explore [Memory Dynamics](./memory.md) for forgetting and retention modeling
- Learn about [Chunk Detection](./chunking.md) for structural knowledge patterns
- Understand [Bayesian Updates](../bayesian/updates.md) for uncertainty handling
- See [Statistical Validation](../statistics/core.md) for proficiency measurement validation