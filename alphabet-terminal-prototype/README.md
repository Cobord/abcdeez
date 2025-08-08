# Alphabet Terminal Prototype

## Graph-Coded Mental Model Training System

A Rust implementation of the adaptive training system described in "Building Graph-Coded Mental Models through Adaptive Training" (PAPER.md). This system helps learners develop flexible, navigable mental representations of structured knowledge domains.

## Overview

This project implements a complete adaptive learning system that:
- Models learner knowledge using Bayesian probabilistic methods
- Adaptively selects tasks using Expected Information Gain (EIG)
- Supports multiple graph topologies (linear, cyclic, DAG, general graphs)
- Tracks detailed performance metrics and learning trajectories
- Exports comprehensive data for analysis

## Key Features

### 1. Adaptive Task Selection
- **Expected Information Gain (EIG)**: Monte Carlo simulation estimates information gain for each candidate task
- **Difficulty Targeting**: Maintains optimal challenge level (70-80% success rate)
- **ε-greedy Exploration**: Balances exploitation with exploration (ε=0.1)

### 2. Comprehensive Task Battery
- **Core Operations**: Successor, predecessor, pairwise order, k-jump navigation
- **Segment Tasks**: Forward/reverse recitation, missing item detection
- **DAG Operations**: Comparability, topological sort, minimal/maximal elements
- **Extended Tasks**: Boundary bridging, insertion adaptation, landmark navigation

### 3. Learner Modeling
- **Latent Node Embeddings**: Probabilistic positions with uncertainty estimates
- **Operation Proficiencies**: Task-specific skill tracking with IRT-based updates
- **Memory Strengths**: Forgetting curves and optimal review scheduling
- **Confusability Matrix**: Tracks systematic confusion patterns
- **Chunk Boundaries**: Detects and models mental segmentation

### 4. Statistical Framework
- **Hierarchical Bayesian Models**: Population-level parameter estimation
- **Ex-Gaussian RT Modeling**: Response time distribution analysis with numerical stability
- **Strategy Detection**: Identifies serial scan vs. direct retrieval strategies
- **Transfer Learning**: Measures and predicts cross-domain transfer

## Architecture

```
src/
├── lib.rs                 # Library exports and feature configuration
├── main.rs               # CLI entry point (requires 'cli' feature)
├── topology.rs           # Graph structures and operations
├── learner.rs            # Core learner model with identifiability constraints
├── bayesian.rs           # Bayesian EIG implementation
├── adaptive.rs           # Adaptive scheduling with EIG selection
├── tasks.rs              # Task generation and session management
├── extended_tasks.rs     # Advanced task types and dynamic topologies
├── statistics.rs         # Statistical models (Ex-Gaussian, strategy analysis)
├── boundaries.rs         # Chunk boundary detection and modeling
├── navigation.rs         # Graph navigation algorithms
├── macro_learning.rs     # Pattern discovery and macro operations
├── transfer_learning.rs  # Cross-domain transfer measurement
├── hierarchical_bayes.rs # Population-level modeling
├── strategy_mixture.rs   # Mixed strategy modeling
├── statistical_validation.rs # Hypothesis testing framework
├── experiments.rs        # Experiment management
├── export.rs            # Data export with strategy detection
├── demo.rs              # Demonstration functions
├── ui.rs                # Terminal UI (optional, requires 'cli' feature)
├── prediction.rs        # Performance prediction
├── hints.rs             # Adaptive hint generation
└── music.rs             # Musical structure domain
```

## Building and Running

### Prerequisites
- Rust 1.70+ (for stable async support)
- Cargo

### Build
```bash
# Standard build with CLI support
cargo build --release

# Library-only build (no terminal UI)
cargo build --release --no-default-features
```

### Run Demonstrations
```bash
# Run comprehensive test suite
cargo run --bin test_implementation

# Run interactive CLI (requires terminal)
cargo run --bin graph-learning-cli
```

