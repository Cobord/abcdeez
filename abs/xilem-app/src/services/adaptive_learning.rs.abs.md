# services/adaptive_learning.rs - Adaptive Learning Service Integration

## Conceptual Overview
Integration layer with abcdeez_core adaptive learning systems. Manages learner models, task selection, response processing, and provides custom intervention logic for the UI application.

## Key Data Flows
- Integrates with abcdeez_core AdaptiveScheduler and LearnerModel
- Processes task responses for model updates
- Selects next tasks based on learning progression
- Provides custom hint and intervention systems
- Exports learner models and Bayesian models

## Main Responsibilities
- abcdeez_core integration and orchestration
- Adaptive task selection algorithms
- Response processing and model updates
- Custom intervention and hint generation
- Learner metrics calculation and reporting
- Model export for persistence and analysis

## Dependencies on Other Components
- `abcdeez_core::*` - Core learning algorithms and models
- `models::*` - Application data models
- Integration between core algorithms and UI needs

## User-Facing Functionality
- Personalized learning experiences
- Adaptive difficulty progression
- Intelligent hint systems
- Learning progress optimization
- Performance-based task selection