# Web Backend Specification

## Overview

The web-backend module is a comprehensive REST API and WebSocket server built with Axum that provides the complete backend infrastructure for the ABCDEEZ learning platform. It integrates with the core learning module and provides authentication, session management, real-time communication, analytics, federation capabilities, and extensive monitoring.

## Architecture

### Technology Stack
- **Framework**: Axum 0.7 (async Rust web framework)
- **Database**: SQLite (default) / PostgreSQL (optional via feature flag)
- **Cache**: In-memory Redis-compatible cache (no external Redis required)
- **Authentication**: JWT with HS256, OAuth 2.0 (Apple, GitHub)
- **WebSocket**: tokio-tungstenite for real-time communication
- **TLS**: rustls with Let's Encrypt support via ACME
- **Monitoring**: Prometheus metrics, OpenTelemetry tracing

### Module Structure
```
web-backend/
├── src/
│   ├── main.rs           # Application entry point
│   ├── lib.rs            # Library interface for testing
│   ├── config.rs         # Configuration management
│   ├── state.rs          # Application state container
│   ├── db.rs             # Database pool and migrations
│   ├── cache.rs          # In-memory Redis-like cache
│   ├── error.rs          # Error types and handling
│   ├── tls.rs            # TLS/SSL management
│   ├── handlers/         # HTTP request handlers
│   ├── middleware/       # Request/response middleware
│   ├── models/           # Data models and DTOs
│   ├── services/         # Business logic services
│   ├── monitoring/       # Health, metrics, performance
│   └── websocket/        # WebSocket handlers
```

## API Endpoints

### Authentication & Authorization

#### User Management
- `POST /api/auth/register` - Register new user
  - Request: `{username, email, password}`
  - Response: `{access_token, refresh_token, user}`
  - Validates password strength, checks for existing users

- `POST /api/auth/login` - User login
  - Request: `{username, password}`
  - Response: `{access_token, refresh_token, user}`
  - Implements rate limiting, account lockout after failed attempts

- `POST /api/auth/refresh` - Refresh access token
  - Request: `{refresh_token}`
  - Response: `{access_token, refresh_token}`

- `POST /api/auth/logout` - Logout and blacklist tokens
  - Adds session to blacklist cache

- `GET /api/auth/me` - Get current user info
  - Requires: Bearer token
  - Response: User profile data

#### OAuth Integration
- `GET /api/auth/oauth/:provider/authorize` - Get OAuth authorization URL
  - Providers: apple, github
  - Response: `{authorization_url, state, provider}`

- `POST /api/auth/oauth/callback` - Handle OAuth callback
  - Request: `{code, state, provider}`
  - Creates or links user account

- `POST /api/auth/apple/signin` - Apple Sign In
  - Request: `{identity_token, authorization_code, user_info}`
  - Validates Apple JWT, creates/updates user

### Learner Management

- `POST /api/learners` - Create learner profile
  - Request: `{display_name, user_id?}`
  - Creates learner with core::LearnerModel integration

- `GET /api/learners/:id` - Get learner details
- `PATCH /api/learners/:id` - Update learner
- `DELETE /api/learners/:id` - Delete learner
- `GET /api/learners/:id/stats` - Get learner statistics
  - Response: `{total_sessions, total_tasks_completed, overall_accuracy, total_practice_time_seconds, last_active, preferred_difficulty, learning_curve}`

- `GET /api/learners/:id/sessions` - List learner's sessions
- `GET /api/learners/:id/export` - Export learner data

### Session Management

- `POST /api/sessions` - Create training session
  - Request: `{learner_id, topology_type, topology_data?}`
  - Creates session with specified task topology

- `GET /api/sessions/:id` - Get session details
- `POST /api/sessions/:id/responses` - Submit task response
  - Request: `{task_type, task_data, user_answer, response_time_ms}`
  - Stores response, updates learner model

