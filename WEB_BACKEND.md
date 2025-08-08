# Web Backend Architecture Design

## Overview

The web backend provides a RESTful API and real-time websocket interface for the graph-coded learning system. It's designed to support both individual learners and collaborative research studies with population-level analytics.

## Architecture Principles

1. **Separation of Concerns**: Core learning logic (graph-learning-core) is independent of API layer
2. **Event Sourcing**: All learner interactions are stored as immutable events
3. **Real-time Adaptation**: WebSocket connections for live struggle detection and hints
4. **Privacy by Design**: Differential privacy for population analytics
5. **Horizontal Scalability**: Stateless API servers with shared database

## Technology Stack

- **Web Framework**: Axum (async, type-safe, fast)
- **Database**: SQLite for development, PostgreSQL for production
- **ORM**: SQLx (compile-time checked queries)
- **WebSockets**: Axum + Tokio for real-time features
- **Cache**: Redis for session state and real-time metrics
- **Authentication**: JWT tokens with refresh mechanism
- **Background Jobs**: Tokio tasks for async processing

## Database Schema

### Core Tables

```sql
-- Users and Authentication
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    metadata JSONB
);

-- Learners (can be anonymous)
CREATE TABLE learners (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    display_name TEXT,
    created_at TIMESTAMP NOT NULL,
    last_active TIMESTAMP,
    total_practice_time_seconds BIGINT DEFAULT 0,
    metadata JSONB
);

-- Training Sessions
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    learner_id UUID NOT NULL REFERENCES learners(id),
    topology_type TEXT NOT NULL,
    topology_data JSONB NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    status TEXT NOT NULL, -- active, completed, abandoned
    summary JSONB,
    INDEX idx_sessions_learner (learner_id),
    INDEX idx_sessions_status (status)
);

-- Individual Task Responses (Event Store)
CREATE TABLE responses (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES sessions(id),
    sequence_number INTEGER NOT NULL,
    task_type TEXT NOT NULL,
    task_data JSONB NOT NULL,
    user_answer TEXT,
    correct BOOLEAN NOT NULL,
    response_time_ms INTEGER NOT NULL,
    hint_level INTEGER, -- 0=none, 1-4=hint levels
    timestamp TIMESTAMP NOT NULL,
    INDEX idx_responses_session (session_id, sequence_number)
);

-- Model Snapshots (for analysis and recovery)
CREATE TABLE model_snapshots (
    id UUID PRIMARY KEY,
    learner_id UUID NOT NULL REFERENCES learners(id),
    session_id UUID REFERENCES sessions(id),
    timestamp TIMESTAMP NOT NULL,
    parameters JSONB NOT NULL, -- Full learner model state
    metrics JSONB,
    INDEX idx_snapshots_learner (learner_id, timestamp DESC)
);

-- Experiments and Studies
CREATE TABLE experiments (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    config JSONB NOT NULL, -- Experiment configuration
    start_date DATE,
    end_date DATE,
    created_by UUID REFERENCES users(id),
    status TEXT NOT NULL -- draft, active, completed, archived
);

CREATE TABLE experiment_participants (
    experiment_id UUID REFERENCES experiments(id),
    learner_id UUID REFERENCES learners(id),
    condition TEXT, -- Control, treatment_a, treatment_b, etc.
    joined_at TIMESTAMP NOT NULL,
    metadata JSONB,
    PRIMARY KEY (experiment_id, learner_id)
);

-- Intervention Events
CREATE TABLE interventions (
    id UUID PRIMARY KEY,
    session_id UUID REFERENCES sessions(id),
    timestamp TIMESTAMP NOT NULL,
    intervention_type TEXT NOT NULL, -- hint, difficulty_change, break_suggestion
    details JSONB,
    effectiveness BOOLEAN -- Was the next response correct?
);
```

## API Endpoints

### Authentication
```
POST   /api/auth/register         # Create new user account
POST   /api/auth/login           # Login with credentials
POST   /api/auth/refresh         # Refresh JWT token
POST   /api/auth/logout          # Invalidate refresh token
GET    /api/auth/me             # Get current user info
```

### Learner Management
```
POST   /api/learners                    # Create learner profile
GET    /api/learners/:id               # Get learner details
PATCH  /api/learners/:id               # Update learner
GET    /api/learners/:id/stats         # Get performance statistics
GET    /api/learners/:id/export        # Export all learner data (GDPR)
DELETE /api/learners/:id               # Delete learner (GDPR)
```

### Sessions
```
POST   /api/sessions                    # Start new session
GET    /api/sessions/:id               # Get session details
POST   /api/sessions/:id/responses     # Submit task response
POST   /api/sessions/:id/complete      # Mark session complete
GET    /api/sessions/:id/replay        # Get session replay data
WS     /api/sessions/:id/live          # WebSocket for real-time session
```

