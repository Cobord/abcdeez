# Task Generation

Task generation is the engine that creates diverse learning experiences. The system generates tasks that probe different aspects of knowledge while maintaining appropriate difficulty levels and pedagogical value.

## Task Taxonomy

The system supports a rich taxonomy of task types:

```rust
#[derive(Debug, Clone)]
pub enum TaskType {
    // Basic ordering tasks
    PairwiseOrder { a: String, b: String },
    Successor { item: String },
    Predecessor { item: String },
    
    // Distance-based tasks
    KJump { start: String, k: i32 },
    ShortestDistance { from: String, to: String },
    
    // Segment tasks
    Segment { start: String, count: usize, reverse: bool },
    
    // Position tasks
    Index { item: String },
    MissingItem { before: String, after: String },
    
    // Graph-theoretic tasks
    Comparability { a: String, b: String },
    TopologicalSort { items: Vec<String> },
    ShortestPath { from: String, to: String },
    MinimalElements,
    MaximalElements,
}
```

## Task Generator Architecture

```rust
pub struct TaskGenerator {
    pub topology: Topology,
    rng: ThreadRng,
    difficulty_model: DifficultyModel,
    constraint_engine: ConstraintEngine,
}
```

### Core Generation Pipeline

```rust
impl TaskGenerator {
    pub fn generate_task(&mut self, constraints: TaskConstraints) -> Task {
        // 1. Select task type based on constraints
        let task_type = self.select_task_type(&constraints);
        
        // 2. Choose items that satisfy constraints
        let items = self.select_items(&task_type, &constraints);
        
        // 3. Generate distractors for multiple choice
        let options = self.generate_options(&task_type, &items);
        
        // 4. Compute difficulty
        let difficulty = self.compute_difficulty(&task_type, &items);
        
        // 5. Format prompt
        let prompt = self.format_prompt(&task_type, &items);
        
        Task {
            task_type,
            prompt,
            correct_answer: self.compute_answer(&task_type, &items),
            options,
            difficulty,
            operation: self.map_to_operation(&task_type),
        }
    }
}
```

## Task Type Implementations

### Successor/Predecessor Tasks

The most fundamental tasks probe adjacent relationships:

```rust
fn generate_successor(&mut self, item: String) -> Task {
    let node = self.topology.get_node(&item).unwrap();
    let successors = self.topology.get_successors(&item);
    
    let correct_answer = if successors.is_empty() {
        "None".to_string()
    } else {
        successors[0].clone() // Assume single successor for simplicity
    };
    
    // Generate plausible distractors
    let distractors = self.generate_successor_distractors(&item, 3);
    let mut options = distractors;
    options.push(correct_answer.clone());
    options.shuffle(&mut self.rng);
    
    Task {
        task_type: TaskType::Successor { item: item.clone() },
        prompt: format!("What comes after {}?", item),
        correct_answer,
        options,
        difficulty: self.compute_successor_difficulty(&item),
        operation: OperationType::Successor,
    }
}

fn generate_successor_distractors(&self, item: &str, count: usize) -> Vec<String> {
    let mut distractors = Vec::new();
    let item_pos = self.topology.get_position(item);
    
    // Strategy 1: Items at similar positions
    let similar_positions = self.topology.nodes.iter()
        .filter(|n| {
            let pos = self.topology.get_position(&n.label);
            (pos - item_pos).abs() < 3.0 && n.label != item
        })
        .map(|n| n.label.clone())
        .collect::<Vec<_>>();
    
    // Strategy 2: Common confusions from history
    if let Some(confusions) = self.get_common_confusions(item) {
        distractors.extend(confusions.iter().take(count/2).cloned());
    }
    
    // Strategy 3: Random items
    distractors.extend(similar_positions.choose_multiple(&mut self.rng, count - distractors.len()).cloned());
    
    distractors
}
```

### K-Jump Tasks

Tasks that require mental traversal:

