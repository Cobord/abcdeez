# Proposed src/ Directory Reorganization

## Current Issues
- 42+ files in flat src/ structure
- Related functionality scattered across multiple files
- Difficult to navigate and understand system architecture
- No clear separation of concerns

## Proposed Structure

```
src/
├── lib.rs                          # Main library entry point
├── error.rs                        # Global error types
├── prelude.rs                      # Common imports
│
├── core/                          # Core learning engine
│   ├── mod.rs
│   ├── learner.rs                 # Main learner model
│   ├── bayesian.rs               # Bayesian inference engine  
│   ├── topology.rs               # Topology abstractions
│   ├── adaptive.rs               # Adaptive scheduling
│   └── memory.rs                 # Memory & chunking models
│
├── tasks/                         # Task generation & management
│   ├── mod.rs
│   ├── generator.rs              # Task generation
│   ├── types.rs                  # Task type definitions
│   ├── difficulty.rs             # Difficulty calibration
│   └── validation.rs             # Task validation
│
├── experimental/                  # Experimental design & execution
│   ├── mod.rs
│   ├── design.rs                 # Experimental design patterns
│   ├── framework.rs              # Experiment execution
│   ├── randomization.rs          # Randomization strategies
│   └── counterbalancing.rs       # Counterbalancing methods
│
├── boundaries/                    # Boundary & chunking systems
│   ├── mod.rs
│   ├── detection.rs              # Boundary detection
│   ├── training.rs               # Boundary crossing training
│   ├── hierarchical.rs           # Hierarchical chunking
│   └── types.rs                  # Boundary type definitions
│
├── statistics/                    # Statistical analysis & validation
│   ├── mod.rs
│   ├── validation.rs             # Statistical validation
│   ├── tests.rs                  # Hypothesis testing
│   ├── modeling.rs               # Statistical modeling
│   ├── mixed_effects.rs          # Mixed effects analysis
│   └── power_analysis.rs         # Power analysis
│
├── intervention/                  # Real-time intervention system
│   ├── mod.rs
│   ├── hints.rs                  # Hint generation
│   ├── struggle.rs               # Struggle detection
│   ├── difficulty.rs             # Adaptive difficulty
│   └── feedback.rs               # Feedback systems
│
├── config/                        # Configuration management
│   ├── mod.rs
│   ├── learner.rs                # Learner configurations
│   ├── domain.rs                 # Domain configurations
│   ├── experimental.rs           # Experiment configurations
│   └── population.rs             # Population-specific configs
│
├── data/                          # Data management & persistence
│   ├── mod.rs
│   ├── export.rs                 # Data export
│   ├── import.rs                 # Data import
│   ├── storage.rs                # Data storage
│   └── formats.rs                # Data format definitions
│
├── research/                      # Research infrastructure
│   ├── mod.rs
│   ├── citations.rs              # Citation management
│   ├── compliance.rs             # IRB/ethics compliance
│   ├── protocols.rs              # Research protocols
│   ├── versioning.rs             # Protocol versioning
│   └── audit.rs                  # Audit trails
│
├── analysis/                      # Advanced analysis methods
│   ├── mod.rs
│   ├── longitudinal.rs           # Longitudinal analysis
│   ├── transfer.rs               # Transfer learning analysis
│   ├── individual.rs             # Individual differences
│   └── population.rs             # Population analysis
│
├── domains/                       # Domain-specific implementations
│   ├── mod.rs
│   ├── alphabet.rs               # Alphabet learning
│   ├── music.rs                  # Music theory
│   ├── mathematics.rs            # Mathematical sequences
│   └── spatial.rs                # Spatial navigation
│
├── interaction/                   # User interaction & tracking
│   ├── mod.rs
│   ├── tracking.rs               # Interaction tracking
│   ├── audio.rs                  # Audio recording/analysis
│   ├── sensors.rs                # Sensor integration
│   └── multimodal.rs             # Multimodal interaction
│
├── optimization/                  # Performance & optimization
│   ├── mod.rs
│   ├── prediction.rs             # Performance prediction
│   ├── scheduling.rs             # Schedule optimization
│   ├── tracing.rs                # Performance tracing
│   └── profiling.rs              # System profiling
│
├── session/                       # Session management
│   ├── mod.rs
│   ├── manager.rs                # Session management
│   ├── multi_session.rs          # Multi-session experiments
│   ├── seeds.rs                  # Seed management
│   └── state.rs                  # Session state
│
├── testing/                       # A/B testing framework
│   ├── mod.rs
│   ├── framework.rs              # A/B test framework
│   ├── variants.rs               # Test variants
│   └── analysis.rs               # A/B test analysis
│
├── ui/                           # User interfaces
│   ├── mod.rs
│   ├── tui.rs                    # Terminal UI
│   ├── dashboard.rs              # Research dashboard
│   ├── components.rs             # UI components
│   └── visualizations.rs         # Data visualizations
│
└── integration/                  # External integrations
    ├── mod.rs
    ├── apis.rs                   # External API integrations
    ├── databases.rs              # Database connections
    ├── cloud.rs                  # Cloud service integrations
    └── hardware.rs               # Hardware integrations
```

## Migration Plan

