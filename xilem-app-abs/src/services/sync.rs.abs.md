# services/sync.rs - Offline Data Synchronization Service

## Conceptual Overview
Manages offline data synchronization with conflict resolution, batch processing, and background sync operations. Ensures data consistency between local storage and backend systems.

## Key Data Flows
- Queues responses for offline synchronization
- Performs batch uploads with error handling
- Handles remote updates and conflict resolution
- Manages persistent storage integration
- Supports background synchronization scheduling

## Main Responsibilities
- Offline response queue management
- Batch synchronization with configurable sizes
- Conflict resolution for concurrent updates
- Background sync task coordination
- Storage integration for crash recovery
- Sync status and metrics tracking

## Dependencies on Other Components
- `models::*` - Response and sync data models
- `services::api::ApiClient` - Backend communication
- `services::storage::StorageService` - Local persistence
- Tokio for async operations and timers

## User-Facing Functionality
- Seamless offline learning capability
- Automatic background data synchronization
- Data consistency across devices
- Recovery from network interruptions
- Transparent sync status indicators