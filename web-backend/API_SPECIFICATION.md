# Graph Learning Backend API Specification

## Version: 1.0.0

## Base URL
```
https://abcdeez.fg-goose.online/api/v1
```

## Authentication

All authenticated endpoints require a JWT token in the Authorization header:
```
Authorization: Bearer <token>
```

## Content Types

- Request: `application/json`
- Response: `application/json`

## Error Response Format

All errors follow this structure:
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {},
    "timestamp": "2024-01-01T00:00:00Z",
    "request_id": "uuid"
  }
}
```

## Endpoints

### Authentication

#### POST /auth/register
Create a new user account.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "secure_password",
  "display_name": "John Doe"
}
```

**Response (201):**
```json
{
  "user_id": "uuid",
  "email": "user@example.com",
  "display_name": "John Doe",
  "created_at": "2024-01-01T00:00:00Z"
}
```

#### POST /auth/login
Authenticate and receive access token.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "secure_password"
}
```

**Response (200):**
```json
{
  "access_token": "jwt_token",
  "refresh_token": "refresh_token",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "user_id": "uuid",
    "email": "user@example.com",
    "display_name": "John Doe"
  }
}
```

#### POST /auth/refresh
Refresh access token using refresh token.

**Request:**
```json
{
  "refresh_token": "refresh_token"
}
```

**Response (200):**
```json
{
  "access_token": "new_jwt_token",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Sessions

#### POST /sessions
Create a new learning session.

**Request:**
```json
{
  "domain": "alphabet",
  "config": {
    "difficulty": 0.5,
    "hint_probability": 0.1,
    "max_trials": 100
  }
}
```

**Response (201):**
```json
{
  "session_id": "uuid",
  "domain": "alphabet",
  "config": {},
  "started_at": "2024-01-01T00:00:00Z",
  "status": "active"
}
```

#### GET /sessions
List all sessions for authenticated user.

**Query Parameters:**
- `limit` (int, default: 20, max: 100)
- `offset` (int, default: 0)
- `status` (string: active|completed|abandoned)
- `domain` (string)
- `from_date` (ISO 8601)
- `to_date` (ISO 8601)

**Response (200):**
```json
{
  "sessions": [
    {
      "session_id": "uuid",
      "domain": "alphabet",
      "started_at": "2024-01-01T00:00:00Z",
      "ended_at": "2024-01-01T01:00:00Z",
      "status": "completed",
      "total_trials": 50,
      "correct_trials": 45,
      "average_rt": 1.234
    }
  ],
  "total": 100,
  "limit": 20,
  "offset": 0
}
```

#### GET /sessions/{session_id}
Get detailed session information.

**Response (200):**
```json
{
  "session_id": "uuid",
  "domain": "alphabet",
  "config": {},
  "started_at": "2024-01-01T00:00:00Z",
  "ended_at": "2024-01-01T01:00:00Z",
  "status": "completed",
  "metrics": {
    "total_trials": 50,
    "correct_trials": 45,
    "accuracy": 0.9,
    "average_rt": 1.234,
    "rt_variance": 0.456,
    "learning_curve": [0.6, 0.7, 0.8, 0.85, 0.9]
  },
  "trials": []
}
```

#### PUT /sessions/{session_id}/end
End an active session.

**Response (200):**
```json
{
  "session_id": "uuid",
  "status": "completed",
  "ended_at": "2024-01-01T01:00:00Z"
}
```

### Tasks

#### POST /sessions/{session_id}/tasks
Request next task for session.

**Request (optional):**
```json
{
  "previous_response": {
    "task_id": "uuid",
    "response": "B",
    "response_time": 1.234,
    "correct": true
  }
}
```

**Response (200):**
```json
{
  "task_id": "uuid",
  "session_id": "uuid",
  "stimulus": "A",
  "choices": ["A", "B", "C", "D"],
  "task_type": "forward",
  "difficulty": 0.5,
  "hint": null
}
```

#### POST /sessions/{session_id}/tasks/{task_id}/response
Submit response to a task.

**Request:**
```json
{
  "response": "B",
  "response_time": 1.234,
  "client_timestamp": "2024-01-01T00:00:00Z"
}
```

**Response (200):**
```json
{
  "task_id": "uuid",
  "correct": true,
  "expected": "B",
  "feedback": "Correct!",
  "next_task_available": true
}
```

### Analytics

#### GET /users/{user_id}/analytics
Get user analytics summary.

**Query Parameters:**
- `from_date` (ISO 8601)
- `to_date` (ISO 8601)
- `domain` (string)
- `granularity` (day|week|month)

**Response (200):**
```json
{
  "user_id": "uuid",
  "period": {
    "from": "2024-01-01T00:00:00Z",
    "to": "2024-01-31T23:59:59Z"
  },
  "summary": {
    "total_sessions": 25,
    "total_time_minutes": 450,
    "average_accuracy": 0.85,
    "improvement_rate": 0.12,
    "domains_practiced": ["alphabet", "numbers"]
  },
  "time_series": [
    {
      "date": "2024-01-01",
      "sessions": 2,
      "trials": 100,
      "accuracy": 0.82,
      "average_rt": 1.45
    }
  ],
  "domain_breakdown": {
    "alphabet": {
      "sessions": 15,
      "accuracy": 0.87,
      "improvement": 0.15
    }
  }
}
```

#### GET /analytics/leaderboard
Get global leaderboard.

**Query Parameters:**
- `domain` (string, required)
- `metric` (accuracy|speed|improvement)
- `period` (day|week|month|all)
- `limit` (int, default: 10)

**Response (200):**
```json
{
  "leaderboard": [
    {
      "rank": 1,
      "user_id": "uuid",
      "display_name": "TopLearner",
      "score": 0.98,
      "sessions": 50
    }
  ],
  "user_rank": {
    "rank": 42,
    "score": 0.85,
    "percentile": 75
  }
}
```

### Data Export

#### GET /users/{user_id}/export
Export user data in various formats.

**Query Parameters:**
- `format` (json|csv|parquet)
- `include` (sessions|trials|analytics)
- `from_date` (ISO 8601)
- `to_date` (ISO 8601)

**Response (200):**
Returns file download with appropriate content-type.

### Batch Operations

#### POST /batch/sessions/{session_id}/trials
Submit multiple trial responses at once (for offline sync).

**Request:**
```json
{
  "trials": [
    {
      "task_id": "uuid",
      "stimulus": "A",
      "response": "B",
      "response_time": 1.234,
      "correct": true,
      "client_timestamp": "2024-01-01T00:00:00Z"
    }
  ],
  "client_info": {
    "app_version": "1.0.0",
    "platform": "web",
    "offline_duration": 3600
  }
}
```

**Response (200):**
```json
{
  "accepted": 10,
  "rejected": 0,
  "duplicates": 2,
  "session_metrics": {
    "total_trials": 52,
    "accuracy": 0.88
  }
}
```

## Rate Limiting

- Default: 100 requests per minute per IP
- Authenticated: 1000 requests per minute per user
- Batch endpoints: 10 requests per minute

Headers returned:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1609459200
```

## Webhooks

### POST /webhooks
Register a webhook endpoint.

**Request:**
```json
{
  "url": "https://example.com/webhook",
  "events": ["session.completed", "milestone.achieved"],
  "secret": "webhook_secret"
}
```

### Webhook Events

#### session.completed
```json
{
  "event": "session.completed",
  "timestamp": "2024-01-01T00:00:00Z",
  "data": {
    "session_id": "uuid",
    "user_id": "uuid",
    "metrics": {}
  }
}
```

#### milestone.achieved
```json
{
  "event": "milestone.achieved",
  "timestamp": "2024-01-01T00:00:00Z",
  "data": {
    "user_id": "uuid",
    "milestone": "accuracy_90",
    "domain": "alphabet"
  }
}
```

## WebSocket API

### Connection
```
wss://api.graphlearning.app/v1/ws
```

### Messages

#### Client -> Server
```json
{
  "type": "subscribe",
  "channel": "session.uuid"
}
```

#### Server -> Client
```json
{
  "type": "task.next",
  "data": {
    "task_id": "uuid",
    "stimulus": "A"
  }
}
```

## Status Codes

- `200 OK` - Request succeeded
- `201 Created` - Resource created
- `204 No Content` - Request succeeded, no content
- `400 Bad Request` - Invalid request
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Access denied
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource conflict
- `422 Unprocessable Entity` - Validation error
- `429 Too Many Requests` - Rate limited
- `500 Internal Server Error` - Server error
- `503 Service Unavailable` - Service temporarily unavailable

## Versioning

API versions are specified in the URL path. Breaking changes require a new version.

Current version: v1
Deprecated versions: none
Sunset policy: 6 months notice before deprecation