```rust
fn generate_k_jump(&mut self, start: String, k: i32) -> Task {
    let path = self.topology.traverse_k_steps(&start, k);
    let correct_answer = path.last().cloned().unwrap_or("None".to_string());
    
    // Difficulty increases with k and boundary crossings
    let difficulty = self.compute_kjump_difficulty(&start, k);
    
    // Generate distractors at various distances
    let distractors: Vec<String> = vec![
        self.topology.traverse_k_steps(&start, k - 1).last().cloned(),
        self.topology.traverse_k_steps(&start, k + 1).last().cloned(),
        self.topology.traverse_k_steps(&start, k * 2).last().cloned(),
    ].into_iter().flatten().collect();
    
    Task {
        task_type: TaskType::KJump { start: start.clone(), k },
        prompt: format!("What is {} steps after {}?", k, start),
        correct_answer,
        options: self.shuffle_with_correct(correct_answer.clone(), distractors),
        difficulty,
        operation: OperationType::KJump(k),
    }
}
```

### Segment Tasks

Probe working memory and sequential processing:

```rust
fn generate_segment(&mut self, start: String, count: usize, reverse: bool) -> Task {
    let sequence = self.topology.get_sequence_from(&start, count);
    
    let correct_answer = if reverse {
        sequence.iter().rev().cloned().collect::<Vec<_>>().join(", ")
    } else {
        sequence.join(", ")
    };
    
    // Generate plausible incorrect sequences
    let mut distractors = Vec::new();
    
    // Missing one item
    if sequence.len() > 1 {
        let mut missing = sequence.clone();
        missing.remove(self.rng.gen_range(1..missing.len()));
        distractors.push(missing.join(", "));
    }
    
    // Swapped adjacent items
    if sequence.len() > 1 {
        let mut swapped = sequence.clone();
        let i = self.rng.gen_range(0..sequence.len()-1);
        swapped.swap(i, i + 1);
        distractors.push(swapped.join(", "));
    }
    
    // Off-by-one start
    if let Some(prev) = self.topology.get_predecessor(&start) {
        let alt_sequence = self.topology.get_sequence_from(&prev, count);
        distractors.push(alt_sequence.join(", "));
    }
    
    Task {
        task_type: TaskType::Segment { start, count, reverse },
        prompt: if reverse {
            format!("List {} items backwards starting from {}", count, start)
        } else {
            format!("List {} items starting from {}", count, start)
        },
        correct_answer,
        options: self.shuffle_with_correct(correct_answer.clone(), distractors),
        difficulty: self.compute_segment_difficulty(count, reverse),
        operation: OperationType::Segment(count, reverse),
    }
}
```

## Difficulty Computation

Task difficulty is computed based on multiple factors:

```rust
impl TaskGenerator {
    fn compute_difficulty(&self, task_type: &TaskType, items: &[String]) -> f64 {
        let base_difficulty = self.get_base_difficulty(task_type);
        let item_difficulty = self.compute_item_difficulty(items);
        let structural_difficulty = self.compute_structural_difficulty(task_type, items);
        
        // Weighted combination
        0.3 * base_difficulty + 
        0.4 * item_difficulty + 
        0.3 * structural_difficulty
    }
    
    fn compute_item_difficulty(&self, items: &[String]) -> f64 {
        // Items later in sequence are harder
        let position_difficulty: f64 = items.iter()
            .map(|item| {
                let pos = self.topology.get_position(item);
                pos / self.topology.nodes.len() as f64
            })
            .sum::<f64>() / items.len() as f64;
        
        // Items near boundaries are harder
        let boundary_difficulty = self.compute_boundary_proximity(items);
        
        (position_difficulty + boundary_difficulty) / 2.0
    }
    
    fn compute_structural_difficulty(&self, task_type: &TaskType, items: &[String]) -> f64 {
        match task_type {
            TaskType::KJump { k, .. } => {
                // Difficulty increases with k
                (*k as f64 / 10.0).min(1.0)
            }
            TaskType::Segment { count, reverse, .. } => {
                // Difficulty increases with length and reversal
                let length_difficulty = (*count as f64 / 7.0).min(1.0);
                let reverse_penalty = if *reverse { 0.2 } else { 0.0 };
                length_difficulty + reverse_penalty
            }
            TaskType::ShortestPath { from, to } => {
                // Difficulty increases with path length
                let distance = self.topology.calculate_distance(from, to);
                (distance as f64 / 20.0).min(1.0)
            }
            _ => 0.5 // Default medium difficulty
        }
    }
}
```

## Constraint System

Tasks can be generated with specific constraints:

