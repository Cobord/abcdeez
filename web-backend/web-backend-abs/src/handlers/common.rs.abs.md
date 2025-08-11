# handlers/common.rs - Shared Handler Utilities and Helpers

## Requirements and Dataflow
- Provides reusable database operation helpers with consistent error handling
- Implements common validation patterns for input sanitization and verification
- Offers standardized response formatting utilities for HTTP responses
- Ensures consistent error handling and logging across all handlers
- Supports transaction management and resource existence checking

## High-level Purpose and Responsibilities
- **Database Utilities**: Transaction management, connection handling, and resource existence checks
- **Validation Framework**: Common validation patterns for UUIDs, strings, and required fields
- **Response Standardization**: Consistent HTTP response formatting with proper status codes
- **Error Handling**: Unified error handling patterns with structured logging
- **Code Reusability**: Shared utilities to reduce duplication across handlers
- **Consistency Enforcement**: Standardized patterns for common operations

## Key Abstractions and Interfaces
- `DbHelper`: Database operation utilities with transaction management
- `ValidationHelper`: Input validation patterns with comprehensive error messages
- `ResponseHelper`: HTTP response formatting with standardized status codes
- Transaction-wrapped operations with automatic rollback on errors
- Resource existence checking with parameterized table/column queries

## Data Transformations and Flow
1. **Database Operations**: Connection acquisition → transaction wrapping → error handling → logging
2. **Validation Pipeline**: Input checking → format validation → error generation → structured feedback
3. **Response Generation**: Data serialization → status code assignment → HTTP response creation
4. **Error Processing**: Error detection → logging → user-friendly message generation → response formatting
5. **Transaction Management**: Begin transaction → operation execution → commit/rollback → resource cleanup

## Dependencies and Interactions
- **Database Layer**: Connection pool management and transaction handling
- **Error System**: Integration with centralized error handling and logging
- **All Handlers**: Used by all handler modules for consistent operations
- **Validation Framework**: Input sanitization and verification across endpoints
- **HTTP Layer**: Response formatting and status code management
- **Logging Infrastructure**: Structured error logging and operation tracking

## Architectural Patterns
- **Utility Pattern**: Shared helper functions for common operations
- **Transaction Wrapper**: Database operations wrapped in consistent transaction handling
- **Validation Pipeline**: Systematic input validation with standardized error messages
- **Response Standardization**: Consistent HTTP response formats across all endpoints
- **Error Centralization**: Unified error handling patterns with structured logging
- **Code Reusability**: DRY principle application through shared utilities