- `GET /api/sessions/:id/responses` - Get session responses
- `POST /api/sessions/:id/complete` - Mark session complete
- `GET /api/sessions/:id/replay` - Get session replay data

### Task Generation

- `GET /api/tasks/generate` - Generate task
  - Query params: learner_id, topology, difficulty
  - Uses core module's task generation

- `GET /api/tasks/difficulty` - Get recommended difficulty
- `GET /api/tasks/hint` - Generate hint for current task
  - Query params: learner_id, task_type, hint_level

### Analytics

All analytics endpoints require `analytics_access` permission or admin role.

- `GET /api/analytics/population` - Population-level statistics
- `GET /api/analytics/bottlenecks` - Identify learning bottlenecks
- `GET /api/analytics/strategies` - Strategy analysis
- `GET /api/analytics/learning-curves` - Learning curve data
- `POST /api/analytics/compare` - Compare learner groups
- `GET /api/analytics/live` - Real-time analytics data

#### Advanced Analytics
- `GET /api/analytics/response-time-analysis` - Response time patterns
- `GET /api/analytics/learner/:id/performance` - Individual performance analysis
- `GET /api/analytics/population/strategies` - Population strategy distribution
- `GET /api/analytics/learner/:id/adaptive-difficulty` - Adaptive difficulty analysis

### Experiment Management

- `GET /api/experiments` - List experiments
- `POST /api/experiments` - Create experiment
  - Request: `{name, description, config, start_date, end_date}`
- `GET /api/experiments/:id` - Get experiment details
- `POST /api/experiments/:id/join` - Join experiment
- `GET /api/experiments/:id/results` - Get experiment results
- `POST /api/experiments/:id/export` - Export experiment data

### Protocol Versioning

- `GET /api/protocols` - List protocols
- `POST /api/protocols` - Create protocol
- `GET /api/protocols/:id` - Get protocol
- `POST /api/protocols/:id/versions` - Create version
- `GET /api/protocols/:id/versions` - List versions
- `GET /api/protocols/:id/versions/:version_id` - Get specific version
- `POST /api/protocols/:id/versions/:version_id/publish` - Publish version
- `GET /api/protocols/:id/versions/:version_id/validate` - Validate protocol
- `POST /api/protocols/:id/compare` - Compare versions
- `POST /api/protocols/:id/history` - Version history
- `POST /api/protocols/:id/branches` - Create branch
- `GET /api/protocols/:id/metrics` - Protocol metrics

### Federation Network

- `GET /api/federation/nodes` - List federation nodes
- `POST /api/federation/nodes` - Register new node
  - Request: `{institution_id, institution_name, base_url, api_version, public_key, capabilities, admin_contact, compliance_certifications}`
  - Response: `{node_id, api_key, our_public_key, status}`

- `GET /api/federation/nodes/:id` - Get node details
- `POST /api/federation/heartbeat` - Node heartbeat
  - Request: `{node_id, timestamp, status, active_experiments, active_learners, api_version}`

- `POST /api/federation/share` - Share data with network
  - Request: `{source_node_id, target_node_ids, data_type, data, signature}`
  - Validates signature, checks agreements

- `POST /api/federation/sync/protocol` - Sync protocol changes
- `POST /api/federation/compliance/verify` - Verify compliance status
- `GET /api/federation/stats` - Network statistics
- `GET /api/federation/agreements` - List agreements
- `POST /api/federation/agreements` - Create agreement

### Gamification

- `GET /api/gamification/profile` - Get gamification profile
- `GET /api/gamification/achievements` - List achievements
- `POST /api/gamification/achievements/:id/unlock` - Unlock achievement
- `GET /api/gamification/leaderboard` - Get leaderboard
- `POST /api/gamification/leaderboard/update` - Update score
- `POST /api/gamification/xp/add` - Add experience points

### Cloud Sync

- `POST /api/sync/devices` - Register device
- `GET /api/sync/status` - Sync status
- `GET /api/sync/pull` - Pull changes
- `POST /api/sync/push` - Push changes
- `POST /api/sync/conflicts/resolve` - Resolve conflicts

