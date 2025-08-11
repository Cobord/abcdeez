# error.rs - Centralized Error Management and HTTP Response Mapping

## Requirements and Dataflow
- Provides unified error handling across all application layers and external dependencies
- Maps internal errors to appropriate HTTP status codes and user-friendly messages
- Implements structured error logging with unique error IDs for debugging
- Handles errors from database, cache, authentication, and core library systems
- Supports specialized error types for numerical, statistical, and convergence failures

## High-level Purpose and Responsibilities
- **Error Consolidation**: Single error type encompassing all failure modes
- **HTTP Response Mapping**: Automatic conversion from internal errors to HTTP responses
- **Structured Logging**: Detailed error logging with correlation IDs and context
- **User Experience**: User-friendly error messages while preserving debug information
- **Security**: Prevents information leakage through careful error message filtering
- **Developer Experience**: Rich error context for debugging and troubleshooting

## Key Abstractions and Interfaces
- `AppError` enum: Comprehensive error type covering all application failure modes
- `ErrorContext`: Internal error logging and ID generation for traceability
- `AppResult<T>`: Type alias providing consistent error handling patterns
- `IntoResponse` implementation: Automatic HTTP response generation from errors
- From trait implementations: Seamless conversion from external library errors

## Data Transformations and Flow
1. **Error Capture**: Conversion from external library errors to unified AppError type
2. **Context Generation**: Assignment of unique error IDs and structured logging
3. **Message Filtering**: Separation of internal debug information from user messages
4. **HTTP Mapping**: Translation of error types to appropriate HTTP status codes
5. **Response Formatting**: JSON response construction with consistent structure
6. **Logging Integration**: Structured logging with error IDs for correlation

## Dependencies and Interactions
- **All Application Layers**: Used throughout handlers, services, and data access layers
- **External Libraries**: Converts errors from SQLx, JWT, UUID, JSON, and core library
- **HTTP Layer**: Integrates with Axum framework for response generation
- **Monitoring Systems**: Provides error metrics and alerting integration points
- **Logging Infrastructure**: Coordinates with tracing system for structured logging
- **Authentication System**: Handles JWT and OAuth-related error conditions

## Architectural Patterns
- **Result Type Propagation**: Consistent error handling through Result<T, AppError> pattern
- **Error Transformation**: Systematic conversion from external to internal error types
- **Logging with Correlation**: Unique error IDs for distributed tracing and debugging
- **Security-Conscious Messaging**: Careful separation of internal and external error details
- **HTTP Response Generation**: Automatic mapping from domain errors to HTTP responses
- **Structured Error Context**: Rich error metadata for debugging and monitoring