```rust
#[derive(Debug, Clone)]
pub struct TaskConstraints {
    pub min_difficulty: f64,
    pub max_difficulty: f64,
    pub avoid_items: Vec<String>,
    pub focus_items: Vec<String>,
    pub task_types: Vec<TaskType>,
    pub require_boundary_crossing: bool,
    pub max_working_memory_load: usize,
}

impl TaskGenerator {
    pub fn generate_constrained(&mut self, constraints: TaskConstraints) -> Task {
        let max_attempts = 100;
        
        for _ in 0..max_attempts {
            let task = self.generate_task(None);
            
            if self.satisfies_constraints(&task, &constraints) {
                return task;
            }
        }
        
        // Fallback: relax constraints
        self.generate_with_relaxed_constraints(constraints)
    }
    
    fn satisfies_constraints(&self, task: &Task, constraints: &TaskConstraints) -> bool {
        // Check difficulty range
        if task.difficulty < constraints.min_difficulty ||
           task.difficulty > constraints.max_difficulty {
            return false;
        }
        
        // Check item constraints
        let task_items = self.extract_items_from_task(task);
        if !constraints.avoid_items.is_empty() {
            for item in &task_items {
                if constraints.avoid_items.contains(item) {
                    return false;
                }
            }
        }
        
        // Check focus items
        if !constraints.focus_items.is_empty() {
            let has_focus = task_items.iter()
                .any(|item| constraints.focus_items.contains(item));
            if !has_focus {
                return false;
            }
        }
        
        // Check boundary crossing
        if constraints.require_boundary_crossing {
            if !self.crosses_boundary(task) {
                return false;
            }
        }
        
        true
    }
}
```

## Adaptive Task Selection

The generator adapts based on learner history:

```rust
impl TaskGenerator {
    pub fn generate_adaptive(&mut self, learner: &LearnerModel) -> Task {
        // Identify areas of uncertainty
        let uncertain_items = learner.get_most_uncertain_items(5);
        
        // Identify struggling operations
        let weak_operations = learner.get_weakest_operations(3);
        
        // Create constraints based on learner state
        let constraints = TaskConstraints {
            min_difficulty: learner.get_optimal_difficulty() - 0.1,
            max_difficulty: learner.get_optimal_difficulty() + 0.1,
            focus_items: uncertain_items,
            task_types: self.map_operations_to_types(weak_operations),
            ..Default::default()
        };
        
        self.generate_constrained(constraints)
    }
}
```

## Extended Task Types

Beyond basic tasks, the system supports complex scenarios:

```rust
// Pattern completion
pub fn generate_pattern_task(&mut self) -> Task {
    let pattern = self.generate_pattern();
    let missing_idx = self.rng.gen_range(0..pattern.len());
    
    let mut prompt_pattern = pattern.clone();
    prompt_pattern[missing_idx] = "?".to_string();
    
    Task {
        prompt: format!("Complete the pattern: {}", prompt_pattern.join(" ")),
        correct_answer: pattern[missing_idx].clone(),
        // ...
    }
}

// Relational reasoning
pub fn generate_analogy_task(&mut self) -> Task {
    // A is to B as C is to ?
    let (a, b) = self.select_related_pair();
    let c = self.select_similar_to(&a);
    let d = self.find_analogous(&a, &b, &c);
    
    Task {
        prompt: format!("{} is to {} as {} is to ?", a, b, c),
        correct_answer: d,
        // ...
    }
}
```

## Performance Optimization

Task generation is optimized for speed:

```rust
impl TaskGenerator {
    pub fn precompute_task_cache(&mut self, count: usize) -> Vec<Task> {
        (0..count)
            .into_par_iter()
            .map(|_| {
                let mut local_gen = self.clone();
                local_gen.generate_task(None)
            })
            .collect()
    }
    
    pub fn batch_generate(&mut self, specs: Vec<TaskSpec>) -> Vec<Task> {
        specs.into_par_iter()
            .map(|spec| self.generate_from_spec(spec))
            .collect()
    }
}
```

## Summary

Task generation provides:
- **Diverse task types**: From simple ordering to complex reasoning
- **Adaptive difficulty**: Calibrated to learner ability
- **Constraint satisfaction**: Generate tasks meeting specific criteria
- **Pedagogical alignment**: Tasks that promote learning
- **Performance optimization**: Fast generation for real-time adaptation

The task generator is the creative force that keeps learning engaging and effective.