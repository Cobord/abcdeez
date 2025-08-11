# /src/state - State Management Dependencies

## Backend (web-backend) Requirements

### User Authentication and Profile
- User profile data synchronization
- Authentication token management and refresh
- Gamification data (levels, XP, achievements, streaks)
- User preferences and settings persistence

### Session State Management
- Session lifecycle tracking (start/end times)
- Current task state synchronization
- Performance buffer data aggregation
- Struggle indicator analysis and intervention triggers

### Theme and UI Preferences
- User theme preference persistence
- Cross-device theme synchronization
- Accessibility settings storage

## Core Library (abcdeez_core) Requirements

### Learning Models Integration
- `LearnerModel` embedding in application state
- Real-time model updates during task progression
- Model serialization for persistence and sync

### Topology Integration
- `core::Topology` integration for session structure
- Topology configuration and customization
- Dynamic topology switching during sessions

### Performance Tracking
- Response metrics integration with core analytics
- Learning curve data generation
- Proficiency tracking across different operations

## Data Structure Requirements

### Application State Persistence
```rust
// Local storage requirements
AppState {
    user: Option<UserState>,
    auth_token: Option<String>,
    refresh_token: Option<String>,
    session: Option<SessionState>,
    learner_model: Option<LearnerModel>,
    theme: Theme,
    sync_queue: Vec<PendingResponse>,
    connection_status: ConnectionStatus,
}
```

### Session State Coordination
```rust
// Backend session synchronization
SessionState {
    session_id: Uuid,
    learner_id: Uuid, 
    topology: abcdeez_core::core::Topology,
    start_time: DateTime<Utc>,
    current_task: Option<Task>,
    performance_buffer: Vec<ResponseMetrics>,
    struggle_indicators: StruggleState,
}
```

### User State Management
```rust
// Profile and gamification sync
UserState {
    id: Uuid,
    username: String,
    email: String,
    learner_id: Option<Uuid>,
    level: u32,
    xp: u32,
    streak: u32,
    achievements: Vec<String>,
}
```

## Business Logic Dependencies

### State Synchronization
- Real-time state updates via WebSocket
- Conflict resolution for concurrent state changes
- Offline state queuing and replay
- State migration and versioning support

### Navigation and Flow Control
- Screen transition validation
- Session continuity across navigation
- Authentication state enforcement
- Error state handling and recovery

### Performance and Analytics
- State change performance monitoring
- User interaction pattern tracking
- State-based adaptive behavior
- Memory and resource optimization

### Integration Points
- Integration with local SQLite storage
- API client state coordination
- WebSocket state synchronization
- Background sync process coordination