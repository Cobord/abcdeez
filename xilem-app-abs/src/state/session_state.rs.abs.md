# state/session_state.rs - Learning Session State

## Conceptual Overview
Manages the state of an active learning session, including task progression, performance tracking, and struggle detection. Integrates with abcdeez_core topology system.

## Key Data Flows
- Tracks current task and pending responses
- Monitors learner struggle indicators (consecutive errors, response times)
- Maintains performance buffer for adaptive learning
- Integrates with core topology system

## Main Responsibilities
- Session lifecycle management (start time, task count)
- Current task state tracking
- Struggle detection and hint level management
- Performance metrics collection
- Integration with abcdeez_core topology

## Dependencies on Other Components
- `abcdeez_core::core::Topology` - Core learning topology
- `models::*` - Task, response, and metrics models

## User-Facing Functionality
- Continuous learning session experience
- Adaptive difficulty based on struggle detection
- Progressive hint system
- Performance tracking for improvement insights