### Phase 1: Core Modules (Week 1)
Move the most critical files first:
```bash
# Create directory structure
mkdir -p src/{core,tasks,experimental,boundaries,statistics}

# Move core files
mv src/learner.rs src/core/
mv src/bayesian.rs src/core/
mv src/topology.rs src/core/
mv src/adaptive.rs src/core/

# Move task files  
mv src/tasks.rs src/tasks/types.rs
# Create new task generator from existing code

# Move experimental files
mv src/experimental_design.rs src/experimental/design.rs
mv src/experiments.rs src/experimental/framework.rs

# Move boundary files
mv src/boundaries.rs src/boundaries/training.rs

# Move statistics files
mv src/statistical_validation.rs src/statistics/validation.rs
mv src/statistics.rs src/statistics/modeling.rs
mv src/mixed_effects.rs src/statistics/
mv src/power_analysis.rs src/statistics/
```

### Phase 2: Support Modules (Week 2) 
```bash
mkdir -p src/{config,intervention,data,research}

# Move config files
mv src/config.rs src/config/learner.rs
# Split into domain, experimental, population configs

# Move intervention files
mv src/hints.rs src/intervention/
# Extract struggle detection, difficulty adaptation

# Move data files
mv src/export.rs src/data/
# Create import, storage, formats modules

# Move research files
mv src/citation_manager.rs src/research/citations.rs
mv src/irb_compliance.rs src/research/compliance.rs
mv src/protocol_version_control.rs src/research/versioning.rs
mv src/audit_trail.rs src/research/audit.rs
```

### Phase 3: Advanced Modules (Week 3)
```bash
mkdir -p src/{analysis,domains,interaction,optimization,session,testing}

# Move analysis files
mv src/transfer_learning.rs src/analysis/transfer.rs
# Create longitudinal, individual, population modules

# Move domain files  
mv src/music.rs src/domains/
# Create alphabet, mathematics, spatial modules

# Move interaction files
mv src/interaction_tracking.rs src/interaction/tracking.rs
mv src/audio_recording.rs src/interaction/audio.rs
mv src/sensor_integration.rs src/interaction/sensors.rs

# Move optimization files
mv src/prediction.rs src/optimization/
mv src/performance_tracing.rs src/optimization/tracing.rs

# Move session files
mv src/multi_session.rs src/session/
mv src/seed_management.rs src/session/seeds.rs

# Move testing files
mv src/ab_testing.rs src/testing/framework.rs
```

### Phase 4: UI & Integration (Week 4)
```bash
mkdir -p src/{ui,integration}

# Move UI files (if any remain in core)
mv src/tui.rs src/ui/ 2>/dev/null || true
mv src/ui.rs src/ui/components.rs 2>/dev/null || true
mv src/research_dashboard.rs src/ui/dashboard.rs 2>/dev/null || true

# Create integration modules for external systems
# Most of this will be new code for API clients, etc.
```

## Updated lib.rs Structure

```rust
// lib.rs - Main library entry point

pub mod error;
pub mod prelude;

// Core learning engine
pub mod core;

// Task system  
pub mod tasks;

// Experimental framework
pub mod experimental;

// Boundary detection & training
pub mod boundaries;

// Statistical analysis
pub mod statistics; 

// Real-time interventions
pub mod intervention;

// Configuration management
pub mod config;

// Data persistence & export  
pub mod data;

// Research infrastructure
pub mod research;

// Advanced analytics
pub mod analysis;

// Domain implementations
pub mod domains;

// Interaction tracking
pub mod interaction;

// Performance optimization
pub mod optimization;

// Session management
pub mod session;

// A/B testing
pub mod testing;

// User interfaces
#[cfg(feature = "ui")]
pub mod ui;

// External integrations
#[cfg(feature = "integrations")] 
pub mod integration;

// Re-export commonly used types
pub use core::{LearnerModel, BayesianLearnerModel};
pub use tasks::{Task, TaskGenerator, TaskType};
pub use experimental::{ExperimentFramework, ExperimentalDesigner};
pub use config::{LearnerConfig, SystemConfig};
pub use error::{Error, Result};
```

## Module-Level Documentation

Each module gets comprehensive documentation:

```rust
//! # Core Learning Engine
//! 
//! This module contains the core adaptive learning algorithms including:
//! 
//! - [`LearnerModel`]: Individual learner state and proficiency tracking
//! - [`BayesianLearnerModel`]: Bayesian inference for learning parameters  
//! - [`AdaptiveScheduler`]: Task selection with Expected Information Gain
//! - [`Topology`]: Abstract representations of learning domains
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::core::{LearnerModel, Topology};
//! 
//! let topology = Topology::alphabet();
//! let learner = LearnerModel::new("learner_001", &topology);
//! ```

pub mod learner;
pub mod bayesian; 
pub mod adaptive;
pub mod topology;
pub mod memory;
```

## Benefits of This Reorganization

1. **Clear Mental Model**: Each directory represents a major system component
2. **Reduced Cognitive Load**: Find related code in predictable locations
3. **Better Testing**: Each module can have focused integration tests
4. **Conditional Compilation**: Optional features (ui, integrations) can be disabled
5. **API Clarity**: Public exports clearly define the system's interface
6. **Maintainability**: Changes isolated to relevant modules
7. **Documentation**: Module-level docs explain each subsystem
8. **Parallel Development**: Teams can work on different modules independently

## Migration Benefits

- **From**: 42 files in flat structure, hard to navigate
- **To**: ~15 focused modules, each containing 2-5 related files
- **Result**: Much clearer architecture and easier maintenance

This reorganization transforms the codebase from a flat file structure into a well-architected modular system that reflects the sophisticated nature of the learning platform.