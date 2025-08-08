# API Reference

Complete API documentation for the alphabet-terminal-prototype library.

## Core Types

### Topology

```rust
pub struct Topology {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub topology_type: TopologyType,
}

impl Topology {
    /// Create a new topology
    pub fn new(topology_type: TopologyType) -> Self
    
    /// Get a node by label
    pub fn get_node(&self, label: &str) -> Option<&Node>
    
    /// Get successors of a node
    pub fn get_successors(&self, label: &str) -> Vec<String>
    
    /// Get predecessors of a node
    pub fn get_predecessors(&self, label: &str) -> Vec<String>
    
    /// Calculate shortest distance between nodes
    pub fn calculate_distance(&self, from: &str, to: &str) -> Option<usize>
    
    /// Get k steps from a node
    pub fn traverse_k_steps(&self, start: &str, k: i32) -> Vec<String>
}

pub enum TopologyType {
    Linear,
    Cyclic,
    Hierarchical,
    Graph,
}

pub struct Node {
    pub id: String,
    pub label: String,
    pub position: usize,
    pub attributes: HashMap<String, Value>,
}

pub struct Edge {
    pub from: String,
    pub to: String,
    pub weight: f64,
}
```

### LearnerModel

```rust
pub struct LearnerModel {
    pub learner_id: String,
    pub node_embeddings: HashMap<String, LatentNodeEmbedding>,
    pub operation_proficiencies: HashMap<String, OperationProficiency>,
    pub memory_strengths: HashMap<String, MemoryStrength>,
    pub response_time_distribution: ExGaussianParameters,
    pub strategy: StrategyType,
    pub learning_rate: f64,
    pub forgetting_rate: f64,
}

impl LearnerModel {
    /// Create new learner model
    pub fn new(learner_id: impl Into<String>, topology: &Topology) -> Self
    
    /// Update from response
    pub fn update_from_response(&mut self, response: &ResponseData)
    
    /// Update operation proficiency
    pub fn update_operation_proficiency(
        &mut self, 
        operation: &OperationType, 
        success: bool
    )
    
    /// Update memory strength
    pub fn update_memory_strength(&mut self, node_id: &str, correct: bool)
    
    /// Detect strategy from response patterns
    pub fn detect_strategy(&mut self, responses: &[ResponseData])
    
    /// Predict performance on task
    pub fn predict_performance(&self, task: &Task) -> PerformancePrediction
    
    /// Calculate overall metrics
    pub fn calculate_metrics(&self) -> LearnerMetrics
}
```

### Task System

```rust
pub struct Task {
    pub task_type: TaskType,
    pub prompt: String,
    pub correct_answer: String,
    pub options: Vec<String>,
    pub difficulty: f64,
    pub operation: OperationType,
}

pub enum TaskType {
    PairwiseOrder { a: String, b: String },
    Successor { item: String },
    Predecessor { item: String },
    KJump { start: String, k: i32 },
    Segment { start: String, count: usize, reverse: bool },
    Index { item: String },
    MissingItem { before: String, after: String },
    ShortestDistance { from: String, to: String },
    // ... more task types
}

pub struct TaskGenerator {
    pub topology: Topology,
}

impl TaskGenerator {
    /// Create new task generator
    pub fn new(topology: Topology) -> Self
    
    /// Generate a random task
    pub fn generate_task(&mut self, task_type: Option<TaskType>) -> Task
    
    /// Generate task with constraints
    pub fn generate_constrained(&mut self, constraints: TaskConstraints) -> Task
    
    /// Generate batch of tasks
    pub fn generate_batch(&mut self, count: usize) -> Vec<Task>
}
```

### Bayesian Model

