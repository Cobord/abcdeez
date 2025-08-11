# Middleware Directory - Security and Infrastructure Integration

## How Core and Apps Should Use Middleware Functionality

### Authentication and Authorization Integration
- **JWT Token Validation**: All protected endpoints automatically validate Bearer tokens
- **Role-Based Access Control**: Middleware enforces admin, researcher, and user permissions
- **Session Management**: Automatic session tracking with blacklist support for logout
- **Permission Checking**: Fine-grained permission validation for sensitive operations

**Integration Requirements**:
```rust
// All API calls to protected endpoints must include JWT token
Authorization: Bearer <jwt_token>

// Role-specific endpoints automatically enforce permissions
GET /api/admin/dashboard  // Requires admin role
GET /api/analytics/*      // Requires analytics_access permission
```

### Rate Limiting and DoS Protection
- **Multi-Tiered Rate Limiting**: Global, IP-based, and user-specific rate limits
- **Endpoint-Specific Limits**: Different limits for authentication, admin, and regular endpoints
- **Burst Protection**: Short-term burst limits prevent rapid-fire attacks
- **Automatic IP Blocking**: Aggressive IPs temporarily blocked automatically

**Client Implementation Guidelines**:
```rust
// Respect rate limit headers
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 45
X-RateLimit-Window: 60

// Implement exponential backoff for HTTP 429 responses
if (response.status === 429) {
    await delay(Math.pow(2, retryCount) * 1000);
}
```

### Security Headers and Content Validation
- **Automatic Security Headers**: CSP, HSTS, and other security headers applied automatically
- **Content Validation**: JSON content type and size limits enforced
- **XSS Protection**: Content Security Policy prevents script injection
- **HTTPS Enforcement**: Production environments require TLS connections

## State Machines and Lifecycle Patterns

### Authentication State Flow
1. **Request Received** → JWT Extraction → **Token Validation**
2. **Token Valid** → User Lookup → **Claims Attached** → **Authorized Request**
3. **Token Invalid** → **Unauthorized Response** (HTTP 401)
4. **Token Blacklisted** → **Session Revoked** → **Unauthorized Response**

### Rate Limiting State Machine
1. **Request Received** → **Global Rate Check** → **IP Rate Check** → **User Rate Check**
2. **Limits OK** → Counter Increment → **Request Allowed**
3. **Limit Exceeded** → Block Decision → **Request Denied** (HTTP 429)
4. **Aggressive Pattern** → IP Block → **Temporary Ban** → **Auto-Unblock**

### Security Validation Pipeline
1. **Request Entry** → Content Validation → IP Check → Rate Limiting
2. **Security Passed** → Authentication → Authorization → **Handler Execution**
3. **Security Failed** → Audit Logging → Error Response → **Request Blocked**

## Integration Patterns and Best Practices

### Correlation ID and Distributed Tracing
- **Automatic ID Generation**: Every request gets unique correlation ID
- **Header Propagation**: IDs passed to downstream services for tracing
- **W3C Trace Context**: Support for standard distributed tracing protocols
- **Error Correlation**: All errors include correlation IDs for debugging

**Usage Pattern**:
```rust
// Client can provide correlation ID
x-correlation-id: custom-trace-id

// Server always returns correlation ID in response
x-correlation-id: uuid-generated-id
x-trace-id: trace-uuid
```

### Audit Logging Integration
- **Comprehensive Logging**: All API calls logged for compliance
- **Security Event Detection**: Failed authentications and suspicious patterns logged
- **Context Preservation**: User context, IP addresses, and request details captured
- **Background Processing**: Audit logging doesn't block request processing

### Error Handling and Response Formatting
- **Consistent Error Format**: Standardized error responses across all endpoints
- **Security-Safe Messages**: Error details don't leak sensitive information
- **Correlation Tracking**: Errors include correlation IDs for debugging
- **Proper HTTP Status Codes**: Appropriate status codes for different error conditions

### Performance Monitoring Integration
- **Request Timing**: Automatic timing for all requests with performance metrics
- **Error Rate Tracking**: Monitor error rates and patterns
- **Resource Usage**: Track database and cache usage patterns
- **Alerting Integration**: Metrics available for monitoring system integration

## Architectural Decisions and Constraints

### Security Architecture
- **Defense in Depth**: Multiple security layers with fail-safe defaults
- **Zero Trust Model**: All requests validated regardless of source
- **Principle of Least Privilege**: Minimum required permissions enforced
- **Security by Default**: Secure configurations without manual intervention

### Performance Design
- **Non-Blocking Operations**: Middleware designed for high concurrency
- **Efficient Caching**: Rate limiting and session data cached efficiently
- **Resource Pooling**: Database and cache connections pooled properly
- **Background Processing**: Heavy operations moved to background tasks

### Scalability Considerations
- **Stateless Design**: Middleware scales horizontally without session affinity
- **Distributed Cache**: Rate limiting works across multiple instances
- **Load Balancer Compatible**: Works with standard load balancing configurations
- **Health Check Integration**: Proper health checks for load balancer configuration

## Security and Performance Considerations

### Security Hardening
- **JWT Security**: Strong signature validation with HS256 algorithm
- **Token Blacklisting**: Revoked tokens properly blocked across instances
- **IP Reputation**: Automatic blocking of abusive IP addresses
- **Content Security**: Strict content validation prevents injection attacks

**Security Configuration**:
```rust
// Production security headers automatically applied
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
Content-Security-Policy: default-src 'none'; script-src 'self'; ...
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
```

### Performance Optimization
- **Connection Pooling**: Efficient database and cache connection management
- **Response Caching**: Appropriate caching for frequently accessed data
- **Compression**: Automatic response compression for bandwidth optimization
- **Request Batching**: Efficient batch processing for audit logging

### Monitoring Integration
- **Metrics Export**: Prometheus-compatible metrics for monitoring systems
- **Health Endpoints**: Comprehensive health checks for system monitoring
- **Alert Generation**: Critical issues automatically generate alerts
- **Dashboard Integration**: Real-time metrics for operational dashboards

## Common Usage Patterns and Examples

### Authentication Flow
```rust
// Automatic JWT validation on protected endpoints
GET /api/learners
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

// Server automatically attaches user claims
// Handler receives Claims extension with user info:
// { sub: uuid, username: "user", role: "admin", permissions: [...] }
```

### Rate Limiting Handling
```rust
// Client should respect rate limit headers
HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Window: 60

// Handle rate limit exceeded
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Window: 60
Retry-After: 45
```

### Error Handling with Correlation
```rust
// Error response includes correlation ID
HTTP/1.1 401 Unauthorized
x-correlation-id: 550e8400-e29b-41d4-a716-446655440000
Content-Type: application/json

{
  "error": "Unauthorized access",
  "status": 401,
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Security Headers in Production
```rust
// Automatic security headers (production environment)
HTTP/1.1 200 OK
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
```

### Audit Trail Integration
```rust
// All API calls automatically logged with context
{
  "timestamp": "2023-01-01T12:00:00Z",
  "correlation_id": "uuid",
  "user_id": "user_uuid",
  "method": "POST",
  "path": "/api/sessions",
  "status": 201,
  "duration_ms": 45,
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0...",
  "action": "create",
  "resource_type": "session",
  "resource_id": "session_uuid"
}
```

The middleware provides comprehensive security, monitoring, and infrastructure capabilities that operate transparently to protect and enhance the API. Applications should implement proper client-side handling for authentication, rate limiting, and error scenarios to work optimally with this middleware stack.