### Music Domain

- `GET /api/music/scales` - List scales
- `GET /api/music/progressions` - Chord progressions
- `POST /api/music/tasks` - Generate music tasks
- `GET /api/music/audio/:note` - Get audio for note

### Admin Panel

All admin endpoints require admin role.

- `GET /api/admin/dashboard` - Admin dashboard data
- `GET /api/admin/users` - List users
- `GET /api/admin/learners` - List all learners
- `GET /api/admin/audit` - Audit trail
- `GET /api/admin/jobs` - List background jobs
- `POST /api/admin/jobs` - Trigger job
- `POST /api/admin/oauth-validation` - Validate OAuth credentials
- `POST /api/admin/config` - Update configuration
- `GET /api/admin/audit-report` - Generate audit report

#### Business Metrics (Admin)
- `GET /api/admin/business/dashboard` - Business dashboard
- `GET /api/admin/business/daily-metrics` - Daily metrics
- `GET /api/admin/business/learning-effectiveness` - Learning effectiveness
- `GET /api/admin/business/user-journey` - User journey analytics
- `GET /api/admin/business/revenue` - Revenue metrics
- `GET /api/admin/business/export` - Export business data

#### Migration Management (Admin)
- `GET /api/admin/migrations/status` - Migration status
- `POST /api/admin/migrations/run` - Run migrations
- `POST /api/admin/migrations/rollback` - Rollback to version
- `POST /api/admin/migrations/rollback-last` - Rollback last migration
- `GET /api/admin/migrations/validate` - Validate migrations
- `POST /api/admin/migrations/backup` - Create backup
- `GET /api/admin/migrations/history` - Migration history
- `GET /api/admin/migrations/preview/:version` - Preview rollback

#### Audit Retention (Admin)
- `GET /api/admin/audit/retention/statistics` - Retention statistics
- `POST /api/admin/audit/retention/cleanup` - Apply retention policies
- `GET /api/admin/audit/retention/compliance-report` - Compliance report
- `GET /api/admin/audit/retention/policies` - List policies
- `PUT /api/admin/audit/retention/policies` - Update policy
- `DELETE /api/admin/audit/retention/policies/:name` - Delete policy
- `GET /api/admin/audit/trail` - Audit trail with retention

### Health & Monitoring

- `GET /health/live` - Simple liveness check
- `GET /health/ready` - Readiness check (validates DB and cache)
- `GET /health/health` - Detailed health check
- `GET /health/health/enhanced` - Enhanced health with component status
- `GET /health/metrics` - Prometheus metrics
- `GET /health/metrics/json` - JSON metrics
- `GET /health/performance` - Performance metrics
- `GET /health/performance/endpoints` - Per-endpoint performance

### Dashboard

- `GET /api/dashboard` - Metrics dashboard HTML
- `GET /api/dashboard/data` - Dashboard data JSON
- `GET /api/dashboard/realtime` - Real-time metrics
- `GET /api/dashboard/otel` - OpenTelemetry metrics
- `GET /api/dashboard/system` - System info
- `GET /api/dashboard/database` - Database report
- `GET /api/dashboard/health` - Detailed health

## WebSocket Protocol

### Session WebSocket (`/api/sessions/:id/live`)

Real-time bidirectional communication during learning sessions.

#### Authentication
First message must be authentication:
```json
{
  "type": "Authenticate",
  "token": "Bearer JWT_TOKEN"
}
```

#### Client Messages
```typescript
type ClientMessage = 
  | { type: "Authenticate", token: string }
  | { type: "StartTask", payload: any }
  | { type: "SubmitResponse", task_type: string, task_data: any, user_answer?: string, response_time_ms: number }
  | { type: "RequestHint", hint_level?: string }
  | { type: "Pause" }
  | { type: "Resume" }
  | { type: "Heartbeat" }
```