```rust
pub struct BayesianLearnerModel {
    pub node_positions: HashMap<String, GaussianPosterior>,
    pub operation_proficiencies: HashMap<String, GaussianPosterior>,
    pub chunk_boundaries: Vec<f64>,
}

impl BayesianLearnerModel {
    /// Create new Bayesian model
    pub fn new(topology: &Topology) -> Self
    
    /// Update posteriors from observation
    pub fn update(&mut self, task: &Task, response: &ResponseData)
    
    /// Compute Expected Information Gain
    pub fn expected_information_gain(&self, task: &Task) -> f64
    
    /// Monte Carlo EIG estimation
    pub fn monte_carlo_eig(&self, task: &Task, n_samples: usize) -> f64
    
    /// Sample from posterior
    pub fn sample_posterior(&self) -> BayesianSample
}

pub struct GaussianPosterior {
    pub mean: f64,
    pub variance: f64,
    pub prior_mean: f64,
    pub prior_variance: f64,
    pub n_observations: usize,
}
```

### Adaptive Scheduler

```rust
pub struct AdaptiveScheduler {
    pub learner: LearnerModel,
    pub bayesian_model: BayesianLearnerModel,
    pub task_generator: TaskGenerator,
    pub history: Vec<TaskResponse>,
}

impl AdaptiveScheduler {
    /// Create new scheduler
    pub fn new(learner: LearnerModel, topology: Topology) -> Self
    
    /// Select next optimal task
    pub fn select_next_task(&mut self) -> Task
    
    /// Update from response
    pub fn update(&mut self, response: TaskResponse)
    
    /// Get session summary
    pub fn get_session_summary(&self) -> SessionSummary
    
    /// Predict remaining time to mastery
    pub fn predict_time_to_mastery(&self) -> Duration
}
```

## Statistical Components

### Ex-Gaussian Model

```rust
pub struct ExGaussianModel {
    pub params: ExGaussianParameters,
}

pub struct ExGaussianParameters {
    pub mu: f64,      // Gaussian mean
    pub sigma: f64,   // Gaussian SD
    pub tau: f64,     // Exponential mean
}

impl ExGaussianModel {
    /// Fit model to data
    pub fn fit(data: &[f64]) -> Self
    
    /// Probability density function
    pub fn pdf(&self, x: f64) -> f64
    
    /// Cumulative distribution function
    pub fn cdf(&self, x: f64) -> f64
    
    /// Generate random sample
    pub fn sample(&self) -> f64
    
    /// Compute percentile
    pub fn percentile(&self, p: f64) -> f64
}
```

### Strategy Detection

```rust
pub enum StrategyType {
    SerialScan,
    DirectAccess,
    Hybrid,
    ChunkBased,
    Associative,
}

pub struct StrategyDetector {
    pub correlation_threshold: f64,
}

impl StrategyDetector {
    /// Detect strategy from responses
    pub fn detect(&self, responses: &[ResponseData]) -> StrategyDetection
    
    /// Compute strategy confidence
    pub fn compute_confidence(&self, responses: &[ResponseData]) -> f64
}
```

## Extended Modules

### Hint System

```rust
pub struct HintGenerator {
    pub hint_levels: Vec<HintLevel>,
}

impl HintGenerator {
    /// Generate hint for struggling learner
    pub fn generate_hint(
        &self, 
        task: &Task, 
        struggle: &StruggleState
    ) -> Hint
    
    /// Check if hint is needed
    pub fn should_provide_hint(&self, history: &[ResponseData]) -> bool
}

pub struct StruggleDetector {
    pub error_threshold: usize,
    pub time_threshold: Duration,
}

impl StruggleDetector {
    /// Detect struggle state
    pub fn detect_struggle(&self, history: &[ResponseData]) -> StruggleState
}
```

### Performance Prediction

```rust
pub struct PerformancePredictor {
    pub model: PredictionModel,
}

impl PerformancePredictor {
    /// Predict future performance
    pub fn predict(
        &self, 
        learner: &LearnerModel, 
        task: &Task, 
        horizon: TimeHorizon
    ) -> Prediction
    
    /// Predict learning trajectory
    pub fn predict_trajectory(
        &self, 
        learner: &LearnerModel, 
        tasks: &[Task]
    ) -> Vec<Prediction>
}
```

