# Models Directory - Data Structures and API Contracts

## How Core and Apps Should Use Data Models

### User and Authentication Models (user.rs)
- **User Registration**: Comprehensive user profile structure with OAuth provider support
- **Authentication Responses**: Standardized token response format with user details
- **OAuth Integration**: Apple Sign In and GitHub authentication data structures
- **Profile Management**: User metadata and preference handling structures

**Integration Pattern**:
```rust
// User registration request/response
POST /api/auth/register
{
  "username": "learner1",
  "email": "user@example.com", 
  "password": "SecurePass123!"
}

// Response: UserResponse with profile data
{
  "id": "uuid",
  "username": "learner1",
  "email": "user@example.com",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z",
  "metadata": { "preferences": {...} }
}

// Token response structure
{
  "access_token": "jwt_token",
  "refresh_token": "refresh_jwt",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": { /* UserResponse */ }
}
```

### Learning Session Models (session.rs, learner.rs)
- **Session Lifecycle**: Complete session state management from creation to completion
- **Topology Configuration**: Learning domain setup with configurable topology data
- **Progress Tracking**: Session status and performance summary structures
- **Learner Profiles**: Comprehensive learner information with statistics and preferences

**Data Flow Integration**:
```rust
// Session creation with topology
{
  "learner_id": "uuid",
  "topology_type": "alphabet",
  "topology_data": {
    "nodes": ["A", "B", "C", "D"],
    "edges": [["A", "B"], ["B", "C"], ["C", "D"]]
  }
}

// Session response with status
{
  "id": "session_uuid",
  "learner_id": "learner_uuid", 
  "topology_type": "alphabet",
  "topology_data": { /* topology configuration */ },
  "start_time": "2023-01-01T10:00:00Z",
  "status": "Active",
  "summary": null // populated on completion
}

// Learner profile structure
{
  "id": "uuid",
  "user_id": "user_uuid",
  "name": "Student Name",
  "metadata": {
    "learning_preferences": "visual",
    "difficulty_preference": "adaptive"
  },
  "created_at": "2023-01-01T00:00:00Z"
}
```

### Task Response Models (response.rs)
- **Response Tracking**: Task interaction capture with timing and confidence data
- **Validation Structures**: Response validation and scoring information
- **API Response Format**: Standardized response wrappers for all endpoints
- **Error Handling**: Consistent error response structures with correlation tracking

**Response Model Usage**:
```rust
// Task response submission
{
  "task_id": "task_uuid",
  "response": "B",
  "response_time_ms": 1200,
  "confidence": 0.85,
  "metadata": {
    "interaction_type": "click",
    "attempt_number": 1
  }
}

// Standardized API response wrapper
{
  "data": { /* actual response data */ },
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 100,
    "total_pages": 2
  },
  "metadata": {
    "request_id": "uuid",
    "timestamp": "2023-01-01T12:00:00Z"
  }
}
```

## State Machines and Lifecycle Patterns

### User Account Data Lifecycle
1. **Registration Data** → Validation → **User Created** → Profile Setup
2. **User Active** → Authentication → **Token Generated** → API Access
3. **OAuth Registration** → Provider Validation → **Account Linked** → Profile Merge
4. **Profile Updates** → Validation → **Data Updated** → Audit Trail

### Session Data State Machine
1. **Session Request** → Learner Validation → **Session Created** → Topology Setup
2. **Session Active** → Task Responses → **Progress Tracked** → Performance Analysis  
3. **Session Complete** → Summary Generation → **Data Archived** → Analytics Update
4. **Session Archived** → Export Request → **Data Delivered** → Compliance Check

### Research Data Lifecycle
1. **Experiment Design** → Participant Models → **Data Collection** → Response Tracking
2. **Data Validation** → Privacy Filtering → **Analysis Ready** → Statistical Processing
3. **Results Generated** → Export Models → **Data Packaged** → Delivery Preparation

## Integration Patterns and Best Practices

### Data Validation and Serialization
- **Input Validation**: Comprehensive validation at model boundaries
- **Type Safety**: Strong typing prevents runtime data errors
- **Serialization Consistency**: Uniform JSON serialization across all models
- **Error Propagation**: Validation errors properly propagated to clients

**Validation Pattern**:
```rust
// Models include validation logic
impl User {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Email format validation
        // Username length and character validation
        // Password strength validation (if present)
    }
}

// API responses include validation errors
{
  "error": "Validation failed",
  "status": 400,
  "details": {
    "email": ["Invalid email format"],
    "username": ["Username must be 3-50 characters"]
  }
}
```

