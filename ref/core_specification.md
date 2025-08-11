# Core Module Specification

## Executive Summary

The `abcdeez-core` module is a sophisticated adaptive learning and cognitive modeling system designed for research applications. It implements Bayesian inference, expected information gain (EIG) algorithms, and comprehensive experimental design tools for conducting cognitive science research on sequential learning tasks.

## 1. Module Architecture

### 1.1 Top-Level Module Organization

```
core/
├── core/           # Core infrastructure
├── learning/       # Learning models and algorithms
├── tasks/          # Task generation and management
├── statistics/     # Statistical analysis tools
├── experiments/    # Experimental design framework
├── compliance/     # Research compliance tools
├── protocol/       # Version control and reproducibility
├── data/           # Data collection and export
├── ui/             # User interface components
└── demo/           # Demonstration utilities
```

### 1.2 Module Dependencies

- **External Dependencies**: 
  - `serde`: Serialization framework
  - `rand`: Random number generation
  - `chrono`: Date/time handling
  - `statrs`: Statistical computations
  - `reqwest`: HTTP client for backend communication
  - `tokio`: Async runtime (optional)
  - `axum`: Backend server (optional)

- **Internal Dependencies**:
  - All modules depend on `core` for fundamental types
  - `learning` depends on `core`, `tasks`
  - `experiments` depends on `core`, `learning`, `tasks`
  - `statistics` depends on `learning`, `tasks`
  - `ui` depends on all major modules

## 2. Core Data Models

### 2.1 Topology System (`core::topology`)

**Purpose**: Represents ordered sequences and relationships between items (e.g., alphabet, musical notes, days of week).

**Key Structures**:

```rust
pub enum TopologyType {
    Linear,        // Sequential ordering (e.g., A-Z)
    Cyclic,        // Circular ordering (e.g., days of week)
    PartialOrder,  // DAG with partial ordering
    GeneralGraph,  // Arbitrary graph structure
}

pub struct Topology {
    pub topology_type: TopologyType,
    pub nodes: Vec<Node>,           // Items in the topology
    pub edges: Vec<Edge>,           // Connections between items
    pub node_map: HashMap<String, usize>, // ID to index mapping
}

pub struct Node {
    pub id: String,      // Unique identifier (e.g., "node_0")
    pub label: String,   // Display label (e.g., "A")
    pub position: f64,   // Position in sequence/space
}

pub struct Edge {
    pub from: String,    // Source node ID
    pub to: String,      // Target node ID
    pub weight: f64,     // Connection strength/distance
}
```

**Key Methods**:
- `new_linear()`: Create linear sequence
- `new_cyclic()`: Create circular sequence
- `new_dag()`: Create directed acyclic graph
- `get_successor()`: Get next item in sequence
- `get_predecessor()`: Get previous item
- `get_distance()`: Calculate distance between nodes
- `get_k_jump()`: Get item k positions away
- `shortest_path()`: Find shortest path between nodes
- `get_topological_sort()`: Order nodes in DAG

### 2.2 Configuration System (`core::config`)

**Purpose**: Provides population-specific and domain-specific configuration presets.

**Key Structures**:

```rust
pub struct LearnerConfig {
    // Initial state parameters
    pub initial_uncertainty: f64,
    pub initial_memory_strength: f64,
    pub initial_proficiency: f64,
    
    // Learning dynamics
    pub learning_rate_base: f64,
    pub learning_rate_decay: f64,
    pub position_update_weight: f64,
    
    // Memory parameters
    pub memory_update_correct: f64,
    pub memory_update_incorrect: f64,
    pub memory_decay_rate: f64,
    
    // Boundaries and constraints
    pub theta_bounds: (f64, f64),
    pub min_uncertainty: f64,
    pub edge_emphasis: f64,
}

pub struct AdaptiveSchedulingConfig {
    pub initial_epsilon: f64,       // Exploration rate
    pub epsilon_decay: f64,         // Exploration decay
    pub target_success_rate: f64,   // Optimal difficulty target
    pub scoring_weights: ScoringWeights,
    pub use_eig: bool,              // Use Expected Information Gain
}

pub struct HintInterventionConfig {
    pub struggle_rt_threshold_ms: u64,
    pub error_streak_threshold: usize,
    pub hint_delay_ms: u64,
    // Difficulty adjustment parameters
    pub too_easy_error_rate: f64,
    pub too_hard_error_rate: f64,
}

pub struct DomainConfig {
    pub task_difficulties: TaskDifficultyConfig,
    pub chunk_boundaries: Vec<ChunkBoundaryConfig>,
    pub baseline_rt_ms: ResponseTimeConfig,
}
```

