# Adaptive Scheduling

The adaptive scheduler orchestrates the learning experience by selecting tasks that maximize learning efficiency. It balances exploration (discovering what the learner doesn't know) with exploitation (reinforcing partial knowledge).

## Core Architecture

```rust
pub struct AdaptiveScheduler {
    pub learner: LearnerModel,
    pub bayesian_model: BayesianLearnerModel,
    pub task_generator: TaskGenerator,
    pub history: Vec<TaskResponse>,
    pub strategy: SchedulingStrategy,
}

pub enum SchedulingStrategy {
    MaximizeEIG,        // Always pick highest information gain
    EpsilonGreedy(f64), // Explore with probability ε
    UCB,                // Upper confidence bound
    Thompson,           // Thompson sampling
    Curriculum,         // Structured progression
}
```

## Task Selection Pipeline

The scheduler follows a systematic process:

```rust
impl AdaptiveScheduler {
    pub fn select_next_task(&mut self) -> Task {
        // 1. Generate candidate tasks
        let candidates = self.generate_candidates();
        
        // 2. Score each candidate
        let scored = self.score_candidates(candidates);
        
        // 3. Apply selection strategy
        let selected = self.apply_strategy(scored);
        
        // 4. Add pedagogical constraints
        let final_task = self.apply_constraints(selected);
        
        final_task
    }
}
```

### Step 1: Candidate Generation

Generate diverse tasks covering different aspects:

```rust
fn generate_candidates(&self) -> Vec<Task> {
    let mut candidates = Vec::new();
    let topology = &self.learner.topology;
    
    // Sample nodes with uncertainty-weighted probability
    let uncertain_nodes = self.get_most_uncertain_nodes(5);
    
    for node in uncertain_nodes {
        // Generate different task types for each node
        candidates.push(self.create_successor_task(&node));
        candidates.push(self.create_predecessor_task(&node));
        candidates.push(self.create_pairwise_task(&node));
        
        // Add k-jump tasks with varying k
        for k in [2, 3, 5] {
            candidates.push(self.create_kjump_task(&node, k));
        }
    }
    
    // Add segment tasks at chunk boundaries
    for boundary in &self.bayesian_model.chunk_boundaries {
        candidates.push(self.create_segment_task_at_boundary(*boundary));
    }
    
    candidates
}

fn get_most_uncertain_nodes(&self, n: usize) -> Vec<String> {
    let mut nodes_with_variance: Vec<_> = self.bayesian_model
        .node_positions
        .iter()
        .map(|(id, posterior)| (id.clone(), posterior.variance))
        .collect();
    
    // Sort by variance (descending)
    nodes_with_variance.sort_by(|a, b| 
        b.1.partial_cmp(&a.1).unwrap()
    );
    
    nodes_with_variance
        .into_iter()
        .take(n)
        .map(|(id, _)| id)
        .collect()
}
```

### Step 2: Scoring Candidates

Multiple factors contribute to task score:

```rust
fn score_candidates(&self, candidates: Vec<Task>) -> Vec<(Task, TaskScore)> {
    candidates.into_iter().map(|task| {
        let score = TaskScore {
            eig: self.bayesian_model.monte_carlo_eig(&task, 100),
            difficulty_match: self.compute_difficulty_match(&task),
            recency_penalty: self.compute_recency_penalty(&task),
            diversity_bonus: self.compute_diversity_bonus(&task),
            pedagogical_value: self.compute_pedagogical_value(&task),
        };
        
        (task, score)
    }).collect()
}

#[derive(Debug, Clone)]
struct TaskScore {
    eig: f64,               // Expected information gain
    difficulty_match: f64,   // How well difficulty matches learner
    recency_penalty: f64,    // Penalty for recently seen items
    diversity_bonus: f64,    // Bonus for task type variety
    pedagogical_value: f64,  // Educational structure value
}

impl TaskScore {
    fn total(&self) -> f64 {
        0.4 * self.eig +
        0.2 * self.difficulty_match +
        0.1 * (1.0 - self.recency_penalty) +
        0.1 * self.diversity_bonus +
        0.2 * self.pedagogical_value
    }
}
```

### Step 3: Strategy Application

Different strategies for different learning phases:

```rust
fn apply_strategy(&self, scored: Vec<(Task, TaskScore)>) -> Task {
    match self.strategy {
        SchedulingStrategy::MaximizeEIG => {
            // Pure exploitation: always pick highest EIG
            scored.into_iter()
                .max_by(|a, b| a.1.eig.partial_cmp(&b.1.eig).unwrap())
                .map(|(task, _)| task)
                .unwrap()
        }
        
        SchedulingStrategy::EpsilonGreedy(epsilon) => {
            // Balance exploration and exploitation
            if rand::random::<f64>() < epsilon {
                // Explore: random choice
                let idx = rand::random::<usize>() % scored.len();
                scored.into_iter().nth(idx).unwrap().0
            } else {
                // Exploit: best score
                scored.into_iter()
                    .max_by(|a, b| a.1.total().partial_cmp(&b.1.total()).unwrap())
                    .map(|(task, _)| task)
                    .unwrap()
            }
        }
        
        SchedulingStrategy::Thompson => {
            // Sample from posterior over task values
            self.thompson_sampling(scored)
        }
        
        SchedulingStrategy::Curriculum => {
            // Follow pedagogical progression
            self.curriculum_selection(scored)
        }
        
        // ... other strategies
    }
}
```

## Difficulty Calibration

Match task difficulty to learner's zone of proximal development:

```rust
impl AdaptiveScheduler {
    fn compute_difficulty_match(&self, task: &Task) -> f64 {
        // Get learner's current performance level
        let learner_level = self.estimate_learner_level();
        
        // Optimal difficulty is slightly above current level
        let optimal_difficulty = learner_level + 0.1;
        
        // Gaussian centered at optimal difficulty
        let diff = (task.difficulty - optimal_difficulty).abs();
        (-diff * diff / 0.1).exp()
    }
    
    fn estimate_learner_level(&self) -> f64 {
        // Recent performance weighted average
        let recent = self.history.iter().rev().take(10);
        
        let weighted_sum: f64 = recent.enumerate()
            .map(|(i, response)| {
                let weight = 0.9_f64.powi(i as i32);
                let performance = if response.correct { 1.0 } else { 0.0 };
                weight * performance
            })
            .sum();
        
        let weight_sum: f64 = (0..10)
            .map(|i| 0.9_f64.powi(i))
            .sum();
        
        weighted_sum / weight_sum
    }
}
```

## Spaced Repetition

Incorporate forgetting curves for optimal review timing:

```rust
impl AdaptiveScheduler {
    fn compute_review_priority(&self, node: &str) -> f64 {
        let memory = &self.learner.memory_strengths[node];
        let time_since = Utc::now()
            .signed_duration_since(memory.last_practice)
            .num_hours() as f64;
        
        // Predicted memory strength
        let decay_rate = 0.1;
        let predicted_strength = memory.strength * (-decay_rate * time_since).exp();
        
        // Priority increases as strength approaches threshold
        let threshold = 0.3;
        if predicted_strength < threshold {
            1.0 // High priority for review
        } else {
            1.0 - predicted_strength // Lower priority for strong memories
        }
    }
    
    fn schedule_reviews(&mut self) -> Vec<Task> {
        let mut review_tasks = Vec::new();
        
        for (node_id, memory) in &self.learner.memory_strengths {
            let priority = self.compute_review_priority(node_id);
            
            if priority > 0.7 {
                // Generate review task
                let node_label = self.get_label_for_id(node_id);
                review_tasks.push(Task {
                    task_type: TaskType::Successor { 
                        item: node_label.clone() 
                    },
                    difficulty: 0.3, // Easy review
                    // ...
                });
            }
        }
        
        review_tasks
    }
}
```

## Curriculum Learning

Structure learning with pedagogical progressions:

```rust
impl AdaptiveScheduler {
    fn curriculum_selection(&self, scored: Vec<(Task, TaskScore)>) -> Task {
        let stage = self.determine_learning_stage();
        
        match stage {
            LearningStage::Familiarization => {
                // Start with easy, adjacent items
                self.select_familiarization_task(scored)
            }
            
            LearningStage::LocalMastery => {
                // Focus on small segments
                self.select_local_mastery_task(scored)
            }
            
            LearningStage::Integration => {
                // Connect segments, cross boundaries
                self.select_integration_task(scored)
            }
            
            LearningStage::Fluency => {
                // Complex tasks, speed emphasis
                self.select_fluency_task(scored)
            }
        }
    }
    
    fn determine_learning_stage(&self) -> LearningStage {
        let metrics = self.learner.calculate_metrics();
        
        if metrics.total_trials < 20 {
            LearningStage::Familiarization
        } else if metrics.current_mastery < 0.5 {
            LearningStage::LocalMastery
        } else if metrics.current_mastery < 0.8 {
            LearningStage::Integration
        } else {
            LearningStage::Fluency
        }
    }
}
```

## Constraint Application

Ensure pedagogical and practical constraints:

```rust
fn apply_constraints(&self, task: Task) -> Task {
    let mut task = task;
    
    // Avoid frustration: limit consecutive failures
    if self.recent_failure_count() > 2 {
        task.difficulty = (task.difficulty - 0.2).max(0.1);
    }
    
    // Avoid boredom: ensure variety
    if self.is_repetitive(&task) {
        task = self.add_variation(task);
    }
    
    // Respect session limits
    if self.session_duration() > Duration::minutes(20) {
        task.difficulty = (task.difficulty - 0.1).max(0.2);
    }
    
    task
}
```

## Performance Monitoring

Track scheduler effectiveness:

```rust
impl AdaptiveScheduler {
    pub fn evaluate_performance(&self) -> SchedulerMetrics {
        SchedulerMetrics {
            learning_efficiency: self.compute_learning_efficiency(),
            task_diversity: self.compute_task_diversity(),
            difficulty_calibration: self.compute_difficulty_calibration(),
            engagement_score: self.compute_engagement_score(),
        }
    }
    
    fn compute_learning_efficiency(&self) -> f64 {
        // Learning gain per unit time
        if self.history.len() < 2 {
            return 0.0;
        }
        
        let initial_performance = self.history[..10]
            .iter()
            .filter(|r| r.correct)
            .count() as f64 / 10.0;
        
        let recent_performance = self.history[self.history.len()-10..]
            .iter()
            .filter(|r| r.correct)
            .count() as f64 / 10.0;
        
        let improvement = recent_performance - initial_performance;
        let trials = self.history.len() as f64;
        
        improvement / trials.sqrt() // Normalized by sqrt(trials)
    }
}
```

## Real-World Example

Let's trace through a complete scheduling decision:

```rust
// Current state: Learner knows A-F well, struggles with M-R, hasn't seen X-Z

// Step 1: Generate candidates
// Candidates include:
// - Successor tasks for M, N, O (uncertain region)
// - Segment task crossing F-G boundary
// - K-jump task from C (well-known anchor)

// Step 2: Score candidates
// "What comes after N?" scores:
// - EIG: 0.42 (high uncertainty)
// - Difficulty: 0.8 (good match for current level)
// - Recency: 0.2 (N was recent)
// - Diversity: 0.6 (different from recent segments)
// - Pedagogical: 0.7 (builds on M knowledge)

// Step 3: Apply strategy (Thompson sampling)
// Sample from value distributions, "After N" wins

// Step 4: Apply constraints
// No recent failures, no adjustment needed

// Final task: "What comes after N?"
```

## Summary

The adaptive scheduler provides:
- **Information-theoretic selection**: Maximizes learning efficiency
- **Multi-factor optimization**: Balances multiple objectives
- **Pedagogical structure**: Follows learning progressions
- **Personalization**: Adapts to individual patterns
- **Constraint satisfaction**: Maintains engagement and prevents frustration

The scheduler is the conductor of the learning orchestra, ensuring each task contributes optimally to the learner's growth.