### Tasks and Adaptation
```
GET    /api/tasks/next                  # Get next adaptive task
POST   /api/tasks/generate             # Generate specific task type
GET    /api/tasks/difficulty           # Get difficulty calibration
POST   /api/hints/request              # Request hint for current task
```

### Analytics
```
GET    /api/analytics/population       # Population-level statistics
GET    /api/analytics/bottlenecks      # Common difficulty points
GET    /api/analytics/strategies       # Strategy distribution
GET    /api/analytics/learning-curves  # Aggregated learning curves
POST   /api/analytics/compare          # Compare learner groups
GET    /api/analytics/live             # Real-time dashboard data
```

### Experiments
```
GET    /api/experiments                 # List experiments
POST   /api/experiments                 # Create experiment
GET    /api/experiments/:id            # Get experiment details
POST   /api/experiments/:id/join       # Join experiment
GET    /api/experiments/:id/results    # Get experiment results
POST   /api/experiments/:id/export     # Export experiment data
```

### Music Domain
```
GET    /api/music/scales               # List available scales
GET    /api/music/progressions         # List chord progressions
POST   /api/music/tasks                # Generate music task
GET    /api/music/audio/:note          # Get audio sample for note
```

## WebSocket Protocol

### Session WebSocket (`/api/sessions/:id/live`)

#### Client -> Server Messages
```typescript
interface ClientMessage {
  type: 'start_task' | 'submit_response' | 'request_hint' | 'pause' | 'resume';
  payload: any;
  timestamp: number;
}
```

#### Server -> Client Messages
```typescript
interface ServerMessage {
  type: 'task' | 'hint' | 'feedback' | 'intervention' | 'stats_update';
  payload: any;
  timestamp: number;
}
```

### Real-time Analytics WebSocket (`/api/analytics/live`)

Broadcasts population-level metrics every second:
```typescript
interface LiveMetrics {
  active_learners: number;
  tasks_per_minute: number;
  average_accuracy: number;
  difficulty_distribution: Record<string, number>;
  strategy_distribution: Record<string, number>;
}
```

## Data Flow Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│  API Server │────▶│  Database   │
│  (Xilem UI) │     │   (Axum)    │     │ (PostgreSQL)│
└─────────────┘     └─────────────┘     └─────────────┘
       │                    │                    ▲
       │                    ▼                    │
       │            ┌─────────────┐             │
       └───────────▶│    Redis    │─────────────┘
         WebSocket  │   (Cache)   │   Model State
                    └─────────────┘
                            │
                            ▼
                    ┌─────────────┐
                    │  Analytics  │
                    │   Worker    │
                    └─────────────┘
```

## Service Layer Design

### Core Services

```rust
// learner_service.rs
pub struct LearnerService {
    db: Arc<PgPool>,
    cache: Arc<RedisClient>,
}

impl LearnerService {
    pub async fn create_learner(&self, req: CreateLearnerRequest) -> Result<Learner>;
    pub async fn get_learner(&self, id: Uuid) -> Result<Learner>;
    pub async fn update_model(&self, id: Uuid, response: TaskResponse) -> Result<()>;
    pub async fn get_model(&self, id: Uuid) -> Result<LearnerModel>;
}

// adaptation_service.rs
pub struct AdaptationService {
    learner_service: Arc<LearnerService>,
}

impl AdaptationService {
    pub async fn select_next_task(&self, learner_id: Uuid) -> Result<Task>;
    pub async fn calculate_eig(&self, learner_id: Uuid, task: &Task) -> Result<f64>;
    pub async fn should_intervene(&self, learner_id: Uuid, elapsed_ms: u64) -> Result<Intervention>;
}

// analytics_service.rs
pub struct AnalyticsService {
    db: Arc<PgPool>,
    cache: Arc<RedisClient>,
}

impl AnalyticsService {
    pub async fn population_stats(&self) -> Result<PopulationStats>;
    pub async fn find_bottlenecks(&self, min_samples: usize) -> Result<Vec<Bottleneck>>;
    pub async fn compare_conditions(&self, experiment_id: Uuid) -> Result<ConditionComparison>;
}
```

## Real-time Features

### Live Struggle Detection
```rust
pub struct StruggleMonitor {
    sessions: Arc<DashMap<Uuid, SessionState>>,
}

impl StruggleMonitor {
    pub async fn monitor_response_time(&self, session_id: Uuid, elapsed_ms: u64) {
        if let Some(session) = self.sessions.get(&session_id) {
            if session.struggle_detector.should_provide_hint(elapsed_ms) {
                self.send_hint(session_id, HintLevel::from_elapsed(elapsed_ms)).await;
            }
        }
    }
}
```

### Adaptive Difficulty Adjustment
```rust
pub struct DifficultyManager {
    target_success_rate: f64, // 0.75
    adjustment_window: usize, // 10 responses
}