**Presets Available**:
- Population types: `adult()`, `child()`, `older_adult()`, `learning_disability()`, `expert()`
- Domain types: `alphabet()`, `music()`, `mathematics()`
- Learning goals: `exploration()`, `mastery()`, `standard()`

### 2.3 Backend Communication (`core::backend`)

**Purpose**: Manages communication with remote backend servers for data synchronization.

**Key Components**:

```rust
pub struct BackendConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub experiment_id: String,
    pub batch_size: usize,
    pub sync_interval_secs: u64,
    pub retry_attempts: u32,
    pub timeout_secs: u64,
}

pub struct BackendClient {
    config: BackendConfig,
    client: reqwest::blocking::Client,
    response_buffer: Vec<TaskResponse>,
    last_sync: DateTime<Utc>,
}
```

**Key Operations**:
- `test_connection()`: Verify backend connectivity
- `register_participant()`: Register new participant
- `start_session()`: Initialize learning session
- `buffer_response()`: Queue response for batch upload
- `sync_responses()`: Upload buffered responses
- `get_next_task()`: Retrieve tasks (for yoked control)
- `upload_session()`: Upload complete session data

## 3. Learning Module

### 3.1 Learner Model (`learning::learner`)

**Purpose**: Maintains state of individual learner's knowledge and abilities.

**Key Components**:

```rust
pub struct LearnerModel {
    pub learner_id: String,
    pub node_embeddings: HashMap<String, LatentNodeEmbedding>,
    pub operation_proficiencies: HashMap<String, OperationProficiency>,
    pub memory_strengths: HashMap<String, MemoryStrength>,
    pub confusability_matrix: HashMap<(String, String), f64>,
    pub chunk_boundaries: Vec<ChunkBoundary>,
    pub total_practice_time: Duration,
    pub session_count: usize,
    pub config: LearnerConfig,
}

pub struct LatentNodeEmbedding {
    pub node_id: String,
    pub position: f64,      // Learned position in mental space
    pub uncertainty: f64,   // Model uncertainty about position
}

pub enum OperationType {
    Successor,              // "What comes after X?"
    Predecessor,            // "What comes before X?"
    PairwiseOrder,         // "Does A come before B?"
    KJump(i32),            // "What is k steps from X?"
    Segment(usize, bool),  // "List n items from X"
    Index,                 // "What position is X?"
}
```

**Key Algorithms**:
- **Position Update**: Weighted average of old and new positions
- **Proficiency Update**: Adaptive learning rate based on practice count and current skill
- **Memory Decay**: Exponential forgetting curve with strength-dependent decay rate
- **Retention Calculation**: Includes primacy/recency effects

### 3.2 Adaptive Scheduler (`learning::adaptive`)

**Purpose**: Selects optimal tasks based on learner state and learning objectives.

**Key Components**:

```rust
pub struct AdaptiveScheduler {
    learner_model: LearnerModel,
    bayesian_model: BayesianLearnerModel,
    topology: Topology,
    task_generator: TaskGenerator,
    epsilon: f64,              // Exploration rate
    use_eig: bool,            // Use Expected Information Gain
    trials_completed: usize,
    exploration_decay: f64,
}
```

**Selection Algorithm**:
1. Decay exploration rate: `ε' = ε × decay^trials`
2. With probability ε': Select random task (exploration)
3. With probability 1-ε': Select optimal task (exploitation)
   - If EIG enabled: Maximize expected information gain
   - Otherwise: Use heuristic scoring (difficulty matching, uncertainty, practice need)

### 3.3 Bayesian Model (`learning::bayesian`)

**Purpose**: Maintains probabilistic beliefs about learner knowledge.

**Key Features**:
- Posterior inference using response data
- Expected Information Gain (EIG) calculation
- Monte Carlo sampling for belief updates
- Uncertainty quantification

### 3.4 Advanced Learning Models

- **Strategy Mixture Model**: Detects and models multiple cognitive strategies
- **Macro Learning**: Cross-session learning and consolidation
- **Transfer Learning**: Models knowledge transfer between domains
- **Hierarchical Bayes**: Population-level parameter estimation

## 4. Task System

### 4.1 Task Types (`tasks::core`)

**Core Task Types**:
1. **Successor/Predecessor**: Next/previous item queries
2. **PairwiseOrder**: Relative ordering questions
3. **KJump**: Items at specific distances
4. **Segment**: Sequences of items
5. **Index**: Position queries
6. **MissingItem**: Fill-in-the-blank
7. **ShortestDistance**: Distance calculations
8. **Comparability**: Whether items are comparable (DAG)
9. **TopologicalSort**: Ordering tasks (DAG)
10. **ShortestPath**: Path finding (graphs)

### 4.2 Task Generation (`tasks::TaskGenerator`)

