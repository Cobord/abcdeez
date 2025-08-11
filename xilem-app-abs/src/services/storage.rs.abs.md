# services/storage.rs - Local SQLite Storage Service

## Conceptual Overview
Local data persistence service using SQLite for offline storage of user data, sessions, responses, learner models, and cache. Provides comprehensive data management with support for synchronization and cleanup operations.

## Key Data Flows
- Persists user profiles and session data locally
- Manages pending response queue for offline sync
- Caches learner models and task data
- Stores gamification data and achievements
- Supports generic key-value storage for extensibility

## Main Responsibilities
- SQLite database schema management
- User and session data persistence
- Offline response queue management
- Learner model caching and retrieval
- Task caching with TTL support
- Gamification data storage
- Generic key-value storage operations
- Database cleanup and maintenance

## Dependencies on Other Components
- `models::*` - Data models for storage
- `abcdeez_core::learning::learner::LearnerModel` - Learning model integration
- Rusqlite for SQLite database operations
- Serde for JSON serialization in storage

## User-Facing Functionality
- Offline application functionality
- Fast data access and caching
- Data persistence across app sessions
- Efficient storage management
- Background data cleanup