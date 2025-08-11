# /src - Root Source Directory Dependencies

## Backend (web-backend) Requirements

### Authentication Endpoints
- POST `/api/auth/login` - User authentication with username/password
- POST `/api/auth/register` - New user account creation
- POST `/api/auth/refresh` - Token refresh for session management  
- POST `/api/auth/logout` - Session termination

### Session Management Endpoints
- POST `/api/sessions` - Create new learning sessions with topology data
- GET `/api/sessions/{id}` - Retrieve session state and progress
- POST `/api/sessions/{id}/responses` - Submit task responses for processing
- POST `/api/sessions/{id}/complete` - Mark session as completed with summary

### Task Generation Endpoints
- GET `/api/tasks/generate` - Generate adaptive tasks based on learner model
- GET `/api/tasks/hint` - Request contextual hints for current tasks

### Learner Management Endpoints
- POST `/api/learners` - Create learner profiles
- GET `/api/learners/{id}/stats` - Retrieve learner analytics and performance

### Synchronization Endpoints
- POST `/api/sync/push` - Upload offline response data
- GET `/api/sync/pull` - Download remote updates and changes

### Gamification Endpoints
- GET `/api/gamification/profile` - User XP, levels, and achievements
- GET `/api/gamification/leaderboard` - Competitive rankings

### Real-time Communication
- WebSocket `/api/sessions/{id}/live` - Live session communication for:
  - Real-time task delivery
  - Immediate response feedback
  - Adaptive interventions
  - Performance updates

## Core Library (abcdeez_core) Requirements

### Learning Models
- `LearnerModel` - Individual learner proficiency tracking
- `BayesianLearnerModel` - Statistical learning model
- `AdaptiveScheduler` - Task selection algorithms

### Task Systems
- `tasks::core::*` - Core learning task types (successor, ordering, etc.)
- `tasks::extended::*` - Extended task types (boundary bridging, etc.) 
- `tasks::music::*` - Music theory learning tasks
- Task generation and difficulty calibration

### Topology System
- `core::Topology` - Learning space structure definition
- Linear, hierarchical, and custom topology support

### Interaction Tracking
- `data::interaction_tracking::*` - Detailed user behavior analytics
- Keystroke dynamics, mouse tracking, hesitation analysis
- Typing dynamics and interaction quality metrics

### Data Management
- Response tracking and analytics
- Performance metrics calculation
- Learning curve analysis

## Data Structure Requirements

### User and Authentication
```rust
User {
    id: Uuid,
    username: String,
    email: String,
    created_at: DateTime<Utc>,
    metadata: Option<serde_json::Value>,
}

AuthResponse {
    access_token: String,
    refresh_token: String,
    user: User,
}
```

### Learning Sessions
```rust
Session {
    id: Uuid,
    learner_id: Uuid,
    topology_type: String,
    topology_data: serde_json::Value,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    status: String, // "active", "completed", "abandoned"
    summary: Option<serde_json::Value>,
}
```

### Tasks and Responses
```rust
Task {
    task_type: TaskType, // Core, Extended, Music, Navigation, Boundary
    prompt: String,
    correct_answer: String,
    options: Vec<String>,
    difficulty: f64,
}

Response {
    id: Uuid,
    session_id: Uuid,
    sequence_number: i32,
    task_type: String,
    task_data: serde_json::Value,
    user_answer: Option<String>,
    correct: bool,
    response_time_ms: u128,
    hint_level: Option<u32>,
    timestamp: DateTime<Utc>,
}
```

## Business Logic Dependencies

### Adaptive Learning Integration
- Integration with abcdeez_core adaptive algorithms
- Custom intervention logic for UI-specific needs
- Learner model synchronization with backend
- Performance metrics calculation and reporting

### Real-time Communication Protocol
- WebSocket message types for live sessions
- Task delivery and response submission protocols
- Intervention and hint delivery systems
- Connection health and recovery management

### Offline Functionality
- Local SQLite database for persistence
- Response queuing for offline scenarios
- Conflict resolution for concurrent updates
- Background synchronization processes