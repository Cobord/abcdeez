# Federated Research Data Collection Guide

This guide explains how to use the federated data collection system to enable secure multi-institutional research collaboration while maintaining data sovereignty and privacy compliance.

## Overview

The federated research system allows multiple institutions to:
- **Collaborate on research studies** while keeping data at source institutions
- **Share standardized protocols** and research methodologies
- **Aggregate anonymized results** for increased statistical power
- **Maintain compliance** with GDPR, HIPAA, and other regulations
- **Ensure data quality** through distributed monitoring

## Architecture

### Federation Network Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                    Federation Network                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ University A│    │ University B│    │Research Inst│         │
│  │    Node     │◄──►│    Node     │◄──►│     C Node  │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│         │                   │                   │              │
│         ▼                   ▼                   ▼              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │Local Database│   │Local Database│   │Local Database│        │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Key Components

1. **Federation Network**: The overall research collaboration network
2. **Federation Nodes**: Individual institutional participants
3. **Shared Protocols**: Standardized research procedures
4. **Federated Studies**: Multi-site research projects
5. **Data Sharing Agreements**: Legal and technical frameworks
6. **Compliance Management**: Privacy and regulatory compliance

## Getting Started

### 1. Initialize Federation Network

```rust
// Create your institution's federation node
let contact = InstitutionContact {
    primary_investigator: "Dr. Jane Smith".to_string(),
    email: "jane.smith@university.edu".to_string(),
    institution: "University Research Center".to_string(),
    department: "Psychology".to_string(),
    irb_contact: "irb@university.edu".to_string(),
    data_protection_officer: "dpo@university.edu".to_string(),
};

let mut network = FederationNetwork::new(
    "University Research Center".to_string(),
    contact
);
```

### 2. Configure Data Governance

Set up your institution's data policies:

```rust
let data_policy = DataGovernancePolicy {
    data_retention_months: 84, // 7 years
    sharing_restrictions: vec![SharingRestriction::IRBApprovalRequired],
    anonymization_level: AnonymizationLevel::Pseudonymized,
    geographic_restrictions: vec![], // No geographic restrictions
    irb_approval_required: true,
    audit_requirements: AuditRequirement {
        logging_level: AuditLevel::Standard,
        retention_years: 7,
        external_audit: false,
        real_time_monitoring: true,
    },
};
```

### 3. Join the Network

```rust
// Discover and join existing networks
let federation_client = FederationClient::new(network.local_node.node_id.clone());

let discovery_result = federation_client.discover_network(vec![
    "https://federation.university-a.edu".to_string(),
    "https://federation.research-inst-b.org".to_string(),
]).await?;

// Add discovered peer nodes
for node in discovery_result.discovered_nodes {
    network.add_peer_node(node)?;
}
```

## Creating and Sharing Protocols

### 1. Define Experiment Template

```rust
let experiment_template = ExperimentTemplate {
    experiment_type: ExperimentType::LearningCurve,
    conditions: vec![
        ExperimentCondition {
            name: "Control".to_string(),
            parameters: HashMap::new(),
            control_group: true,
        },
        ExperimentCondition {
            name: "Experimental".to_string(),
            parameters: HashMap::new(),
            control_group: false,
        },
    ],
    min_trials: 100,
    max_duration_minutes: 60,
    randomization_scheme: RandomizationScheme::BlockRandomization(10),
    required_sensors: vec!["audio".to_string()],
    data_collection_points: vec![
        DataCollectionPoint {
            name: "Response Time".to_string(),
            data_type: DataType::ResponseTime,
            required: true,
            validation_rules: vec![
                ValidationRule::Range(100.0, 10000.0),
            ],
        },
        DataCollectionPoint {
            name: "Accuracy".to_string(),
            data_type: DataType::Accuracy,
            required: true,
            validation_rules: vec![],
        },
    ],
};
```

### 2. Create Data Schema

```rust
let mut field_definitions = HashMap::new();
field_definitions.insert("response_time_ms".to_string(), FieldDefinition {
    field_type: "integer".to_string(),
    description: "Response time in milliseconds".to_string(),
    constraints: vec!["min:100".to_string(), "max:10000".to_string()],
    privacy_level: PrivacyLevel::Public,
});

let data_schema = DataSchema {
    schema_version: "1.0".to_string(),
    field_definitions,
    required_fields: vec!["response_time_ms".to_string()].into_iter().collect(),
    anonymization_rules: HashMap::new(),
    validation_schema: "{}".to_string(), // JSON Schema
};
```

