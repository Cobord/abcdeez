# lib.rs - Application Router and Core Exports

## Requirements and Dataflow
- Provides centralized module exports and public API surface
- Constructs the complete HTTP router with all endpoints and middleware layers
- Defines health check endpoints for monitoring and load balancer integration
- Serves static content including admin panel interface
- Implements comprehensive CORS and security policy configuration

## High-level Purpose and Responsibilities
- **Module Organization**: Central module registry and public API definition
- **Router Assembly**: Complete HTTP route construction with nested path organization
- **Middleware Orchestration**: Applies security, authentication, audit, and performance middleware layers
- **Health Endpoints**: Provides deep health checks for database and cache connectivity
- **Admin Interface**: Serves administrative dashboard and management tools
- **API Versioning**: Supports both `/api` and `/api/v1` prefixes for backward compatibility

## Key Abstractions and Interfaces
- `build_router()`: Main router construction function accepting shared application state
- `ready_check()`: Database and cache connectivity verification
- `serve_admin_panel()`: Static HTML content serving for administration
- Route grouping by functional domain (auth, learners, sessions, analytics, etc.)
- Middleware layering with proper ordering for security and functionality

## Data Transformations and Flow
1. **Module Exports**: Consolidates all subsystem modules into cohesive API
2. **Route Definition**: Maps HTTP endpoints to handler functions with proper parameter extraction
3. **Middleware Application**: Applies cross-cutting concerns in correct order
4. **State Injection**: Provides shared application state to all handlers
5. **Health Verification**: Tests database and cache connectivity on demand
6. **CORS Configuration**: Environment-specific cross-origin policy application
7. **Response Formatting**: Consistent JSON response structure across endpoints

## Dependencies and Interactions
- **Handler Modules**: All domain-specific request handlers (auth, learners, sessions, etc.)
- **Middleware Stack**: Security headers, authentication, rate limiting, audit logging
- **Application State**: Database pool, cache connection, configuration access
- **Monitoring Systems**: Health checks, metrics collection, performance tracking
- **Static Assets**: Admin panel HTML and related static resources
- **WebSocket Support**: Real-time session and analytics connections

## Architectural Patterns
- **Layered Middleware**: Onion-style middleware application with proper ordering
- **Route Nesting**: Hierarchical path organization by functional domain
- **Dependency Injection**: Shared state container pattern throughout request lifecycle
- **Health Check Stratification**: Multiple levels of health verification (live/ready/detailed)
- **CORS Policy Management**: Environment-specific cross-origin configuration
- **API Versioning**: Multiple endpoint versions for backward compatibility support