# ABCDeez Core

## A Research-Grade Cognitive Assessment & Adaptive Learning System

Welcome to the comprehensive documentation for **ABCDeez Core**, a sophisticated Rust library implementing state-of-the-art methods for cognitive assessment, adaptive learning, and psychological modeling.

## What is ABCDeez Core?

ABCDeez Core is a research-grade library that models human learning and memory processes for ordered sequences and structured knowledge. Built with scientific rigor and validated against established psychological phenomena, it provides:

- **Cognitive Modeling**: Advanced models of human learning, memory, and strategic processing
- **Adaptive Assessment**: Bayesian methods for optimal task selection and difficulty calibration
- **Statistical Analysis**: Comprehensive statistical tools for response time analysis and performance metrics
- **Psychological Validity**: Implementations that reproduce well-established cognitive phenomena

## Key Capabilities

### 🧠 Cognitive Science Foundation
- Models serial position effects, spacing effects, and practice curves
- Implements forgetting curves based on memory research
- Captures strategy shifts from algorithmic to retrieval-based processing
- Validates against empirical psychological data

### 📊 Advanced Statistics
- Ex-Gaussian distribution modeling for response times
- Bayesian inference with proper uncertainty quantification
- Power analysis and effect size calculations
- Multiple comparison corrections and robust statistics

### 🎯 Adaptive Learning
- Information-theoretic task selection
- Personalized difficulty calibration
- Optimal review scheduling based on memory decay
- Real-time performance monitoring

### 🔬 Research Quality
- 165+ comprehensive tests covering all components
- Property-based testing for mathematical invariants
- Cross-validation and bootstrap methods
- Simulation-based validation studies

## Who Should Use This?

This library is designed for:

- **Researchers** studying human cognition and learning
- **Educators** building adaptive learning systems
- **Psychologists** conducting cognitive assessments
- **Data Scientists** analyzing behavioral data
- **Developers** creating evidence-based educational technology

## Scientific Rigor

Every component in ABCDeez Core is:
- Grounded in peer-reviewed psychological research
- Validated through comprehensive statistical testing
- Verified against known empirical phenomena
- Documented with theoretical foundations

## Getting Started

```rust
use abcdeez_core::prelude::*;

// Create a topology (e.g., alphabet sequence)
let topology = Topology::alphabet();

// Initialize a learner model
let mut learner = LearnerModel::new("student_001".to_string(), &topology);

// Create a Bayesian assessment model
let mut bayesian = BayesianLearnerModel::new(&topology);

// Generate an adaptive task
let task = bayesian.select_next_task(TaskType::Successor);

// Process response and update models
let response = ResponseData {
    task,
    correct: true,
    response_time: 1250.0,
};

learner.update_with_response(&response);
bayesian.update_with_response(response);
```

## Architecture Overview

The system is organized into several interconnected modules:

```
abcdeez-core/
├── core/           # Core data structures and configuration
├── learning/       # Learner models and Bayesian framework
├── statistics/     # Statistical analysis and validation
├── tasks/          # Task generation and scheduling
└── tests/          # Comprehensive test suite
```

## Key Features

### Topology System
Flexible representation of knowledge structures:
- Linear sequences (alphabets, numbers)
- Cyclic structures (days, months)
- Partial orders (prerequisite graphs)
- General directed graphs

### Learning Models
Sophisticated cognitive modeling:
- Latent node embeddings with uncertainty
- Operation-specific proficiency tracking
- Memory strength with forgetting curves
- Chunk boundary detection

### Bayesian Framework
Principled uncertainty quantification:
- Hierarchical Bayesian models
- Monte Carlo information gain
- Model comparison metrics (AIC, BIC, WAIC)
- Posterior predictive checking

### Statistical Methods
Research-grade statistical tools:
- Response time distribution analysis
- Strategy classification algorithms
- Robust effect size calculations
- Advanced validation techniques

## Quality Guarantees

✅ **100% Test Coverage** of critical paths  
✅ **Mathematical Correctness** verified through property testing  
✅ **Statistical Validity** confirmed through simulation  
✅ **Psychological Realism** validated against empirical data  
✅ **Production Ready** with comprehensive error handling  

## Documentation Structure

This book is organized to serve multiple audiences:

- **Quick Start**: Jump to [System Overview](./overview/system.md) for high-level understanding
- **Implementation**: See [API Reference](./api/introduction.md) for coding details  
- **Research**: Explore [Psychological Phenomena](./psychology/introduction.md) for theoretical background
- **Validation**: Review [Testing & Quality](./testing/overview.md) for quality assurance

## License and Contributing

ABCDeez Core is open-source software designed to advance research in cognitive science and education. We welcome contributions from researchers, educators, and developers.

---

*Start exploring with the [System Overview →](./overview/system.md)*