### Data Export

```rust
pub struct DataExporter {
    pub format: ExportFormat,
}

impl DataExporter {
    /// Export session data
    pub fn export(&self, session: &SessionData) -> Result<Vec<u8>, ExportError>
    
    /// Export to file
    pub fn export_to_file(
        &self, 
        session: &SessionData, 
        path: &Path
    ) -> Result<(), ExportError>
    
    /// Stream export
    pub async fn export_streaming<W: AsyncWrite>(
        &self,
        sessions: impl Stream<Item = SessionData>,
        writer: W
    ) -> Result<(), ExportError>
}
```

## Usage Examples

### Basic Session

```rust
use alphabet_terminal_prototype::prelude::*;

// Create topology
let topology = Topology::alphabet();

// Create learner
let mut learner = LearnerModel::new("student_001", &topology);

// Create task generator
let mut generator = TaskGenerator::new(topology.clone());

// Create adaptive scheduler
let mut scheduler = AdaptiveScheduler::new(learner, topology);

// Run learning session
for _ in 0..50 {
    // Get next task
    let task = scheduler.select_next_task();
    
    // Present to user and get response
    let response = present_task_to_user(&task);
    
    // Update models
    scheduler.update(response);
}

// Export results
let exporter = DataExporter::new(ExportFormat::JSON);
let data = scheduler.export_session();
exporter.export_to_file(&data, Path::new("session.json"))?;
```

### Custom Task Generation

```rust
// Generate specific task type
let task = generator.generate_task(Some(
    TaskType::KJump { 
        start: "A".to_string(), 
        k: 3 
    }
));

// Generate with constraints
let constraints = TaskConstraints {
    min_difficulty: 0.5,
    max_difficulty: 0.7,
    focus_items: vec!["M".to_string(), "N".to_string()],
    ..Default::default()
};
let task = generator.generate_constrained(constraints);
```

### Bayesian Inference

```rust
// Create Bayesian model
let mut bayes = BayesianLearnerModel::new(&topology);

// Update from response
bayes.update(&task, &response);

// Compute information gain
let eig = bayes.expected_information_gain(&next_task);

// Sample from posterior
let sample = bayes.sample_posterior();
```

### Performance Analysis

```rust
// Create predictor
let predictor = PerformancePredictor::default();

// Predict immediate performance
let prediction = predictor.predict(
    &learner,
    &task,
    TimeHorizon::Immediate
);

// Predict learning trajectory
let trajectory = predictor.predict_trajectory(
    &learner,
    &upcoming_tasks
);

// Analyze strategy
let detector = StrategyDetector::default();
let strategy = detector.detect(&response_history);
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum AlphabetError {
    #[error("Invalid topology: {0}")]
    InvalidTopology(String),
    
    #[error("Task generation failed: {0}")]
    TaskGenerationError(String),
    
    #[error("Model update failed: {0}")]
    ModelUpdateError(String),
    
    #[error("Export failed: {0}")]
    ExportError(String),
}

pub type Result<T> = std::result::Result<T, AlphabetError>;
```

## Feature Flags

```toml
[features]
default = ["cli", "export"]
cli = ["clap", "colored"]
export = ["serde", "csv", "sqlx"]
parallel = ["rayon"]
async = ["tokio", "async-trait"]
visualization = ["plotters"]
```

## Version Compatibility

- Rust: 1.70.0+
- Dependencies: See Cargo.toml
- API Stability: 0.x versions may have breaking changes

## Summary

The API provides:
- **Core types**: Topology, LearnerModel, Task
- **Bayesian inference**: Posterior updates and EIG
- **Adaptive scheduling**: Optimal task selection
- **Statistical models**: Ex-Gaussian, strategy detection
- **Extended features**: Hints, prediction, export
- **Error handling**: Comprehensive error types
- **Examples**: Common usage patterns

This enables flexible integration into various learning applications and research workflows.