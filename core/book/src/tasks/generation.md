# Task Generation

Task generation is the heart of assessment in ABCDeez Core. The system provides sophisticated algorithms for creating diverse, calibrated tasks that adapt to learner abilities and maximize information gain.

## Core Concepts

### What is a Task?

In ABCDeez Core, a task represents any cognitive challenge presented to a learner:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_type: TaskType,
    pub prompt: String,
    pub correct_answer: String,
    pub options: Vec<String>,      // For multiple choice
    pub difficulty: f64,           // [0, 1] calibrated difficulty
    pub operation: OperationType,  // Cognitive operation required
}
```

### Task Types

The system supports a rich variety of task types:

```rust
pub enum TaskType {
    // Sequential tasks
    Successor { item: String },
    Predecessor { item: String },
    KJump { start: String, k: i32 },
    
    // Comparison tasks
    PairwiseOrder { a: String, b: String },
    Comparability { a: String, b: String },
    
    // Segment tasks
    Segment { 
        start: String, 
        count: usize, 
        reverse: bool 
    },
    
    // Structure tasks
    MissingItem { before: String, after: String },
    Index { item: String },
    
    // Graph tasks
    ShortestDistance { from: String, to: String },
    ShortestPath { from: String, to: String },
    TopologicalSort { items: Vec<String> },
    MinimalElements,
    MaximalElements,
}
```

## Task Generator

### Basic Usage

```rust
use abcdeez_core::tasks::{TaskGenerator, TaskType};
use abcdeez_core::core::topology::Topology;

// Create generator with topology
let topology = Topology::alphabet();
let mut generator = TaskGenerator::new(topology);

// Generate random task
let task = generator.generate_task(None);

// Generate specific task type
let successor_task = generator.generate_task(
    Some(TaskType::Successor { item: "M".to_string() })
);
```

### Seeded Generation

For reproducible experiments:

```rust
// Create generator with seed
let mut generator = TaskGenerator::with_seed(topology, Some(42));

