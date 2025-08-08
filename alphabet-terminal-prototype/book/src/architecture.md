# System Architecture

This chapter provides a high-level view of how all components fit together to create an adaptive learning system.

## Overall Design

The system follows a modular, layered architecture:

```
┌─────────────────────────────────────────┐
│           User Interface Layer          │
│  (Terminal UI / Web UI / Xilem GUI)     │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│         Application Logic Layer         │
│   (Adaptive Scheduler, Task Manager)    │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│          Core Modeling Layer           │
│  (Bayesian Model, Learner Model)       │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│         Statistical Foundation          │
│  (Ex-Gaussian, KL Divergence, EIG)     │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│            Data Layer                   │
│    (Topology, Response Storage)         │
└─────────────────────────────────────────┘
```

## Core Components

### Topology (topology.rs)

Defines the structure being learned:

```rust
pub struct Topology {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub topology_type: TopologyType,
}
```

**Responsibilities:**
- Define learning domain structure
- Provide navigation methods
- Calculate distances
- Support multiple topology types

### Learner Model (learner.rs)

Represents individual knowledge state:

```rust
pub struct LearnerModel {
    pub learner_id: String,
    pub node_embeddings: HashMap<String, LatentNodeEmbedding>,
    pub operation_proficiencies: HashMap<String, OperationProficiency>,
    pub memory_strengths: HashMap<String, MemoryStrength>,
    // ...
}
```

**Responsibilities:**
- Track knowledge state
- Update from responses
- Predict performance
- Detect strategies

### Bayesian Model (bayesian.rs)

Probabilistic inference engine:

```rust
pub struct BayesianLearnerModel {
    pub node_positions: HashMap<String, GaussianPosterior>,
    pub operation_proficiencies: HashMap<String, GaussianPosterior>,
    pub chunk_boundaries: Vec<f64>,
    // ...
}
```

**Responsibilities:**
- Maintain probability distributions
- Compute Expected Information Gain
- Update posteriors from evidence
- Sample for Monte Carlo methods

### Task System (tasks.rs)

Task generation and management:

```rust
pub struct Task {
    pub task_type: TaskType,
    pub prompt: String,
    pub correct_answer: String,
    pub options: Vec<String>,
    pub difficulty: f64,
    pub operation: OperationType,
}
```

**Responsibilities:**
- Generate diverse task types
- Format prompts and options
- Track correct answers
- Assign difficulty levels

### Adaptive Scheduler (adaptive.rs)

Orchestrates learning experience:

```rust
pub struct AdaptiveScheduler {
    pub learner: LearnerModel,
    pub bayesian_model: BayesianLearnerModel,
    pub task_generator: TaskGenerator,
    pub history: Vec<TaskResponse>,
}
```

**Responsibilities:**
- Select optimal tasks
- Balance exploration/exploitation
- Apply pedagogical constraints
- Track learning history

### Statistical Components (statistics.rs)

Mathematical and statistical tools:

```rust
pub struct ExGaussianModel { /* ... */ }
pub struct StrategyAnalysis { /* ... */ }
pub struct PerformanceAnalysis { /* ... */ }
```

**Responsibilities:**
- Fit response time distributions
- Detect learning strategies
- Analyze performance patterns
- Provide statistical tests

## Data Flow

### Task Generation Flow

```
1. Scheduler requests candidates
   ↓
2. TaskGenerator creates diverse tasks
   ↓
3. BayesianModel computes EIG for each
   ↓
4. Scheduler scores and ranks tasks
   ↓
5. Top task presented to user
```

### Response Processing Flow

```
1. User completes task
   ↓
2. Response captured (correct/incorrect, RT)
   ↓
3. LearnerModel updates proficiencies
   ↓
4. BayesianModel updates posteriors
   ↓
5. Statistics updated (RT distribution, strategy)
   ↓
6. History logged for analysis
```

## Module Organization

The crate is organized into logical modules:

```rust
// lib.rs - Public API
pub mod topology;      // Domain structures
pub mod learner;       // Individual models
pub mod tasks;         // Task generation
pub mod adaptive;      // Scheduling
pub mod statistics;    // Statistical tools
pub mod bayesian;      // Bayesian inference

// Extended functionality
pub mod extended_tasks;    // Complex task types
pub mod music;            // Music domain
pub mod navigation;       // Path finding
pub mod boundaries;       // Chunk detection
pub mod prediction;       // Performance prediction
pub mod hints;           // Intervention system
pub mod macro_learning;   // Abstraction discovery
pub mod transfer_learning; // Cross-domain transfer
pub mod hierarchical_bayes; // Population modeling
pub mod experiments;      // Experiment framework

// Utilities
pub mod export;           // Data export
pub mod demo;            // Demo scenarios
pub mod ui;              // Terminal interface
```

