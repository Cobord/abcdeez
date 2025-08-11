# Federation Models Architecture

## Requirements and Dataflow

### Core Requirements
- Multi-institutional research collaboration and data sharing
- Secure node authentication and mutual TLS
- Protocol version synchronization across federation
- Compliance verification and audit trails
- Real-time node health monitoring and heartbeats
- Flexible data sharing agreements with granular permissions

### Data Flow Patterns
1. **Node Registration**: Institution → RegisterNodeRequest → Verification → RegisterNodeResponse → API Key
2. **Heartbeat Cycle**: Node → NodeHeartbeat → Status Update → Health Monitoring
3. **Data Sharing**: Source Node → FederationShareRequest → Signature Verification → Target Nodes
4. **Protocol Sync**: Node → ProtocolSyncRequest → Version Check → ProtocolSyncResponse → Changes
5. **Compliance Check**: Node → ComplianceVerificationRequest → Audit → ComplianceVerificationResponse

## High-level Purpose and Responsibilities

### Primary Purpose
Defines the data models for a distributed research federation that enables secure collaboration between educational and research institutions while maintaining strict compliance and data governance.

### Core Responsibilities
- **Node Management**: Registration, authentication, and lifecycle management of federation partners
- **Agreement Framework**: Formal data sharing and collaboration agreements with configurable rules
- **Protocol Synchronization**: Version control and change management for research protocols
- **Compliance Monitoring**: Automated verification of regulatory compliance (HIPAA, FERPA, GDPR, IRB)
- **Secure Data Exchange**: Encrypted data sharing with digital signatures and audit trails

## Key Abstractions and Interfaces

### Primary Entities
- **FederationNode**: Represents partner institutions with authentication and capability metadata
- **FederationAgreement**: Formal contracts between institutions defining sharing permissions
- **NodeHeartbeat**: Real-time health and activity status from federation partners
- **FederationShareRequest**: Secure data sharing requests with encryption and signatures

### Status Management
- **NodeStatus**: Active, Inactive, Suspended, Pending lifecycle states
- **AgreementStatus**: Proposed, Negotiating, Active, Expired, Terminated workflow states
- **ComplianceStatus**: Per-regulation compliance tracking with expiry dates

### Data Types
- **DataType**: ExperimentResults, ProtocolDefinition, LearnerProgress, AggregatedAnalytics, ComplianceReport
- **AgreementType**: DataSharing, ResearchCollaboration, ProtocolExchange, Full
- **ComplianceType**: HIPAA, FERPA, GDPR, IRB, DataRetention

## Data Transformations and Flow

### Node Lifecycle
```
RegisterNodeRequest → Verification → FederationNode → NodeHeartbeat → Status Updates
```

### Agreement Workflow  
```
Proposal → Negotiation → Active Agreement → Data Sharing Rules → Compliance Verification
```

### Data Sharing Process
```
ShareRequest → Signature Verification → Agreement Check → Encryption → Target Delivery
```

### Protocol Synchronization
```
ProtocolSyncRequest → Version Comparison → ProtocolChange Detection → Sync Response
```

## Dependencies and Interactions

### External Dependencies
- **chrono**: Timestamp management for heartbeats and expiry tracking
- **serde**: JSON serialization for API communication and data storage
- **sqlx**: Database persistence for nodes, agreements, and compliance records
- **uuid**: Unique identifiers for nodes, agreements, and protocols

### Internal System Interactions
- **Handlers**: Federation API endpoints consume these models for node registration and data sharing
- **Services**: Federation service layer uses these models for business logic and validation
- **Database**: All persistent models map to database tables via sqlx FromRow
- **Authentication**: Public key fields integrate with TLS and signature verification systems

## Architectural Patterns

### Security by Design
- Digital signatures for all data sharing requests
- Public key infrastructure for node authentication
- Mutual TLS requirements for secure communication
- Granular data sharing rules with compliance enforcement

### Event-Driven Architecture
- NodeHeartbeat enables real-time monitoring and alerting
- ProtocolChange tracking for version control and change propagation
- Compliance auditing with scheduled verification cycles

### Multi-Tenancy
- Institution-scoped data isolation via node IDs
- Flexible capability negotiation per partnership
- Agreement-based access control with expiration handling

### Compliance Framework
- Multi-standard support (HIPAA, FERPA, GDPR, IRB, DataRetention)
- Automated compliance verification with audit trails
- Evidence collection and reporting for regulatory requirements