// Tasks will be generated deterministically
let task1 = generator.generate_task(None);
let task2 = generator.generate_task(None);
// Same seed = same sequence
```

## Task Generation Algorithms

### 1. Successor/Predecessor Tasks

Tests knowledge of sequential relationships:

```rust
fn generate_successor(&mut self, item: String) -> Task {
    let prompt = format!("What comes after '{}'?", item);
    let node = self.topology.get_node_by_label(&item).unwrap();
    let successor_id = self.topology.get_successor(&node.id);
    
    let correct_answer = successor_id
        .and_then(|id| self.topology.get_node_by_id(&id))
        .map(|n| n.label.clone())
        .unwrap_or("None".to_string());
    
    // Generate plausible distractors
    let mut options = self.generate_options(&correct_answer, 4);
    options.shuffle(&mut self.rng);
    
    Task {
        task_type: TaskType::Successor { item },
        prompt,
        correct_answer,
        options,
        difficulty: 0.3,  // Base difficulty
        operation: OperationType::Successor,
    }
}
```

### 2. K-Jump Tasks

Tests ability to skip through sequences:

```rust
fn generate_k_jump(&mut self, start: String, k: i32) -> Task {
    let direction = if k > 0 { "after" } else { "before" };
    let prompt = format!(
        "What is {} positions {} '{}'?", 
        k.abs(), direction, start
    );
    
    // Calculate target
    let target = self.topology.get_k_jump(&start, k);
    
    // Difficulty increases with jump size
    let difficulty = 0.3 + (k.abs() as f64 * 0.1).min(0.7);
    
    Task {
        task_type: TaskType::KJump { start, k },
        prompt,
        correct_answer: target.unwrap_or("Out of bounds".to_string()),
        options: generate_k_jump_options(&target, k),
        difficulty,
        operation: OperationType::KJump(k),
    }
}
```

### 3. Segment Recital Tasks

Tests chunk-based memory and sequential recall:

```rust
fn generate_segment(&mut self, start: String, count: usize, reverse: bool) -> Task {
    // Enhanced segment task with multiple formats
    let recital_type = self.rng.gen_range(0..3);
    
    let (prompt, correct_answer, difficulty_bonus) = match recital_type {
        0 => {
            // Standard: from starting point
            let direction = if reverse { "reverse" } else { "forward" };
            let prompt = format!(
                "List {} items starting from '{}' in {} order:",
                count, start, direction
            );
            let segment = self.topology.get_segment(&start, count, reverse);
            (prompt, segment.join(", "), 0.0)
        }
        1 if count <= 4 => {
            // Preceding items
            let prompt = format!(
                "List the {} items that come before '{}':",
                count, start
            );
            let items = get_preceding_items(&start, count);
            (prompt, items.join(", "), 0.1)
        }
        _ => {
            // Following items
            let prompt = format!(
                "What are the next {} items after '{}'?",
                count, start
            );
            let items = get_following_items(&start, count);
            (prompt, items.join(", "), 0.05)
        }
    };
    
    let difficulty = 0.3 
        + (count as f64 * 0.1)           // More items = harder
        + if reverse { 0.2 } else { 0.0 } // Reverse is harder
        + difficulty_bonus;
    
    Task {
        task_type: TaskType::Segment { start, count, reverse },
        prompt,
        correct_answer,
        options: vec![],  // Free response
        difficulty,
        operation: OperationType::Segment(count, reverse),
    }
}
```

### 4. Pairwise Order Tasks

Tests relative positioning knowledge:

```rust
fn generate_pairwise_order(&self, a: String, b: String) -> Task {
    let prompt = format!("Does '{}' come before '{}'?", a, b);
    
    let correct_answer = match self.topology.is_before(&a, &b) {
        Some(true) => "Yes",
        Some(false) => "No",
        None => "Not applicable",  // For cyclic structures
    };
    
    // Difficulty based on distance
    let distance = self.topology.get_distance(&a, &b).unwrap_or(0);
    let difficulty = (distance as f64 / 10.0).min(1.0);
    
    Task {
        task_type: TaskType::PairwiseOrder { a, b },
        prompt,
        correct_answer: correct_answer.to_string(),
        options: vec!["Yes".to_string(), "No".to_string()],
        difficulty,
        operation: OperationType::PairwiseOrder,
    }
}
```

## Difficulty Calibration

### Factors Affecting Difficulty

1. **Distance**: Items farther apart are harder to compare
2. **Direction**: Backward tasks are typically harder
3. **Chunk boundaries**: Crossing chunks increases difficulty
4. **Working memory load**: More items = higher difficulty
5. **Interference**: Similar items increase difficulty

### Dynamic Difficulty Adjustment

```rust
impl TaskGenerator {
    pub fn calibrate_difficulty(&self, task: &mut Task, learner: &LearnerModel) {
        // Base difficulty from task properties
        let mut difficulty = task.difficulty;
        
        // Adjust for individual proficiency
        let proficiency = learner.get_operation_proficiency(&task.operation);
        difficulty *= (2.0 - proficiency).max(0.5);
        
        // Adjust for memory strength
        if let Some(item) = task.get_primary_item() {
            let memory = learner.get_memory_strength(&item);
            difficulty *= (2.0 - memory).max(0.5);
        }
        
        // Account for chunk boundaries
        if task.crosses_chunk_boundary() {
            difficulty *= 1.3;
        }
        
        task.difficulty = difficulty.min(1.0);
    }
}
```

## Distractor Generation

### Principles of Good Distractors

1. **Plausibility**: Must seem reasonable
2. **Common errors**: Based on typical mistakes
3. **Semantic similarity**: Related but incorrect
4. **Systematic errors**: Off-by-one, transpositions

### Implementation

```rust
fn generate_options(&mut self, correct: &str, count: usize) -> Vec<String> {
    let mut options = vec![correct.to_string()];
    
    // Get semantically similar items
    let similar = self.get_similar_items(correct);
    
    // Add common error patterns
    if let Some(node) = self.topology.get_node_by_label(correct) {
        // Off-by-one errors
        if let Some(pred) = self.topology.get_predecessor(&node.id) {
            options.push(pred.label);
        }
        if let Some(succ) = self.topology.get_successor(&node.id) {
            options.push(succ.label);
        }
        
        // Transposition errors (for multi-character items)
        if correct.len() > 1 {
            options.push(transpose_characters(correct));
        }
    }
    
    // Fill remaining with random items
    while options.len() < count {
        let random = self.topology.nodes
            .choose(&mut self.rng)
            .map(|n| n.label.clone());
        if let Some(item) = random {
            if !options.contains(&item) {
                options.push(item);
            }
        }
    }
    
    options.truncate(count);
    options
}
```

## Adaptive Task Selection

### Information-Theoretic Selection

Select tasks that maximize expected information gain:

```rust
pub fn select_optimal_task(
    &mut self,
    learner: &LearnerModel,
    bayesian: &BayesianLearnerModel,
    candidates: Vec<TaskType>,
) -> Task {
    let mut best_task = None;
    let mut best_eig = 0.0;
    
    for task_type in candidates {
        let task = self.generate_task(Some(task_type));
        let eig = bayesian.monte_carlo_eig(&task, 100);
        
        if eig > best_eig {
            best_eig = eig;
            best_task = Some(task);
        }
    }
    
    best_task.unwrap_or_else(|| self.generate_task(None))
}
```

### Constraint-Based Selection

Balance multiple objectives:

```rust
pub struct TaskConstraints {
    pub min_difficulty: f64,
    pub max_difficulty: f64,
    pub avoid_recent: Duration,
    pub require_operation: Option<OperationType>,
    pub prefer_weak_items: bool,
}