#### Server Messages
```typescript
type ServerMessage =
  | { type: "AuthenticationResult", success: boolean, message: string, user_id?: string, timestamp: number }
  | { type: "Task", payload: TaskMessage, timestamp: number }
  | { type: "Hint", payload: HintMessage, timestamp: number }
  | { type: "Feedback", payload: FeedbackMessage, timestamp: number }
  | { type: "Intervention", payload: InterventionMessage, timestamp: number }
  | { type: "StatsUpdate", payload: StatsMessage, timestamp: number }
  | { type: "Error", message: string, timestamp: number }
  | { type: "Heartbeat", timestamp: number }
```

### Analytics WebSocket (`/api/analytics/ws`)

Real-time analytics stream for researchers and admins.

Broadcasts `LiveMetrics` every 5 seconds:
```json
{
  "active_learners": 42,
  "tasks_per_minute": 156.5,
  "average_accuracy": 0.75,
  "difficulty_distribution": {...},
  "strategy_distribution": {...},
  "timestamp": 1234567890
}
```

## Data Models

### Core Entities

#### User
```rust
struct User {
    id: Uuid,
    username: String,
    email: String,
    password_hash: Option<String>, // Optional for OAuth users
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    metadata: Option<Json>,
    apple_user_id: Option<String>,
    github_user_id: Option<String>,
    oauth_provider_id: Option<String>,
    auth_provider: String, // "local", "apple", "github"
    is_private_email: Option<bool>,
}
```

#### Learner
```rust
struct Learner {
    id: Uuid,
    user_id: Option<Uuid>,
    display_name: Option<String>,
    created_at: DateTime<Utc>,
    last_active: Option<DateTime<Utc>>,
    total_practice_time_seconds: i64,
    metadata: Option<Json>,
    learning_model: core::LearnerModel, // Integration with core module
}
```

#### Session
```rust
struct Session {
    id: Uuid,
    learner_id: Uuid,
    topology_type: String,
    topology_data: Json, // Serialized core::Topology
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    status: String, // "active", "completed", "abandoned"
    summary: Option<Json>,
}
```

#### Response
```rust
struct Response {
    id: Uuid,
    session_id: Uuid,
    sequence_number: i32,
    task_type: String,
    task_data: Json,
    user_answer: Option<String>,
    correct: bool,
    response_time_ms: i32,
    hint_level: Option<i32>, // 0=none, 1-4=hint levels
    timestamp: DateTime<Utc>,
}
```

### Federation Models

#### FederationNode
```rust
struct FederationNode {
    id: Uuid,
    institution_id: String,
    institution_name: String,
    base_url: String,
    api_version: String,
    public_key: String,
    status: NodeStatus, // Active, Inactive, Suspended, Pending
    last_heartbeat: Option<DateTime<Utc>>,
    capabilities: Vec<String>,
    metadata: Json,
}
```

#### FederationAgreement
```rust
struct FederationAgreement {
    id: Uuid,
    initiator_node_id: Uuid,
    partner_node_id: Uuid,
    agreement_type: AgreementType, // DataSharing, ResearchCollaboration, ProtocolExchange, Full
    data_sharing_rules: Json,
    compliance_requirements: Vec<String>,
    status: String, // Proposed, Negotiating, Active, Expired, Terminated
    expires_at: Option<DateTime<Utc>>,
}
```

## Services

### Core Services

#### LearnerService
- Manages learner profiles and learning models
- Integrates with core module for model updates
- Provides statistics and performance metrics

#### AdaptationService
- Selects next tasks based on learner model
- Determines interventions (hints, difficulty adjustments)
- Generates contextual hints

#### AnalyticsService
- Calculates population-level metrics
- Identifies learning bottlenecks
- Provides real-time analytics
- Implements differential privacy for sensitive data

#### FederationService
- Manages inter-institutional communication
- Handles node registration and heartbeats
- Validates compliance and agreements
- Synchronizes protocols across network

