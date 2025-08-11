# /src/services - Business Logic Services Dependencies

## Backend (web-backend) Requirements

### API Client Service (`api.rs`)
Complete REST API coverage:
- Authentication endpoints with token management
- Session CRUD operations with real-time updates
- Task generation and hint request handling
- Learner management and analytics
- Synchronization endpoints for offline support
- Gamification and leaderboard integration
- Error handling and retry logic

### WebSocket Service (`websocket.rs`)
Real-time communication protocol:
- Authenticated WebSocket connections
- Live task delivery and response submission
- Real-time feedback and intervention messages
- Connection health monitoring and reconnection
- Session state synchronization
- Performance and statistics updates

### Synchronization Service (`sync.rs`)
Offline data management:
- Batch response upload with error handling
- Remote update integration and conflict resolution
- Background synchronization scheduling
- Network failure recovery and retry logic

## Core Library (abcdeez_core) Requirements

### Adaptive Learning Service (`adaptive_learning.rs`)
- `AdaptiveScheduler` - Task selection algorithms
- `LearnerModel` - Individual learner proficiency tracking
- `BayesianLearnerModel` - Statistical learning models
- `core::Topology` - Learning space structure
- Task type integration and conversion
- Performance metrics calculation

### Interaction Tracking Service (`interaction_tracking.rs`)
- `InteractionTracker` - Behavioral data capture
- `InteractionSession` - Session-level interaction data
- Keystroke dynamics and mouse tracking
- Hesitation analysis and typing patterns
- Behavioral pattern recognition
- Interaction quality metrics

### Storage Service (`storage.rs`)
- Local data persistence with SQLite
- abcdeez_core model serialization/deserialization
- Cache management for performance optimization

## Data Structure Requirements

### Service Integration Models
```rust
// API communication
ApiClient - HTTP client with authentication
WebSocketClient - Real-time bidirectional communication
SyncService - Offline data queue management

// Timing and metrics
PrecisionTimer - High-resolution timing measurement
TaskTimer - Task-specific timing events
SessionTimer - Session-level timing with pause/resume

// Interaction tracking
InteractionMetrics - Behavioral analytics data
TrackedEvent - User interaction event capture
```

### Performance and Analytics
```rust
// Comprehensive timing data
TaskTimingResult {
    total_time_ms: u128,
    first_input_delay_ms: Option<u128>,
    hint_requested_delay_ms: Option<u128>,
}

// Learning performance metrics  
LearnerMetrics {
    overall_proficiency: f64,
    operation_proficiencies: HashMap<String, f64>,
    bidirectionality_index: f64,
    accuracy_rate: f64,
}
```

## Business Logic Dependencies

### Authentication and Security
- Token-based authentication with refresh cycle
- Secure credential storage and management
- OAuth integration for third-party authentication
- Session security and timeout handling

### Learning Algorithm Integration
- Real-time adaptive task selection
- Learner model updates and synchronization
- Performance-based difficulty adjustment
- Custom intervention and hint systems

### Data Persistence and Sync
- Local SQLite database management
- Offline queue with conflict resolution
- Background synchronization processes
- Data integrity and recovery mechanisms

### Real-time Communication
- WebSocket protocol implementation
- Message queuing and delivery guarantees
- Connection state management and recovery
- Live session coordination

### Analytics and Monitoring
- Detailed interaction behavior tracking
- Performance metrics collection and analysis
- Learning analytics data generation
- System performance monitoring

### Integration Requirements
- Service orchestration and dependency management
- Error handling and recovery strategies
- Resource management and optimization
- Testing and debugging support