### 3. Share Protocol with Network

```rust
let protocol_id = network.create_shared_protocol(
    "Multi-Site Learning Study".to_string(),
    experiment_template,
    data_schema,
)?;

// Share with specific nodes
let target_nodes = vec!["node-university-b".to_string(), "node-research-inst-c".to_string()];
let shared_with = federation_client.share_protocol(
    &network.shared_protocols[&protocol_id],
    target_nodes
).await?;
```

## Participating in Federated Studies

### 1. Browse Available Protocols

Use the UI to browse shared protocols:
- Navigate to Research Dashboard
- Click "Browse Protocols"
- View available multi-site studies
- Check requirements and compatibility

### 2. Join a Study

```rust
// Join an existing protocol
network.join_protocol(&protocol_id)?;

// Propose participation in a study
let participation_proposal = ParticipationProposal {
    proposed_participants: 50,
    available_capabilities: vec!["audio_recording".to_string(), "eeg".to_string()],
    data_sharing_level: DataSharingLevel::AggregateOnly,
    timeline: ParticipationTimeline {
        enrollment_start: Utc::now() + Duration::days(30),
        enrollment_end: Utc::now() + Duration::days(180),
        data_collection_end: Utc::now() + Duration::days(210),
    },
};

let response = federation_client.request_study_participation(
    study_id,
    coordinator_endpoint,
    participation_proposal
).await?;
```

## Data Sharing and Privacy

### Anonymization Levels

1. **Identified**: Contains personally identifiable information (PII)
   - Use: Internal analysis only
   - Sharing: Not permitted across institutions

2. **Pseudonymized**: Reversible anonymization with key management
   - Use: Cross-institutional analysis with agreements
   - Sharing: Requires specific data sharing agreements

3. **Anonymized**: Irreversible anonymization
   - Use: General research sharing
   - Sharing: Permitted with institutional approval

4. **Aggregate**: Only summary statistics
   - Use: Meta-analyses and publication
   - Sharing: Broadly permitted

### Compliance Standards

The system supports multiple compliance frameworks:

- **GDPR** (EU General Data Protection Regulation)
  - Right to be forgotten
  - Consent management
  - Cross-border transfer restrictions

- **HIPAA** (US Health Insurance Portability)
  - Protected health information
  - Business associate agreements
  - Audit trail requirements

- **Common Rule** (US Federal Research Regulations)
  - IRB approval requirements
  - Informed consent procedures
  - Risk-benefit analysis

- **FERPA** (US Family Educational Rights)
  - Educational record protection
  - Directory information policies
  - Parental consent requirements

## Study Coordination

### 1. Initiate Federated Study

```rust
let target_participants = HashMap::from([
    ("node-university-a".to_string(), 100),
    ("node-university-b".to_string(), 75),
    ("node-research-inst-c".to_string(), 50),
]);

let study_id = network.initiate_federated_study(
    protocol_id,
    "Multi-Site Cognitive Learning Study".to_string(),
    target_participants,
)?;
```

### 2. Monitor Study Progress

Track enrollment and data quality across sites:

```rust
let study = &network.active_studies[&study_id];
for (node_id, participation) in &study.participating_nodes {
    println!("Site {}: {} participants, {:.1}% quality", 
        node_id, 
        study.current_enrollment[node_id],
        participation.quality_metrics.data_quality_score * 100.0
    );
}
```

### 3. Coordinate Interim Analyses

```rust
let analysis_proposal = InterimAnalysisProposal {
    analysis_type: AnalysisType::Efficacy,
    data_cutoff_date: Utc::now(),
    proposed_methods: vec!["t-test".to_string(), "effect_size".to_string()],
    stopping_rules: vec![
        StoppingRule {
            rule_type: "efficacy".to_string(),
            threshold: 0.01,
            description: "Stop for efficacy if p < 0.01".to_string(),
        },
    ],
};

let analysis_response = federation_client.coordinate_interim_analysis(
    study_id,
    coordinator_endpoint,
    analysis_proposal
).await?;
```

## Data Aggregation and Meta-Analysis

### 1. Share Aggregate Statistics

```rust
let aggregates = HashMap::from([
    ("mean_response_time".to_string(), json!(1250.5)),
    ("accuracy_rate".to_string(), json!(0.847)),
    ("participant_count".to_string(), json!(67)),
    ("effect_size_cohens_d".to_string(), json!(0.42)),
]);

let shared_with = federation_client.share_aggregate_data(
    study_id,
    aggregates,
    target_nodes
).await?;
```

