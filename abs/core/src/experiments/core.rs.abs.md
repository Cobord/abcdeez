# Experiment Core Module Abstract

## High-level Purpose and Responsibility
The experiments core module provides the foundational infrastructure for managing experimental workflows in the learning system. It serves as the central orchestrator for experiment lifecycle management, data collection, and result computation, supporting various experimental paradigms including A/B testing, multi-session studies, and adaptive learning experiments.

## Key Data Structures and Relationships
- **Experiment**: Central struct containing experiment metadata, configuration, state, and collected data
- **ExperimentConfig**: Configuration parameters including participant limits, task parameters, and randomization settings
- **ExperimentResult**: Computed outcomes including statistical measures, performance metrics, and effect sizes
- **ExperimentState**: Runtime state management (NotStarted, Running, Paused, Completed)
- **TaskResponse**: Individual response records linking tasks to participant behavior
- **ParticipantData**: Aggregated participant information and performance history

## Main Data Flows and Transformations
1. **Experiment Creation**: Configuration → Experiment instantiation with participant management
2. **Task Generation**: Experiment parameters → Task generation through TaskGenerator integration
3. **Response Collection**: Participant inputs → TaskResponse records with timing and accuracy data
4. **Result Computation**: Collected responses → Statistical analysis and performance metrics
5. **State Management**: External events → State transitions with validation and persistence

## External Dependencies and Interfaces
- **Learning Module**: Integration with adaptive learning algorithms and learner state management
- **Tasks Module**: Task generation and validation through TaskGenerator and TaskSession
- **Statistics Module**: Statistical analysis, hypothesis testing, and effect size calculations
- **Data Module**: Persistence layer for experiment data and participant tracking
- **Protocol Module**: Version control and reproducibility through seed management

## State Management Patterns
- **Finite State Machine**: Experiment lifecycle managed through explicit state transitions
- **Event-driven Updates**: Participant actions trigger state changes and data collection
- **Immutable Results**: Computed results are immutable once experiment is completed
- **Concurrent Participant Management**: Thread-safe participant data collection and aggregation

## Core Algorithms or Business Logic Abstractions
- **Adaptive Sampling**: Dynamic participant assignment based on experiment design and statistical power
- **Real-time Analysis**: Incremental computation of performance metrics during experiment execution
- **Randomization Control**: Deterministic randomization using configurable seeds for reproducibility
- **Effect Size Calculation**: Standardized effect size computation (Cohen's d, eta-squared) for result interpretation
- **Completion Criteria**: Automated experiment termination based on statistical significance or sample size targets