**Key Features**:
- Seeded random generation for reproducibility
- Difficulty calibration based on task parameters
- Distractor generation for multiple-choice
- Support for all topology types

### 4.3 Extended Task Types

- **Music Tasks**: Scale exercises, interval recognition, chord progressions
- **Navigation Tasks**: Spatial reasoning, route planning
- **Boundary Tasks**: Chunk detection, segmentation

## 5. Experimental Framework

### 5.1 Design Types (`experiments::design`)

**Supported Designs**:
- **Between-Subjects**: Random assignment to conditions
- **Within-Subjects**: All participants experience all conditions
- **Mixed Designs**: Combination of between and within factors

**Randomization Methods**:
- Simple random assignment
- Block randomization
- Stratified randomization
- Adaptive randomization

**Counterbalancing Methods**:
- Complete counterbalancing
- Latin Square
- Balanced Latin Square
- Williams Square (controls carryover)

### 5.2 A/B Testing Framework (`experiments::ab_testing`)

**Features**:
- Real-time effect size monitoring
- Sequential analysis with early stopping
- Multiple comparison corrections
- Power analysis

### 5.3 Multi-Session Management (`experiments::multi_session`)

**Capabilities**:
- Longitudinal study design
- Session scheduling
- Cross-session analysis
- Retention curve modeling

## 6. Statistical Analysis

### 6.1 Core Statistics (`statistics::core`)

**Key Analyses**:
- Response time distribution fitting (Ex-Gaussian)
- Strategy detection and classification
- Performance metrics calculation
- Session-level summaries

### 6.2 Validation Tools (`statistics::validation`)

**Assumption Checks**:
- Normality tests (Shapiro-Wilk, Anderson-Darling)
- Homoscedasticity tests
- Outlier detection
- Independence verification

### 6.3 Mixed Effects Models (`statistics::mixed_effects`)

**Features**:
- Random effects for participants and items
- Fixed effects for experimental conditions
- Nested and crossed designs
- Model comparison and selection

### 6.4 Power Analysis (`statistics::power_analysis`)

**Capabilities**:
- Sample size calculation
- Effect size estimation
- Real-time power monitoring
- Adaptive stopping rules

## 7. Compliance and Protocol

### 7.1 Research Compliance (`compliance/`)

**Components**:
- **IRB Tools**: Consent templates, application generation
- **Preregistration**: Study protocol documentation
- **Audit Trail**: Complete action logging
- **Citation Management**: Reference tracking

### 7.2 Protocol Management (`protocol/`)

**Features**:
- **Version Control**: Protocol versioning and change tracking
- **Seed Management**: Reproducibility through controlled randomization
- **Collaboration**: Multi-researcher protocol management

## 8. Data Collection and Export

### 8.1 Data Collection (`data/`)

**Supported Modalities**:
- **Audio Recording**: Think-aloud protocols
- **Sensor Integration**: Mock EEG, eye tracking, GSR
- **Interaction Tracking**: Mouse, keyboard events
- **Performance Metrics**: Response times, accuracy

### 8.2 Export Formats

```rust
pub struct LearnerDataExport {
    pub learner_id: String,
    pub sessions: Vec<SessionData>,
    pub model_state: LearnerModel,
    pub statistics: DetailedStatistics,
    pub metadata: HashMap<String, Value>,
}
```

**Export Options**:
- JSON for programmatic access
- CSV for statistical software
- Binary formats for efficiency

## 9. User Interface

### 9.1 Terminal UI (`ui::tui`)

**Features**:
- Interactive learning sessions
- Real-time performance visualization
- Configuration management
- Session replay

### 9.2 Hint System (`ui::hints`)

**Adaptive Interventions**:
- Struggle detection based on errors and response time
- Multi-level hint generation
- Difficulty adjustment recommendations

## 10. External Interface Expectations

### 10.1 Expected from App/Web Backend

The core module expects external systems to provide:

1. **User Management**:
   - Participant registration
   - Session authentication
   - Group assignment

2. **Data Persistence**:
   - Response storage
   - Session state management
   - Cross-session tracking

3. **Task Presentation**:
   - Visual/audio rendering
   - Response collection
   - Timing accuracy

4. **Configuration**:
   - Study protocol settings
   - Experimental conditions
   - Population parameters

### 10.2 API Contract

**Inbound (to Core)**:
```rust
// Task response from UI
TaskResponse {
    task: Task,
    user_answer: String,
    correct: bool,
    response_time_ms: u128,
    timestamp: DateTime<Utc>,
}

// Configuration update
SystemConfig {
    learner: LearnerConfig,
    adaptive: AdaptiveSchedulingConfig,
    hints: HintInterventionConfig,
    domain: DomainConfig,
}
```