## Trait Design

Key traits enable extensibility:

```rust
// Topology navigation
pub trait Navigable {
    fn get_successor(&self, item: &str) -> Option<String>;
    fn get_predecessor(&self, item: &str) -> Option<String>;
    fn calculate_distance(&self, from: &str, to: &str) -> usize;
}

// Learning updates
pub trait Learnable {
    fn update_from_response(&mut self, response: ResponseData);
    fn predict_performance(&self, task: &Task) -> f64;
}

// Task generation
pub trait TaskGeneratorTrait {
    fn generate_task(&mut self, constraints: Option<TaskConstraints>) -> Task;
    fn generate_batch(&mut self, n: usize) -> Vec<Task>;
}
```

## Concurrency Model

The system supports parallel computation:

```rust
use rayon::prelude::*;

impl BayesianLearnerModel {
    pub fn parallel_eig_computation(&self, tasks: &[Task]) -> Vec<f64> {
        tasks.par_iter()
            .map(|task| self.monte_carlo_eig(task, 1000))
            .collect()
    }
}
```

## Error Handling

Comprehensive error handling throughout:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Invalid topology: {0}")]
    InvalidTopology(String),
    
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #[error("Numerical error: {0}")]
    NumericalError(String),
    
    #[error("Invalid task configuration: {0}")]
    InvalidTask(String),
}

pub type Result<T> = std::result::Result<T, ModelError>;
```

## Configuration

System behavior is configurable:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub monte_carlo_samples: usize,
    pub learning_rate: f64,
    pub forgetting_rate: f64,
    pub min_task_difficulty: f64,
    pub max_task_difficulty: f64,
    pub exploration_epsilon: f64,
    pub session_duration_minutes: u32,
}

impl Default for SystemConfig {
    fn default() -> Self {
        SystemConfig {
            monte_carlo_samples: 1000,
            learning_rate: 0.1,
            forgetting_rate: 0.01,
            min_task_difficulty: 0.1,
            max_task_difficulty: 0.9,
            exploration_epsilon: 0.1,
            session_duration_minutes: 20,
        }
    }
}
```

## Testing Strategy

Comprehensive test coverage:

```rust
#[cfg(test)]
mod tests {
    // Unit tests for individual components
    mod unit {
        mod topology_tests;
        mod learner_tests;
        mod bayesian_tests;
        mod statistics_tests;
    }
    
    // Integration tests for workflows
    mod integration {
        mod learning_workflow;
        mod adaptive_scheduling;
        mod data_export;
    }
    
    // Property-based tests
    mod properties {
        use quickcheck::quickcheck;
        // Test invariants
    }
    
    // Benchmarks
    mod bench {
        use criterion::criterion_group;
        // Performance tests
    }
}
```

## Extension Points

The architecture supports extensions:

### Adding New Domains

```rust
impl Topology {
    pub fn custom_domain(structure: CustomStructure) -> Self {
        // Define nodes and edges
        // Set topology type
        // Return new topology
    }
}
```

### Custom Task Types

```rust
pub enum CustomTaskType {
    MyNewTask { /* ... */ }
}

impl TaskGenerator {
    pub fn generate_custom(&mut self) -> Task {
        // Generate custom task
    }
}
```

### Alternative Learning Models

```rust
pub trait LearningModel {
    fn update(&mut self, response: ResponseData);
    fn predict(&self, task: &Task) -> Prediction;
}

struct NeuralLearnerModel { /* ... */ }
impl LearningModel for NeuralLearnerModel { /* ... */ }
```

## Performance Considerations

Key optimizations:

1. **Lazy Evaluation**: Compute expensive values only when needed
2. **Caching**: Store computed EIG values
3. **Parallelization**: Use Rayon for independent computations
4. **Memory Pooling**: Reuse allocations in hot paths
5. **SIMD**: Use packed operations for vector math

## Summary

The architecture provides:
- **Modularity**: Clear separation of concerns
- **Extensibility**: Easy to add new features
- **Testability**: Each component independently testable
- **Performance**: Optimized for computational efficiency
- **Maintainability**: Clear interfaces and documentation

This modular design enables the system to evolve while maintaining stability and performance.