#### OAuth Services
- AppleAuthService: Apple Sign In integration
- GitHubOAuthService: GitHub OAuth flow
- Validates tokens, manages user linking

#### PrivacyAccountingService
- Tracks privacy budget usage
- Implements differential privacy mechanisms
- Generates privacy reports

#### ProtocolService
- Version control for experiment protocols
- Branching and merging support
- Validation and testing

#### BatchJobService
- Background job processing
- OAuth credential validation
- Scheduled maintenance tasks

## Authentication & Authorization

### JWT Token Structure
```json
{
  "sub": "user_uuid",
  "username": "string",
  "exp": 1234567890,
  "iat": 1234567890,
  "role": "user|researcher|admin",
  "permissions": ["analytics_access", ...],
  "session_id": "optional_session_tracking"
}
```

### Authorization Levels
1. **Public**: No authentication required
2. **Authenticated**: Valid JWT required
3. **Researcher**: Requires researcher role or analytics_access permission
4. **Admin**: Requires admin role

### OAuth Providers
- **Apple Sign In**: Full integration with JWT validation
- **GitHub OAuth**: Standard OAuth 2.0 flow
- Automatic account linking based on email

## Caching & Performance

### In-Memory Cache
- Redis-compatible API without external dependency
- TTL support for all cached values
- Commands: GET, SET, SETEX, DEL, EXISTS, INCR, EXPIRE
- Used for:
  - Rate limiting counters
  - Session blacklists
  - Temporary data
  - Performance optimization

### Performance Features
- Request/response compression (gzip)
- Connection pooling for database
- Async/await throughout
- Efficient binary protocol for WebSocket
- Endpoint-specific rate limiting

## Security

### Middleware Stack
1. **Correlation ID**: Request tracing
2. **Security Headers**: HSTS, CSP, X-Frame-Options, etc.
3. **Rate Limiting**: Global, per-IP, per-user, endpoint-specific
4. **IP Blocking**: Auto-block suspicious IPs
5. **Content Validation**: Type and size validation
6. **Authentication**: JWT validation with blacklist checking
7. **Authorization**: Role and permission-based access control
8. **Audit Logging**: Comprehensive activity logging

### Security Features
- Password strength validation
- Account lockout after failed attempts
- Token blacklisting on logout
- Session tracking and validation
- Automatic HTTPS with Let's Encrypt
- Environment-specific security policies

### Privacy & Compliance
- Differential privacy for analytics
- GDPR-compliant data export
- Audit trail with retention policies
- Compliance tracking for federation nodes
- Privacy budget accounting

## Database Schema

### Core Tables
- `users`: User accounts and authentication
- `learners`: Learner profiles
- `sessions`: Training sessions
- `responses`: Individual task responses
- `model_snapshots`: Learner model checkpoints
- `experiments`: Research experiments
- `experiment_participants`: Experiment enrollment
- `interventions`: Intervention events
- `audit_log`: Comprehensive audit trail

### Federation Tables
- `federation_nodes`: Partner institutions
- `federation_api_keys`: Node authentication
- `federation_agreements`: Data sharing agreements
- `federation_heartbeats`: Node health monitoring
- `federation_transfers`: Data transfer queue
- `federation_compliance`: Compliance records

### Supporting Tables
- `music_contexts`: Music domain data
- `job_queue`: Background job processing
- `protocol_versions`: Protocol version control
- `gamification_profiles`: User achievements
- `cloud_sync_devices`: Device registration

## Configuration