impl DifficultyManager {
    pub async fn adjust_difficulty(&self, learner_id: Uuid, recent_responses: &[bool]) -> f64 {
        let success_rate = recent_responses.iter().filter(|&&x| x).count() as f64 
            / recent_responses.len() as f64;
        
        if success_rate > self.target_success_rate + 0.1 {
            self.increase_difficulty()
        } else if success_rate < self.target_success_rate - 0.1 {
            self.decrease_difficulty()
        } else {
            self.current_difficulty()
        }
    }
}
```

## Privacy and Security

### Data Protection
- **Encryption**: All PII encrypted at rest using AES-256
- **Anonymization**: Learner IDs are UUIDs, no PII required
- **Audit Logging**: All data access logged for compliance
- **Data Retention**: Automatic deletion after configured period

### Differential Privacy
```rust
pub struct PrivateAnalytics {
    epsilon: f64, // Privacy budget
    sensitivity: f64, // Query sensitivity
}

impl PrivateAnalytics {
    pub fn add_noise(&self, true_value: f64) -> f64 {
        let scale = self.sensitivity / self.epsilon;
        let noise = Laplace::new(0.0, scale).sample(&mut thread_rng());
        true_value + noise
    }
}
```

## Deployment Architecture

### Docker Compose (Development)
```yaml
version: '3.8'
services:
  api:
    build: ./web-backend
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: "postgresql://user:pass@db:5432/graph_learning"
      REDIS_URL: "redis://redis:6379"
    depends_on:
      - db
      - redis
      
  db:
    image: postgres:15
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: graph_learning
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
      
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
```

### Kubernetes (Production)
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: graph-learning-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: graph-learning-api
  template:
    metadata:
      labels:
        app: graph-learning-api
    spec:
      containers:
      - name: api
        image: graph-learning/api:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
```

## Performance Optimization

### Caching Strategy
1. **Session State**: Redis with 1-hour TTL
2. **Learner Models**: Redis with write-through to DB
3. **Analytics**: Pre-computed and cached hourly
4. **Static Assets**: CDN for audio/images

### Database Optimization
1. **Indexes**: On foreign keys and common query patterns
2. **Partitioning**: Response table by month
3. **Connection Pooling**: SQLx with 20 connections
4. **Read Replicas**: For analytics queries

### API Performance
1. **Rate Limiting**: 100 requests/minute per user
2. **Response Compression**: Gzip for JSON responses
3. **Batch Operations**: Bulk insert for responses
4. **Async Processing**: Background jobs for heavy computation

## Monitoring and Observability

### Metrics (Prometheus)
```rust
static TASK_COUNTER: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("tasks_completed_total", "Total tasks completed").unwrap()
});

static RESPONSE_TIME: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!("task_response_time_seconds", "Task response time").unwrap()
});
```

### Logging (OpenTelemetry)
```rust
#[instrument(skip(db))]
pub async fn submit_response(
    db: &PgPool,
    session_id: Uuid,
    response: TaskResponse,
) -> Result<()> {
    info!("Processing response for session {}", session_id);
    // ...
}
```

### Health Checks
```
GET /health/live     # Kubernetes liveness probe
GET /health/ready    # Kubernetes readiness probe
GET /metrics        # Prometheus metrics endpoint
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_adaptive_task_selection() {
        let service = create_test_service().await;
        let task = service.select_next_task(test_learner_id()).await.unwrap();
        assert_eq!(task.difficulty, 0.5);
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_full_session_flow() {
    let app = create_test_app().await;
    
    // Start session
    let response = app.post("/api/sessions")
        .json(&json!({"topology": "alphabet"}))
        .send()
        .await;
    assert_eq!(response.status(), 201);
    
    // Submit responses
    // ...
}
```

### Load Testing (k6)
```javascript
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 100,
  duration: '30s',
};

export default function() {
  let response = http.get('http://localhost:3000/api/tasks/next');
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
```

## Migration Strategy

### From CLI to Web
1. Export learner data from CLI using export module
2. Import into web backend via batch API
3. Maintain backward compatibility for data formats
4. Provide migration tools for researchers

### Database Migrations (SQLx)
```sql
-- migrations/001_initial_schema.sql
CREATE TABLE users (...);

-- migrations/002_add_interventions.sql
CREATE TABLE interventions (...);

-- migrations/003_add_music_domain.sql
ALTER TABLE tasks ADD COLUMN music_context JSONB;
```

## Future Enhancements

1. **GraphQL API**: For flexible querying
2. **ML Pipeline**: Real-time model training
3. **Voice Interface**: Speech recognition for responses
4. **Multiplayer**: Collaborative learning sessions
5. **VR/AR Support**: Spatial graph navigation
6. **Blockchain**: Verifiable learning credentials