### Database Integration Patterns
- **UUID Primary Keys**: All entities use UUID for distributed system compatibility
- **Metadata Fields**: Flexible JSON metadata for extensible data storage
- **Timestamp Tracking**: Created/updated timestamps for audit and synchronization
- **Soft Deletion**: Models support soft deletion for data retention compliance

### API Response Standardization
- **Consistent Structure**: All responses follow standardized format patterns
- **Error Handling**: Uniform error response structure across all endpoints
- **Pagination Support**: Built-in pagination for collection responses
- **Metadata Inclusion**: Request tracking and timing information in responses

## Architectural Decisions and Constraints

### Data Model Design Principles
- **Domain Separation**: Models organized by functional domain boundaries
- **Immutability**: Prefer immutable data structures where possible
- **Validation at Boundaries**: Input validation at model entry points
- **Serialization Control**: Explicit control over JSON serialization formats

### Performance Considerations
- **Lazy Loading**: Large related data loaded on demand
- **Pagination**: Built-in pagination for large datasets
- **Index-Friendly**: Model design supports efficient database indexing
- **Memory Efficiency**: Structures designed for minimal memory footprint

### Security and Privacy
- **Sensitive Data Handling**: Password hashes and tokens properly protected
- **PII Protection**: Personal information handled with appropriate safeguards
- **Audit Trail Support**: Models support comprehensive audit logging
- **GDPR Compliance**: Data structures support privacy regulation compliance

## Common Usage Patterns and Examples

### User Management Integration
```rust
// Complete user registration flow
CreateUserRequest {
    username: "learner123",
    email: "learner@university.edu", 
    password: "SecurePassword123!"
}

// Response includes complete user profile
UserResponse {
    id: uuid,
    username: "learner123",
    email: "learner@university.edu",
    created_at: timestamp,
    updated_at: timestamp,
    metadata: Optional<serde_json::Value>
}
```

### Learning Session Management
```rust
// Session creation with full configuration
CreateSessionRequest {
    learner_id: uuid,
    topology_type: "alphabet",
    topology_data: Some(json_topology_config)
}

// Session tracking throughout lifecycle
Session {
    id: uuid,
    learner_id: uuid,
    topology_type: "alphabet",
    topology_data: serde_json::Value,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    status: SessionStatus::Active,
    summary: Option<SessionSummary>
}
```

### Response Data Collection
```rust
// Task response with comprehensive data
TaskResponse {
    task_id: uuid,
    session_id: uuid,
    response: "user_answer",
    response_time_ms: 1250,
    correct: Some(true),
    score: Some(0.95),
    metadata: Some(additional_context)
}

// Response validation and feedback
ResponseRecord {
    id: uuid,
    task_id: uuid,
    learner_id: uuid,
    response_data: TaskResponse,
    created_at: timestamp,
    validation_status: "validated"
}
```

### Analytics Data Structures  
```rust
// Learner statistics for analytics
LearnerStats {
    total_sessions: 45,
    total_responses: 1250,
    average_accuracy: 0.847,
    average_response_time_ms: 1450,
    improvement_rate: 0.023,
    last_active: timestamp
}

// Population analytics aggregation
PopulationMetrics {
    total_learners: 1000,
    active_learners: 750,
    completion_rate: 0.85,
    average_performance: 0.78,
    trend_data: Vec<TrendPoint>
}
```

## Security and Performance Considerations

### Data Security
- **Sensitive Field Protection**: Password hashes never included in API responses
- **Token Management**: JWT tokens handled securely with proper expiration
- **Personal Data**: PII fields properly protected and access-controlled
- **Audit Requirements**: All data changes logged for compliance

### Performance Optimization
- **Efficient Serialization**: Optimized JSON serialization for large datasets
- **Database Compatibility**: Model structure optimized for database performance
- **Memory Management**: Structures designed for efficient memory usage
- **Caching Support**: Models designed to work efficiently with caching layers

### Validation and Error Handling
- **Comprehensive Validation**: All input data validated at model boundaries
- **Type Safety**: Rust's type system prevents many runtime errors
- **Error Context**: Validation errors include sufficient context for debugging
- **User-Friendly Messages**: Error messages appropriate for end users

The models provide a robust foundation for type-safe data handling across the learning management system with comprehensive validation, serialization, and integration support. Applications should use these structures consistently to ensure data integrity and API compatibility.