### Environment Variables
```bash
# Core
DATABASE_URL=sqlite://graph_learning.db
REDIS_URL=redis://127.0.0.1:6379  # Not required (in-memory cache)
PORT=3000
JWT_SECRET=minimum_64_character_secret
ENVIRONMENT=development|staging|production

# OAuth
APPLE_CLIENT_ID=com.example.app
APPLE_TEAM_ID=XXXXXXXXXX
APPLE_KEY_ID=XXXXXXXXXX
APPLE_PRIVATE_KEY_PATH=/path/to/key.p8
GITHUB_CLIENT_ID=xxxxxxxxxxxx
GITHUB_CLIENT_SECRET=xxxxxxxxxxxx

# TLS
TLS_ENABLED=true
TLS_DOMAIN=example.com
TLS_USE_LETSENCRYPT=true
ADMIN_EMAIL=admin@example.com

# Security
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_SECONDS=60
MAX_FAILED_LOGIN_ATTEMPTS=5
REQUIRE_STRONG_PASSWORDS=true

# Privacy
PRIVACY_EPSILON=1.0
PRIVACY_DELTA=1e-9
```

### Production Requirements
- JWT secret ≥ 64 characters
- CORS origin explicitly configured (no wildcards)
- OAuth providers fully configured
- HTTPS enforced
- Strong password requirements
- Rate limiting enabled

## Monitoring & Observability

### Health Checks
- `/health/live`: Simple liveness
- `/health/ready`: Full readiness (DB + cache)
- `/health/health`: Detailed component health

### Metrics
- Prometheus format at `/health/metrics`
- JSON format at `/health/metrics/json`
- Per-endpoint performance tracking
- Database query metrics
- Cache hit/miss rates
- WebSocket connection counts

### Tracing
- Correlation IDs for request tracking
- Structured logging with tracing
- OpenTelemetry support (optional)
- Audit logging for all API calls

### Dashboard
- Real-time metrics visualization
- System resource monitoring
- Database performance stats
- Active session tracking

## Integration Points

### Core Module Integration
The web-backend directly integrates with `abcdeez-core`:
- Uses `core::LearnerModel` for adaptive learning
- Generates tasks via `core::Task` system
- Applies `core::Topology` for session structure
- Leverages `core::HintLevel` for assistance
- Implements `core::InterventionAction` for adaptations

### Expected Frontend Interactions
1. **Authentication Flow**:
   - Register/login to get JWT tokens
   - Include Bearer token in all API requests
   - Refresh token before expiration

2. **Session Flow**:
   - Create session with topology
   - Connect WebSocket for real-time updates
   - Submit responses and receive feedback
   - Complete session and view summary

3. **Analytics Access**:
   - Request analytics with proper permissions
   - Subscribe to WebSocket for live updates
   - Export data in various formats

### Federation Protocol
- REST API for node registration
- Heartbeat mechanism for health monitoring
- Signed data sharing with verification
- Protocol synchronization across network
- Compliance verification system

## Deployment Considerations

### Database
- SQLite for development/small deployments
- PostgreSQL for production (via feature flag)
- Automatic migration on startup
- Connection pooling configured

### TLS/SSL
- Automatic Let's Encrypt certificate provisioning
- Certificate renewal scheduler
- Support for custom certificates
- HTTP to HTTPS redirect

### Scaling
- Stateless design (except WebSocket connections)
- Cache layer reduces database load
- Async processing for heavy operations
- Background job queue for batch processing

### Monitoring
- Health endpoints for container orchestration
- Prometheus metrics for monitoring systems
- Structured logging for log aggregation
- Performance tracking for optimization

## Error Handling

### Error Types
- `AppError::BadRequest(String)` - 400
- `AppError::Unauthorized` - 401
- `AppError::Forbidden` - 403
- `AppError::NotFound(String)` - 404
- `AppError::Conflict(String)` - 409
- `AppError::RateLimitExceeded` - 429
- `AppError::InternalServerError` - 500
- `AppError::DatabaseError(sqlx::Error)` - 500
- `AppError::ValidationError(String)` - 422

### Error Response Format
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {...} // Optional additional context
  }
}
```

## Testing Support

The module provides `lib.rs` with helper functions for testing:
- `build_router()`: Create router for testing
- `create_test_state_sqlite()`: Create test AppState
- Test-friendly endpoint paths (`/api` and `/api/v1`)