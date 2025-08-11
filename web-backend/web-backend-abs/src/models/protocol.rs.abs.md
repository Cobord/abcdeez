# Protocol Models Architecture

## Requirements and Dataflow

### Core Requirements
- Version-controlled research protocol management with semantic versioning
- Git-like branching and merging for protocol development
- Immutable published protocol versions with deprecation lifecycle
- Real-time protocol validation and difference comparison
- Federation-aware protocol publishing and synchronization
- Performance metrics tracking for protocol effectiveness

### Data Flow Patterns
1. **Protocol Creation**: CreateProtocolRequest → Validation → Protocol + Initial ProtocolVersion
2. **Version Development**: CreateVersionRequest → Validation → ProtocolVersion → Draft Status
3. **Publishing Workflow**: PublishVersionRequest → Validation → Status Update → Federation Notification
4. **Branching/Merging**: Base Version → Branch Creation → Development → Merge → Version Update
5. **Comparison Analysis**: CompareVersionsRequest → Diff Calculation → VersionDifference → Response
6. **Metrics Collection**: Protocol Usage → Session Data → Performance Calculation → ProtocolMetrics

## High-level Purpose and Responsibilities

### Primary Purpose
Provides a comprehensive version control system for research protocols, enabling collaborative development, controlled publication, and performance tracking while maintaining scientific reproducibility and integrity.

### Core Responsibilities
- **Version Management**: Semantic versioning with parent-child relationships and branching
- **Publication Control**: Status-based lifecycle from draft through published to deprecated
- **Collaboration Framework**: Git-like branching and merging for team protocol development
- **Validation System**: Rule-based protocol validation with error and warning reporting
- **Change Tracking**: Detailed difference analysis between protocol versions
- **Performance Analysis**: Usage metrics and effectiveness tracking for optimization

## Key Abstractions and Interfaces

### Core Entities
- **Protocol**: Container entity grouping related protocol versions
- **ProtocolVersion**: Immutable version snapshot with complete definition and metadata
- **ProtocolBranch**: Development branch for experimental protocol variations
- **ProtocolMetrics**: Performance and usage analytics for protocol optimization

### Lifecycle Management
- **ProtocolStatus**: Draft, Review, Published, Deprecated, Archived state machine
- **CreateProtocolRequest/Response**: Initial protocol and version creation
- **PublishVersionRequest**: Controlled publication with federation notification
- **CompareVersionsRequest/Response**: Detailed version difference analysis

### Validation Framework
- **ProtocolValidationResult**: Comprehensive validation with errors and warnings
- **ValidationError/Warning**: Structured validation feedback with field-level details
- **VersionDifference**: Change detection with typed modification tracking

## Data Transformations and Flow

### Version Creation Workflow
```
CreateVersionRequest → Definition Validation → Parent Linking → ProtocolVersion → Draft Status
```

### Publishing Pipeline
```
Draft Version → Validation → Review Status → Publication → Federation Sync → Current Version Update
```

### Branching and Merging
```
Base Version → Branch Creation → Development → Comparison → Merge → Version Integration
```

### Difference Analysis
```
Version A + Version B → Field Comparison → Change Detection → Diff Calculation → Structured Response
```

## Dependencies and Interactions

### External Dependencies
- **chrono**: Timestamp management for version tracking and lifecycle events
- **serde**: JSON serialization for protocol definitions and validation rules
- **sqlx**: Database persistence with FromRow mapping and type safety
- **uuid**: Unique identifiers for protocols, versions, and branches

### Internal System Interactions
- **Handlers**: Protocol API endpoints consume these models for CRUD and comparison operations
- **Services**: Protocol service layer implements validation, versioning, and federation logic
- **Federation**: Protocol synchronization with partner institutions via federation models
- **Analytics**: Metrics collection integrates with performance tracking systems
- **Experiments**: Protocol definitions link to experiment configurations and execution

## Architectural Patterns

### Git-Inspired Version Control
- Parent-child version relationships with branching support
- Semantic versioning with automated progression
- Merge tracking between branches and main line
- Change history with comprehensive changelog requirements

### Immutable Published Versions
- Published versions cannot be modified, only deprecated
- Definition and validation rules frozen at publication
- Timestamped lifecycle events (creation, publication, deprecation)
- Hash-based integrity verification for protocol definitions

### Status-Based Lifecycle Management
- State machine progression: Draft → Review → Published → Deprecated → Archived
- Role-based permissions for status transitions
- Automated validation requirements for status advancement
- Federation notification triggers on publication events

### Extensible Validation Framework
- JSON-based validation rules with custom logic support
- Field-level error and warning reporting
- Pluggable validation engine with rule inheritance
- Performance impact assessment for validation complexity

### Performance-Driven Optimization
- Real-time metrics collection from protocol execution
- Effectiveness tracking with completion and success rates
- Usage pattern analysis for protocol improvement recommendations
- A/B testing support through branch comparison metrics