impl TaskGenerator {
    pub fn generate_constrained(
        &mut self,
        constraints: TaskConstraints,
        learner: &LearnerModel,
    ) -> Task {
        let candidates = self.filter_by_constraints(constraints, learner);
        
        if constraints.prefer_weak_items {
            // Focus on items with low memory strength
            self.select_weak_item_task(candidates, learner)
        } else {
            // Random selection from valid candidates
            self.generate_task(candidates.choose(&mut self.rng))
        }
    }
}
```

## Task Sessions

### Managing Assessment Sessions

```rust
pub struct TaskSession {
    pub generator: TaskGenerator,
    pub current_task: Option<Task>,
    pub task_start_time: Option<Instant>,
    pub history: Vec<TaskResponse>,
}

impl TaskSession {
    pub fn start_task(&mut self, task_type: Option<TaskType>) -> &Task {
        let task = self.generator.generate_task(task_type);
        self.current_task = Some(task);
        self.task_start_time = Some(Instant::now());
        self.current_task.as_ref().unwrap()
    }
    
    pub fn submit_answer(&mut self, answer: String) -> TaskResponse {
        let task = self.current_task.take().expect("No active task");
        let start_time = self.task_start_time.take().expect("No start time");
        
        let response_time_ms = start_time.elapsed().as_millis();
        let correct = answer.trim().eq_ignore_ascii_case(&task.correct_answer);
        
        let response = TaskResponse {
            task: task.clone(),
            user_answer: answer,
            correct,
            response_time_ms,
            timestamp: chrono::Utc::now(),
        };
        
        self.history.push(response.clone());
        response
    }
}
```

## Special Task Types

### Graph-Based Tasks

For DAGs and general graphs:

```rust
// Find minimal elements (no prerequisites)
fn generate_minimal_elements(&self) -> Task {
    let minimal = self.topology.nodes.iter()
        .filter(|n| !self.topology.edges.iter().any(|e| e.to == n.id))
        .map(|n| n.label.clone())
        .collect::<Vec<_>>();
    
    Task {
        task_type: TaskType::MinimalElements,
        prompt: "Which elements have no prerequisites?".to_string(),
        correct_answer: minimal.join(", "),
        options: vec![],
        difficulty: 0.5,
        operation: OperationType::PairwiseOrder,
    }
}