### 2. Perform Meta-Analysis

Aggregate results across institutions:

```rust
// Collect data from all participating sites
let mut all_site_data = Vec::new();
for (node_id, _) in &study.participating_nodes {
    if let Some(site_data) = fetch_site_aggregates(node_id).await {
        all_site_data.push(site_data);
    }
}

// Perform meta-analysis
let meta_analysis_result = perform_fixed_effects_meta_analysis(&all_site_data);
```

## Security and Encryption

### 1. End-to-End Encryption

All inter-institutional communications use AES-256-GCM encryption:

```rust
let federation_client = FederationClient::new(local_node_id);
// Automatic encryption/decryption for all network communications
```

### 2. Key Management

- **Automatic key generation** for each federation network
- **Key rotation** every 90 days
- **Secure key distribution** using public key cryptography
- **Key escrow** for regulatory compliance (optional)

### 3. Audit Logging

All federation activities are logged:

```rust
// Automatic audit logging for:
// - Data sharing requests
// - Protocol sharing
// - Node joining/leaving
// - Compliance checks
// - Data access attempts
```

## Compliance Monitoring

### 1. Real-Time Compliance Checks

```rust
let compliance_report = federation_client.verify_network_compliance(&network).await?;

if !compliance_report.overall_compliant {
    for (node_id, compliant) in compliance_report.node_compliance {
        if !compliant {
            println!("Node {} is not compliant", node_id);
        }
    }
}
```

### 2. Regular Audits

- **Quarterly compliance reviews**
- **Annual security assessments**
- **IRB coordination and updates**
- **Data retention policy enforcement**

## UI Features

### Federation Dashboard

The research dashboard provides:

1. **Network Status**: View connected institutions and network health
2. **Protocol Browser**: Discover and join shared research protocols
3. **Study Coordination**: Monitor multi-site study progress
4. **Data Sharing Controls**: Manage privacy settings and sharing permissions
5. **Compliance Monitor**: Track regulatory compliance across the network

### Key UI Components

- **🌐 Setup Federation**: Initialize your institution's network participation
- **📋 Browse Protocols**: Explore available multi-site research opportunities
- **📈 Study Coordination**: Real-time monitoring of federated studies
- **🔒 Privacy Settings**: Configure data sharing and anonymization policies
- **📊 Share Aggregates**: Contribute anonymized data to network studies
- **🔑 Key Management**: Manage encryption keys and secure communications

## Best Practices

### 1. Data Governance

- **Establish clear data policies** before joining networks
- **Regular review** of sharing agreements and compliance requirements
- **Staff training** on federated research protocols and privacy requirements
- **Technical safeguards** including encryption, access controls, and audit logs

### 2. Quality Assurance

- **Standardized protocols** to ensure data consistency across sites
- **Regular data quality checks** and validation procedures
- **Inter-site calibration** of equipment and procedures
- **Coordinated training** for research staff at all participating sites

### 3. Ethical Considerations

- **Multi-site IRB coordination** to ensure consistent ethical oversight
- **Transparent consent processes** that inform participants about data sharing
- **Clear data ownership** and responsibility agreements
- **Participant rights protection** across all participating institutions

### 4. Technical Implementation

- **Robust error handling** for network communications
- **Graceful degradation** when sites are temporarily unavailable
- **Regular backups** and disaster recovery procedures
- **Performance monitoring** to ensure system responsiveness

## Troubleshooting

### Common Issues

1. **Node Connection Failures**
   - Check network connectivity and firewall settings
   - Verify API endpoints and authentication credentials
   - Review SSL/TLS certificate validity

2. **Compliance Warnings**
   - Update institutional policies and approvals
   - Renew data sharing agreements
   - Complete required staff training

3. **Data Quality Issues**
   - Review protocol adherence across sites
   - Check data validation rules and constraints
   - Coordinate calibration procedures

4. **Synchronization Problems**
   - Verify system clocks are synchronized
   - Check for version mismatches in protocols
   - Review network latency and timeout settings

### Support and Resources

- **Technical Documentation**: Comprehensive API and configuration guides
- **Training Materials**: Video tutorials and best practices guides
- **Community Forum**: Peer support and knowledge sharing
- **Professional Support**: Dedicated technical support for institutional subscribers

This federated research system enables unprecedented collaboration while maintaining the highest standards of data privacy, security, and regulatory compliance. By participating in the federation network, institutions can contribute to larger, more powerful studies while maintaining full control over their data and research processes.