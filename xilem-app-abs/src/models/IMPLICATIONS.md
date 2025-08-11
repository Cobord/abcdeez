# /src/models - Data Models Dependencies

## Backend (web-backend) Requirements

### API Data Transfer Objects
All models must be serializable/deserializable for API communication:

### User and Authentication Models
```rust
// API endpoints: /auth/*, /users/*
User, Learner - User profile and learner analytics data
AuthResponse - Authentication token and user data
```

### Session and Task Models  
```rust
// API endpoints: /sessions/*, /tasks/*
Session - Learning session metadata and lifecycle
Task - Task data with multiple type variants
TaskResponse - User response submission format
```

### Response and Analytics Models
```rust
// API endpoints: /sessions/*/responses, /sync/*
Response - Complete response data with metrics
PendingResponse - Offline response queue format
ResponseMetrics - Performance analytics data
```

### Synchronization Models
```rust
// API endpoints: /sync/*
Sync request/response formats for offline data
Conflict resolution data structures
```

## Core Library (abcdeez_core) Requirements

### Task Type Integration
```rust
TaskType::Core(CoreTaskType) - Integration with abcdeez_core::tasks::core
TaskType::Extended(ExtendedTaskType) - Integration with abcdeez_core::tasks::extended
TaskType::Music(MusicTask) - Music theory task support
TaskType::Navigation(NavigationTask) - Spatial navigation tasks
TaskType::Boundary(BoundaryTask) - Boundary detection tasks
```

### Data Analysis Integration
- Response data compatible with abcdeez_core analytics
- Performance metrics aligned with core learning algorithms
- Task difficulty calibration data structures

## Data Structure Requirements

### Serialization Compatibility
All models require:
- Serde serialization/deserialization
- JSON format compatibility for API/storage
- UUID support for unique identification
- DateTime handling with UTC timezone

### Task Data Flexibility
```rust
Task {
    task_data: serde_json::Value, // Flexible task-specific data
    // Support for various task type parameters
    // Integration with visual presentation systems
}
```

### Response Tracking
```rust
Response/PendingResponse {
    // High-precision timing data
    response_time_ms: u128,
    // Sequence tracking for session analysis
    sequence_number: i32,
    // Rich task context preservation
    task_data: serde_json::Value,
}
```

### User Profile Integration
```rust
User/Learner {
    // Flexible metadata for extensibility
    metadata: Option<serde_json::Value>,
    // Practice time tracking for analytics
    total_practice_time_seconds: i64,
}
```

## Business Logic Dependencies

### Data Validation
- Input sanitization and validation rules
- Data integrity constraints
- Business rule enforcement in model layer

### Analytics Integration
- Response data aggregation for learning analytics
- Performance trend calculation
- User progress tracking and reporting

### Caching and Performance
- Model data optimized for local caching
- Efficient serialization for network transmission
- Memory-efficient data structures

### Migration and Versioning
- Data model versioning support
- Migration strategies for model updates
- Backward compatibility considerations