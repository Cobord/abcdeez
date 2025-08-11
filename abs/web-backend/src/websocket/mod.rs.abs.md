# WebSocket Module Core Architecture

## Requirements and Dataflow

### Core Requirements
- Secure real-time communication for educational learning sessions
- JWT-based authentication with role-based access control
- Adaptive learning intervention system with real-time response
- Live analytics broadcasting for administrative dashboards
- Session state management with performance tracking
- Heartbeat mechanism for connection health monitoring

### Data Flow Patterns
1. **Connection Establishment**: WebSocket Upgrade → Authentication Challenge → Token Validation → Session Authorization
2. **Learning Session**: Task Request → Task Generation → Response Collection → Feedback Delivery → Intervention Assessment
3. **Adaptive Response**: Performance Analysis → Struggle Detection → Intervention Selection → Real-time Delivery
4. **Analytics Broadcasting**: Metrics Collection → Data Aggregation → Live Broadcasting → Dashboard Updates
5. **Session Management**: State Tracking → Performance Monitoring → Resource Cleanup → Graceful Termination

## High-level Purpose and Responsibilities

### Primary Purpose
Provides real-time, secure communication infrastructure that enables interactive learning experiences, adaptive educational interventions, and live analytics dashboards through WebSocket connections with comprehensive session management.

### Core Responsibilities
- **Real-time Communication**: Bi-directional messaging between client and server for immediate interaction
- **Secure Authentication**: JWT token validation with session ownership verification
- **Adaptive Learning**: Real-time intervention system based on learner performance analysis
- **Session Management**: Complete learning session lifecycle with state tracking
- **Analytics Broadcasting**: Live metrics streaming for administrative and research dashboards
- **Performance Monitoring**: Response time tracking and cognitive load assessment

## Key Abstractions and Interfaces

### Message Protocol
- **ClientMessage**: Typed client-to-server messages including authentication, task requests, and responses
- **ServerMessage**: Typed server-to-client messages including tasks, feedback, interventions, and analytics
- **TaskMessage**: Task delivery with difficulty and timing metadata
- **FeedbackMessage**: Immediate response validation with explanatory content

### Session Management
- **SessionInfo**: Session metadata linking learners to active learning topology
- **SessionState**: Runtime state tracking including current task and performance history
- **LiveMetrics**: Real-time analytics data for dashboard broadcasting
- **StatsMessage**: Performance statistics updates for learner progress tracking

### Authentication Framework
- **JWT Validation**: Token-based authentication with expiration and signature verification
- **Claims Processing**: User identity and permission extraction from validated tokens
- **Session Ownership**: Verification that sessions belong to authenticated users
- **Role-based Access**: Permission checking for analytics and administrative functions

## Data Transformations and Flow

### Authentication Flow
```
WebSocket Connection → Auth Message → JWT Validation → Claims Extraction → Session Authorization → Active Session
```

### Learning Interaction Loop
```
Task Request → Task Selection → Task Delivery → Response Collection → Performance Analysis → Feedback/Intervention
```

### Adaptive Intervention Pipeline
```
Response Analysis → Struggle Assessment → Intervention Decision → Message Generation → Real-time Delivery
```

### Analytics Broadcasting
```
Metrics Collection → Data Aggregation → Live Metrics Generation → WebSocket Broadcast → Dashboard Update
```

## Dependencies and Interactions

### External Dependencies
- **axum**: WebSocket upgrade handling and HTTP framework integration
- **futures_util**: Async stream processing for WebSocket message handling
- **jsonwebtoken**: JWT token validation and claims processing
- **serde**: Message serialization and deserialization for type-safe communication
- **tokio**: Async runtime with interval timers for heartbeat and analytics broadcasting

### Internal System Interactions
- **Services Layer**: Integration with AdaptationService, LearnerService, and AnalyticsService for business logic
- **State Management**: Access to application state for database connections and configuration
- **Authentication**: JWT validation using shared secret and claims processing
- **Database**: Session information retrieval and learner data access
- **Monitoring**: Integration with metrics collection for WebSocket connection tracking

## Architectural Patterns

### Type-Safe Message Protocol
- Strongly typed message enums prevent runtime serialization errors
- Tagged union serialization ensures proper message routing and handling
- Comprehensive message validation with structured error responses
- Protocol versioning support for backward compatibility

### Secure Connection Management
- Multi-stage authentication with token validation and session verification
- Connection timeout handling with automatic cleanup and resource management
- Heartbeat mechanism for connection health monitoring and failure detection
- Graceful shutdown procedures with proper resource deallocation

### Adaptive Learning Integration
- Real-time performance analysis with immediate intervention capability
- Struggle level assessment based on error patterns and response times
- Context-aware intervention selection using learner model and session history
- Seamless integration with core adaptive learning algorithms

### Event-Driven Architecture
- Async/await patterns throughout for non-blocking concurrent operations
- Event loop design with message handling and periodic task execution
- State machine implementation for session lifecycle management
- Resource-conscious design with bounded collections and cleanup procedures

### Educational Domain Specialization
- Learning session-specific message types and interaction patterns
- Cognitive load assessment through response time analysis and error tracking
- Intervention system tailored to educational psychology principles
- Analytics dashboard focused on learning outcomes and engagement metrics

### Performance and Scalability
- Efficient JSON serialization with minimal memory allocation overhead
- Connection pooling and resource management for concurrent session support
- Background processing for computationally intensive tasks
- Configurable timing parameters for different deployment scenarios