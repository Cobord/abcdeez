# db.rs - Database Abstraction and Migration Management

## Requirements and Dataflow
- Provides database-agnostic abstraction layer supporting SQLite and PostgreSQL
- Manages connection pooling with configurable parameters and timeouts
- Implements comprehensive migration system with rollback capabilities
- Handles data type conversions between databases (UUID, JSON storage)
- Provides migration validation, backup creation, and history tracking

## High-level Purpose and Responsibilities
- **Database Abstraction**: Unified interface across SQLite and PostgreSQL backends
- **Connection Management**: Pool configuration with environment-based parameters
- **Migration Orchestration**: Forward and backward migration execution with safety checks
- **Data Type Handling**: Database-specific UUID and JSON storage abstractions
- **Migration Safety**: Rollback capabilities, validation, and backup creation
- **Development Support**: SQLite for development, PostgreSQL for production flexibility

## Key Abstractions and Interfaces
- `Database` struct: High-level database operations wrapper
- `MigrationManager`: Advanced migration handling with rollback support
- `MigrationInfo`: Migration metadata and status tracking
- Type aliases (`DbPool`, `DbRow`, `DbTransaction`) for database portability
- Data conversion functions (`uuid_to_db`, `json_to_db`) for cross-database compatibility

## Data Transformations and Flow
1. **Connection Pool Creation**: Environment-based pool configuration and initialization
2. **Database-Specific Adaptations**: UUID and JSON handling based on backend type
3. **Migration Execution**: Forward migration application with dependency tracking
4. **Rollback Processing**: Reverse migration execution with transaction safety
5. **Data Type Conversion**: Automatic conversion between application and database types
6. **Backup Operations**: Database backup creation before major migrations

## Dependencies and Interactions
- **Configuration System**: Database URL and pooling parameters from environment
- **Migration Files**: File system integration for migration script loading
- **Application State**: Database pool injection into shared application state
- **All Handlers**: Database access through connection pool abstraction
- **Monitoring Systems**: Connection health checks and pool metrics
- **Admin Interface**: Migration status reporting and management operations

## Architectural Patterns
- **Database Abstraction Layer**: Feature flags and type aliases for multi-backend support
- **Migration as Code**: Version-controlled database schema evolution
- **Rollback Safety**: Transactional rollback with integrity checking
- **Connection Pooling**: Resource management with configurable lifecycle policies
- **Type Safety**: Compile-time database feature selection and type checking
- **Backup Strategy**: Automated backup creation for migration safety