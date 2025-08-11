# cache.rs - In-Memory Redis-Compatible Cache

## Requirements and Dataflow
- Provides Redis-compatible API for caching operations without external Redis dependency
- Implements TTL-based expiration with automatic cleanup mechanisms
- Supports common Redis commands (GET, SET, SETEX, DEL, INCR, EXISTS, EXPIRE)
- Manages concurrent access through async-safe data structures
- Maintains API compatibility with existing Redis client code

## High-level Purpose and Responsibilities
- **Redis API Emulation**: Drop-in replacement for Redis client with identical command interface
- **Memory Management**: In-process cache with automatic expiration and cleanup
- **Concurrency Safety**: Thread-safe operations using Arc/Mutex patterns
- **TTL Management**: Automatic expiration handling with background cleanup tasks
- **Command Translation**: Redis command parsing and execution against in-memory storage
- **Development Simplification**: Eliminates Redis server dependency for development/testing

## Key Abstractions and Interfaces
- `ConnectionManager`: Redis connection manager compatible interface
- `InMemoryCache`: Core in-memory storage with TTL support
- `Command`: Redis command builder with argument chaining
- `FromCacheResponse`: Type conversion trait for Redis response adaptation
- `ToArg`: Type conversion trait for command arguments
- Background cleanup task with automatic expired entry removal

## Data Transformations and Flow
1. **Command Construction**: Redis-style command building with typed arguments
2. **Argument Serialization**: Convert typed arguments to string representations
3. **Command Execution**: Parse and execute Redis commands against in-memory store
4. **TTL Processing**: Handle expiration times and automatic cleanup
5. **Response Adaptation**: Convert internal results to expected Redis response types
6. **Background Cleanup**: Periodic removal of expired cache entries

## Dependencies and Interactions
- **Application State**: Integrated into shared application state for dependency injection
- **Handler Layer**: Used by all handlers requiring caching functionality
- **Monitoring Systems**: Cache hit/miss metrics and performance tracking
- **Authentication**: Token blacklisting and session management
- **Rate Limiting**: Request counting and window management
- **Session Storage**: Temporary session data and state management

## Architectural Patterns
- **Drop-in Replacement**: API-compatible Redis client replacement pattern
- **Background Processing**: Autonomous cleanup task for memory management
- **Type-Safe Commands**: Strongly-typed command arguments and responses
- **Concurrent Data Structures**: Thread-safe access patterns for shared cache
- **TTL Management**: Lazy and proactive expiration handling
- **Memory Efficiency**: Automatic cleanup prevents unbounded memory growth