// Topological sorting
fn generate_topological_sort(&self, items: Vec<String>) -> Task {
    let correct_order = self.topology.get_topological_sort()
        .map(|sorted| {
            sorted.into_iter()
                .filter(|s| items.contains(s))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or("Not applicable".to_string());
    
    Task {
        task_type: TaskType::TopologicalSort { items },
        prompt: "Arrange these items in a valid order:".to_string(),
        correct_answer: correct_order,
        options: vec![],
        difficulty: 0.8,
        operation: OperationType::PairwiseOrder,
    }
}
```

### Music-Based Tasks

For musical structures:

```rust
pub struct MusicTaskGenerator {
    structure: MusicStructure,
    theory: MusicTheory,
}

impl MusicTaskGenerator {
    pub fn generate_interval_task(&mut self) -> Task {
        let note1 = self.random_note();
        let interval = self.random_interval();
        
        Task {
            prompt: format!("What note is a {} above {}?", interval, note1),
            correct_answer: self.theory.apply_interval(note1, interval),
            // ... other fields
        }
    }
    
    pub fn generate_chord_task(&mut self) -> Task {
        let root = self.random_note();
        let chord_type = self.random_chord_type();
        
        Task {
            prompt: format!("What notes make up a {} {} chord?", root, chord_type),
            correct_answer: self.theory.build_chord(root, chord_type).join(", "),
            // ... other fields
        }
    }
}
```

## Performance Optimization

### Caching Generated Tasks

```rust
pub struct CachedTaskGenerator {
    generator: TaskGenerator,
    cache: LruCache<TaskType, Vec<Task>>,
    cache_size: usize,
}

impl CachedTaskGenerator {
    pub fn generate(&mut self, task_type: TaskType) -> Task {
        if let Some(cached) = self.cache.get_mut(&task_type) {
            if !cached.is_empty() {
                return cached.pop().unwrap();
            }
        }
        
        // Generate batch for efficiency
        let mut tasks = Vec::new();
        for _ in 0..self.cache_size {
            tasks.push(self.generator.generate_task(Some(task_type.clone())));
        }
        
        let task = tasks.pop().unwrap();
        self.cache.put(task_type, tasks);
        task
    }
}
```

### Parallel Generation

```rust
use rayon::prelude::*;

pub fn generate_task_bank(
    topology: &Topology,
    task_types: Vec<TaskType>,
    count_per_type: usize,
) -> HashMap<TaskType, Vec<Task>> {
    task_types
        .par_iter()
        .map(|task_type| {
            let mut generator = TaskGenerator::new(topology.clone());
            let tasks = (0..count_per_type)
                .map(|_| generator.generate_task(Some(task_type.clone())))
                .collect();
            (task_type.clone(), tasks)
        })
        .collect()
}
```

## Best Practices

1. **Variety**: Mix task types to prevent strategy fixation
2. **Calibration**: Regularly recalibrate difficulty based on performance
3. **Spacing**: Don't repeat identical tasks too soon
4. **Adaptivity**: Adjust to individual learning patterns
5. **Validation**: Test generated tasks for correctness
6. **Accessibility**: Provide alternative formats when needed

## Testing Task Generation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_generation_deterministic() {
        let topo = Topology::alphabet();
        let mut gen1 = TaskGenerator::with_seed(topo.clone(), Some(42));
        let mut gen2 = TaskGenerator::with_seed(topo.clone(), Some(42));
        
        for _ in 0..10 {
            let task1 = gen1.generate_task(None);
            let task2 = gen2.generate_task(None);
            assert_eq!(task1.prompt, task2.prompt);
            assert_eq!(task1.correct_answer, task2.correct_answer);
        }
    }
    
    #[test]
    fn test_difficulty_bounds() {
        let topo = Topology::alphabet();
        let mut gen = TaskGenerator::new(topo);
        
        for _ in 0..100 {
            let task = gen.generate_task(None);
            assert!(task.difficulty >= 0.0);
            assert!(task.difficulty <= 1.0);
        }
    }
}
```

## Next Steps

- Learn about [Task Types](./types.md) in detail
- Explore [Difficulty Calibration](./difficulty.md)
- Understand [Adaptive Selection](./adaptive.md)
- See [Task API Reference](../api/tasks.md)