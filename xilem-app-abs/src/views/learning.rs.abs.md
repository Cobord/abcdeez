# views/learning.rs - Interactive Learning Interface

## Conceptual Overview
Core learning experience interface that presents adaptive tasks, manages user responses, tracks performance, and provides feedback. Supports multiple task types with specialized visual presentations and real-time progress tracking.

## Key Data Flows
- Displays current learning tasks with visual presentations
- Captures user responses and timing data
- Updates struggle indicators and performance metrics
- Manages hint system and task progression
- Integrates with adaptive learning algorithms

## Main Responsibilities
- Task presentation with specialized visuals
- Response capture and submission
- Real-time performance tracking (accuracy, timing)
- Progress indication and session management
- Hint system integration
- Loading states for task transitions
- Session creation and management

## Dependencies on Other Components
- `models::Task` - Task data structures
- `state::SessionState` - Learning session management
- `views::core_task_visual` - Core task visualizations
- `views::extended_task_visual` - Extended task visualizations
- Integration with abcdeez_core for task generation

## User-Facing Functionality
- Interactive learning task experiences
- Multiple task types (core, extended, music, navigation, boundary)
- Real-time feedback and performance tracking
- Adaptive hint system
- Progress visualization during sessions
- Seamless task transitions and loading