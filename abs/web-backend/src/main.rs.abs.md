# main.rs - Application Entry Point

## Requirements and Dataflow
- Initializes and coordinates all application services and infrastructure components
- Sets up structured tracing with detailed logging configuration for debugging
- Establishes database connection pool and runs migrations on startup
- Creates shared application state for dependency injection across handlers
- Configures TLS/HTTPS support with automatic certificate management
- Starts background monitoring and batch processing tasks

## High-level Purpose and Responsibilities
- **Application Bootstrap**: Orchestrates the startup sequence and dependency initialization
- **Service Coordination**: Launches concurrent background services (health monitoring, performance tracking, batch jobs, OAuth validation)
- **TLS Management**: Handles both HTTP and HTTPS server setup with Let's Encrypt integration
- **Production Safety**: Validates configuration for production deployments
- **Error Handling**: Provides graceful failure handling during startup

## Key Abstractions and Interfaces
- `AppState`: Centralized dependency container shared across the application
- `Config`: Environment-based configuration management with validation
- `TlsManager`: Certificate provisioning and renewal automation
- Background task spawning pattern using `tokio::spawn`
- Structured logging with correlation IDs and detailed context

## Data Transformations and Flow
1. **Environment → Configuration**: Loads and validates environment variables
2. **Configuration → Database Pool**: Establishes connection pooling based on config
3. **Database → Migrations**: Applies schema migrations automatically
4. **State Assembly**: Combines database, cache, and config into shared state
5. **Router Construction**: Builds HTTP routing with middleware layers
6. **Background Services**: Spawns concurrent monitoring and processing tasks
7. **Server Binding**: Starts HTTP/HTTPS listeners on configured ports

## Dependencies and Interactions
- **Database Layer**: Requires functional database connection and migration system
- **Cache Layer**: Depends on in-memory cache connection manager
- **Configuration**: Environment-based config with production safety validation
- **Core Library**: Integrates with `abcdeez_core` for domain logic
- **Monitoring**: Interfaces with health check and performance monitoring systems
- **TLS Infrastructure**: Coordinates with certificate management and ACME challenges
- **Batch Processing**: Manages background job workers and OAuth credential validation

## Architectural Patterns
- **Dependency Injection**: Centralized state container pattern
- **Concurrent Services**: Background task orchestration with shared state
- **Configuration-Driven**: Environment-based feature flags and settings
- **Production Hardening**: Explicit validation and safety checks for deployment environments