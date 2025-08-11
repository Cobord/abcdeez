# state/app_state.rs - Root Application State

## Conceptual Overview
The central state container for the entire application. Manages user authentication, current sessions, navigation, UI state, offline sync, WebSocket connections, and performance metrics.

## Key Data Flows
- Integrates with abcdeez_core LearnerModel for adaptive learning
- Manages authentication tokens and user data
- Handles session state and current learning tasks
- Queues responses for offline sync
- Tracks WebSocket connection status
- Collects performance metrics (frame times, response times)

## Main Responsibilities
- Global application state management
- Navigation stack and current screen tracking
- Loading states for different operations
- Error handling and display
- Theme management
- Offline data synchronization queue
- WebSocket connection management
- Performance monitoring

## Dependencies on Other Components
- `abcdeez_core::learning::learner::LearnerModel` - Core learning model
- `models::*` - Data models for tasks, responses, users
- `services::AdaptiveLearningService` - Learning algorithm integration
- `state::*` - User, session, and theme state

## User-Facing Functionality
- Seamless navigation between screens
- Authentication and user management
- Learning session continuity
- Offline capability with sync
- Real-time updates via WebSocket
- Performance monitoring for optimization