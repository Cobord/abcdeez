# core/config.rs - System Configuration Management Abstract

## High-Level Purpose
Comprehensive configuration management system providing validated, hierarchical configuration for all framework components, with population-specific presets and domain-specific parameter tuning for adaptive learning systems.

## Key Data Structures and Relationships
- **SystemConfig**: Top-level configuration aggregating all subsystem configurations
- **LearnerConfig**: Individual learner behavior parameters and learning dynamics
- **AdaptiveSchedulingConfig**: Task scheduling and adaptation parameters
- **HintInterventionConfig**: Intelligent hint system configuration
- **DomainConfig**: Domain-specific task difficulties and knowledge structures
- **Preset System**: Population and goal-specific configuration templates

## Main Data Flows
- **Configuration Validation**: Comprehensive parameter validation with range and consistency checking
- **Preset Selection**: Population-type and domain-specific configuration generation
- **Parameter Hierarchies**: Nested configuration structures with inheritance and overrides
- **Serialization Support**: JSON/TOML serialization for persistent configuration storage

## External Dependencies
- **serde**: Configuration serialization and deserialization
- Standard library collections for configuration data structures

## State Management Patterns
- **Immutable Configuration**: Read-only configuration objects with validation
- **Preset Templates**: Pre-defined configuration templates for common scenarios
- **Validation State**: Comprehensive validation with detailed error reporting

## Core Algorithms and Business Logic Abstractions
- **Parameter Validation**: Multi-level validation with domain-specific constraints
- **Population Modeling**: Evidence-based parameter sets for different learner populations
- **Domain Adaptation**: Task difficulty and response time modeling for different knowledge domains
- **Adaptive Tuning**: Dynamic parameter adjustment based on learning goals and population characteristics

## Configuration Domains
- **Learning Parameters**: Memory strength, uncertainty, proficiency modeling
- **Adaptive Scheduling**: Epsilon-greedy exploration, success rate targeting, scoring weights
- **Hint Systems**: Struggle detection, error streak thresholds, adaptive timing
- **Domain Knowledge**: Task difficulties, chunk boundaries, response time baselines
- **Population Presets**: Adult, child, older adult, learning disability, and expert configurations

## Validation and Quality Assurance
- **Range Validation**: Parameter bounds checking with domain-appropriate limits
- **Consistency Validation**: Cross-parameter consistency and logical constraint checking
- **Weight Normalization**: Automatic validation of probability distributions and weight sums
- **Evidence-Based Defaults**: Scientifically-grounded default parameters based on research literature