# Sync Models Architecture

## Requirements and Dataflow

### Core Requirements
- Cross-platform data synchronization across mobile and desktop devices
- Cloud provider agnostic sync with automatic platform detection
- Conflict resolution for concurrent data modifications
- Device registration and management with push notification support
- Incremental sync with versioning and checksum validation
- Multi-user device sharing with user-scoped data isolation

### Data Flow Patterns
1. **Device Registration**: RegisterDeviceRequest → Device Validation → RegisterDeviceResponse → Sync Token
2. **Push Sync**: SyncPushRequest → Conflict Detection → Queue Processing → SyncPushResponse
3. **Pull Sync**: SyncPullRequest → Change Detection → Incremental Update → SyncPullResponse
4. **Conflict Resolution**: SyncConflict → Manual/Automatic Resolution → Data Merge → Resolution Response
5. **Status Monitoring**: Device Status → Sync Metrics → Connected Devices → SyncStatusResponse

## High-level Purpose and Responsibilities

### Primary Purpose
Provides a comprehensive multi-device synchronization framework that enables seamless data sharing across platforms while handling conflicts, versioning, and cloud storage integration for educational and research applications.

### Core Responsibilities
- **Device Management**: Registration, authentication, and lifecycle management of user devices
- **Sync Orchestration**: Bidirectional data synchronization with incremental updates
- **Conflict Resolution**: Intelligent conflict detection and resolution strategies
- **Version Control**: Change tracking with checksums and version management
- **Cloud Integration**: Platform-aware cloud provider selection (iCloud, Google Drive, OneDrive, Dropbox)
- **Data Integrity**: Checksum validation and corruption detection

## Key Abstractions and Interfaces

### Core Entities
- **SyncMetadata**: Per-device sync state with version tracking and timestamps
- **SyncQueueItem**: Pending synchronization operations with conflict detection
- **SyncConflict**: Detected conflicts with resolution strategies and merged data
- **RegisteredDevice**: Device registration with platform and push notification info

### Sync Operations
- **SyncChange**: Individual data modifications with operation type and versioning
- **SyncPacket**: Batch sync container with checksum validation
- **SyncOperation**: Create, Update, Delete operation types
- **ConflictResolution**: LocalWins, RemoteWins, Merge, Manual resolution strategies

### Cloud Integration
- **CloudProvider**: Platform-aware cloud service selection
- **DeviceInfo**: Connected device metadata with sync status
- **DeletedEntity**: Tombstone records for deletion propagation

## Data Transformations and Flow

### Device Registration Workflow
```
RegisterDeviceRequest → Platform Detection → Cloud Provider Assignment → Sync Token Generation
```

### Push Synchronization
```
SyncPushRequest → Version Check → Conflict Detection → Queue Processing → Response Generation
```

### Pull Synchronization
```
SyncPullRequest → Change Detection → Entity Filtering → Incremental Response → Version Update
```

### Conflict Resolution Process
```
Concurrent Changes → Conflict Detection → Resolution Strategy → Data Merge → Conflict Closure
```

## Dependencies and Interactions

### External Dependencies
- **chrono**: Timestamp management for sync operations and conflict resolution
- **serde**: JSON serialization for complex sync data structures
- **sqlx**: Database persistence with FromRow mapping for sync metadata
- **uuid**: Unique identifiers for devices, conflicts, and sync operations
- **sha2**: Checksum calculation for data integrity validation

### Internal System Interactions
- **Handlers**: Sync API endpoints consume these models for device and sync operations
- **Services**: Sync service layer implements conflict resolution and cloud integration
- **Push Notifications**: Device registration integrates with push notification systems
- **Storage**: Cloud provider integration for data persistence and retrieval
- **Authentication**: User-scoped device access control and permissions

## Architectural Patterns

### Multi-Device State Management
- Per-device sync metadata with version tracking
- Global sync queue with device-specific filtering
- Last-seen timestamps for device activity monitoring
- Push token management for real-time sync notifications

### Intelligent Conflict Resolution
- Entity-type-aware merge strategies (sessions vs learner models)
- Timestamp-based precedence for immutable data
- Field-level merging for complex data structures
- Manual resolution fallback for complex conflicts

### Platform-Aware Cloud Integration
- Automatic cloud provider selection based on device platform
- iOS/macOS → iCloud, Android → Google Drive, Windows → OneDrive
- Storage quota monitoring and usage tracking
- Cross-platform compatibility with local fallback

### Data Integrity Assurance
- Checksum validation for sync packets
- Version-controlled incremental updates
- Corruption detection and recovery mechanisms
- Tombstone records for proper deletion handling

### Scalable Sync Architecture
- Incremental sync with filtering by entity type
- Batched operations for efficiency
- Connection state management across devices
- Background sync queue processing with retry logic