**Outbound (from Core)**:
```rust
// Next task selection
Task {
    task_type: TaskType,
    prompt: String,
    correct_answer: String,
    options: Vec<String>,
    difficulty: f64,
    operation: OperationType,
}

// Performance metrics
LearnerMetrics {
    accuracy: f64,
    avg_response_time: f64,
    tasks_completed: usize,
    current_proficiency: HashMap<String, f64>,
}
```

### 10.3 Communication Protocols

**Synchronous Mode**:
- Direct function calls
- Immediate response required
- Used for: Task selection, response processing

**Asynchronous Mode** (with `async` feature):
- Message passing via channels
- Batch processing
- Used for: Backend sync, data export

**HTTP API** (with `backend-server` feature):
- RESTful endpoints
- JSON payloads
- Used for: Remote clients, web integration

## 11. Key Algorithms and Workflows

### 11.1 Learning Session Workflow

```
1. Initialize:
   - Create Topology (alphabet, music, etc.)
   - Initialize LearnerModel with config
   - Create AdaptiveScheduler

2. For each trial:
   a. Select task (via scheduler)
   b. Present to user (external system)
   c. Collect response
   d. Update models:
      - Update operation proficiency
      - Update memory strength
      - Update Bayesian beliefs
   e. Check interventions:
      - Detect struggle
      - Generate hints if needed
   f. Log data

3. End session:
   - Calculate statistics
   - Export data
   - Sync with backend
```

### 11.2 Expected Information Gain (EIG) Calculation

```
1. For each candidate task:
   a. Sample possible responses
   b. For each response:
      - Calculate posterior belief update
      - Compute information gain (KL divergence)
   c. Weight by response probability
   d. Sum to get expected gain

2. Select task with maximum EIG
```

### 11.3 Memory Decay Model

```
Retention(t) = Strength × exp(-λ × t)
where:
  λ = base_decay × (2 - current_strength)
  t = hours since last practice
  
With edge effects:
  Retention'(t) = Retention(t) × (1 + ε × edge_emphasis)
  where edge_emphasis peaks at sequence boundaries
```

## 12. Performance Characteristics

### 12.1 Computational Complexity

- Task selection: O(n × m) where n = tasks, m = MC samples
- Model update: O(1) for most operations
- EIG calculation: O(n × m × k) where k = response options
- Statistical analysis: O(n log n) for most tests

### 12.2 Memory Requirements

- Per learner: ~10KB base + 1KB per session
- Topology: O(n²) for full graph representations
- Response buffer: Configurable, default 50 responses

### 12.3 Scalability

- Supports unlimited participants (with backend)
- Handles topologies up to 1000 nodes efficiently
- Batch processing for large datasets

## 13. Error Handling

The module uses a comprehensive error type system:

```rust
pub enum Error {
    InvalidTopology(String),
    TaskGenerationError(String),
    NumericalError(String),
    InvalidParameters(String),
    NotFound(String),
    InsufficientData { required: usize, actual: usize },
    StatisticalAssumptionViolation(String),
    ConvergenceFailure { iterations: usize, tolerance: f64, final_error: f64 },
    IoError(std::io::Error),
    SerializationError(String),
}
```

All public APIs return `Result<T, Error>` for proper error propagation.

## 14. Configuration and Deployment

### 14.1 Feature Flags

- `default`: ["cli", "async"]
- `cli`: Terminal UI support
- `async`: Tokio async runtime
- `backend-server`: Axum server capabilities

### 14.2 Environment Variables

- `LEARNING_API_URL`: Backend API endpoint
- `LEARNING_API_KEY`: API authentication key
- `EXPERIMENT_ID`: Current experiment identifier

### 14.3 Configuration Files

Supports TOML and JSON configuration:
- `backend.toml`: Backend connection settings
- `config.toml`: System configuration
- `experiment.json`: Experimental protocol

## 15. Testing and Validation

The module includes comprehensive test suites:

- **Unit tests**: Core algorithms and data structures
- **Integration tests**: Module interactions
- **Property tests**: Randomized testing with proptest
- **Stress tests**: Performance under load
- **Statistical tests**: Validation of statistical methods
- **Benchmarks**: Performance measurements

## Summary

The `abcdeez-core` module provides a complete framework for adaptive learning research, combining sophisticated cognitive modeling with robust experimental design tools. It expects external systems to handle user interaction and data persistence while providing the algorithmic core for learning optimization and analysis.

Key strengths:
- Population-specific adaptation
- Rigorous statistical framework
- Reproducible experimental design
- Comprehensive compliance tools
- Flexible architecture supporting multiple UIs

The module is designed for researchers conducting cognitive science experiments, educational technology developers, and anyone studying sequential learning behaviors.