### Feature Flags
- `cli` (default): Enables terminal UI with crossterm
- `async`: Enables async/await support with tokio

## Usage Examples

### Basic Usage
```rust
use graph_learning_core::*;

// Create a linear topology (alphabet)
let topology = Topology::alphabet();

// Initialize learner model
let mut learner = LearnerModel::new("student_1".to_string(), &topology);

// Apply identifiability constraints to prevent gauge freedom
learner.apply_identifiability_constraints();

// Create adaptive scheduler with EIG
let mut scheduler = AdaptiveScheduler::new(learner, topology);

// Select next optimal task
let task = scheduler.select_next_task();

// Update model with response
scheduler.update_model(&task, correct, response_time_ms);
```

### Bayesian EIG Calculation
```rust
use graph_learning_core::bayesian::*;

let bayesian = BayesianLearnerModel::new(&topology);

// Calculate Expected Information Gain for a task
let eig = bayesian.calculate_eig(&task);

// Rank multiple tasks by EIG
let ranked = bayesian.rank_tasks_by_eig(candidate_tasks);
```

### Export Session Data
```rust
use graph_learning_core::export::*;

let export = LearnerDataExport::from_learner_model(
    &learner_model,
    sessions,
    Some("exp_001".to_string())
);

// Save to JSON
export.save_to_file("data/participant_001.json")?;
```

## Mathematical Foundations

### Expected Information Gain (EIG)
The system implements the paper's EIG formula:

```
EIG(q) = E[KL(p(θ|D_t) || p(θ|D_t, Response to q))]
```

Where:
- θ represents model parameters (positions, proficiencies)
- D_t is the data observed up to time t
- KL is Kullback-Leibler divergence

Implementation uses Monte Carlo sampling with n=1000 samples for estimation.

### KL Divergence for Gaussians
For Gaussian posteriors N(μ₁, σ₁²) and N(μ₂, σ₂²):

```
KL(P||Q) = log(σ₂/σ₁) + (σ₁² + (μ₁ - μ₂)²)/(2σ₂²) - 1/2
```

### Numerical Stability
Ex-Gaussian PDF calculation includes stability checks:
- Bounds checking for exponential arguments (-50 < arg < 50)
- Special handling for extreme erfc arguments
- Finite result validation

## Performance Metrics

### Core Metrics
1. **Bidirectionality Index**: |θ_forward - θ_backward|
2. **Symbolic Distance Slope**: Regression slope of RT vs. distance
3. **Chunk Boundary Penalty**: Average boundary crossing cost
4. **Transfer Index**: Performance ratio across domains

### Strategy Detection
Automatic classification based on RT-distance correlation:
- r > 0.7: Serial Scan
- r < 0.3: Direct Index  
- 0.3 ≤ r ≤ 0.7: Mixed Strategy

## Testing

Run the comprehensive test suite:
```bash
cargo test
```

Run demonstrations:
```bash
cargo run --bin test_implementation
```

## Data Export Format

The system exports comprehensive JSON data including:
- Session histories with timestamps
- Performance trajectories
- Model snapshots (embeddings, proficiencies)
- Error analysis and confusion matrices
- Strategy classifications
- Population-level statistics

## Implementation Status

✅ **Fully Implemented**:
- All core task types from PAPER.md
- Bayesian EIG with Monte Carlo estimation
- Hierarchical Bayesian modeling
- Ex-Gaussian RT modeling with numerical stability
- Identifiability constraints for embeddings
- Strategy detection and classification
- Transfer learning measurement
- Comprehensive data export

## Citation

If you use this code in your research, please cite:

```
Building Graph-Coded Mental Models through Adaptive Training
[Author Names]
[Publication Details]
```

## License

[License information - to be specified]

## Contributing

Contributions are welcome! Please ensure:
1. All tests pass
2. Code follows Rust conventions
3. Mathematical correctness is maintained
4. Documentation is updated

## Contact

[Contact information - to be specified]