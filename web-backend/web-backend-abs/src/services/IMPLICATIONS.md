# Services Directory - Integration Guide

## Overview

The services directory contains the business logic layer that orchestrates complex operations across multiple domains. These services implement the core functionality for adaptive learning, analytics, authentication, auditing, batch processing, federation, privacy, and protocol management. Each service encapsulates domain-specific logic while maintaining clean separation of concerns.

## How Core and Apps Should Use This Functionality

### Core Integration Patterns

#### Service Layer Architecture
```rust
// Services implement the main business logic
use crate::services::{AdaptationService, LearnerService, ProtocolService};

// Services are dependency-injected into handlers
pub struct AppState {
    pub adaptation_service: Arc<AdaptationService>,
    pub learner_service: Arc<LearnerService>,
    pub protocol_service: Arc<ProtocolService>,
    // ... other services
}
```

#### Adaptive Learning Integration
```rust
// Core integrates with adaptation service for personalized learning
let adaptation_service = AdaptationService::new(learner_service);
let intervention = adaptation_service.determine_intervention(&learner_state).await?;
let adapted_task = adaptation_service.adapt_task(&task, &intervention).await?;
```

#### Privacy-First Design
```rust
// All services integrate with privacy accounting
use crate::services::{PrivacyAccountingService, privacy};

let privacy_budget = privacy_accounting.get_available_budget(&user_id).await?;
let anonymized_data = privacy::anonymize_data(&raw_data, privacy_budget)?;
```

### Application Usage Patterns

#### Batch Processing Integration
```rust
// Long-running operations use batch job service
let batch_service = BatchJobService::new(db_pool, metrics);
let job_id = batch_service.schedule_analytics_job(AnalyticsJobParams {
    user_cohort: "experiment_group_a",
    time_range: last_30_days,
}).await?;
```

#### Federation Service Usage
```rust
// Multi-institutional research collaboration
let federation_service = FederationService::new(db_pool, config);
let share_request = federation_service.create_share_request(
    protocol_data,
    target_institutions,
    privacy_level
).await?;
```

## State Machines and Lifecycle Patterns

### Adaptive Learning State Machine
```
Initial Assessment → Skill Modeling → Task Adaptation → Performance Monitoring → Model Update → Intervention
```

### Privacy Accounting Lifecycle
```
Budget Allocation → Query Tracking → Noise Addition → Budget Consumption → Renewal → Audit
```

### Federation Protocol Lifecycle
```
Node Registration → Agreement Negotiation → Data Sharing → Compliance Verification → Audit Trail
```

### Batch Job State Machine
```
Job Scheduling → Queue Processing → Execution → Progress Tracking → Completion → Cleanup
```

## Integration Patterns and Best Practices

### Service Composition Patterns
```rust
// Services compose together through dependency injection
impl AdaptationService {
    pub fn new(
        learner_service: Arc<LearnerService>,
        analytics_service: Arc<AnalyticsService>,
        privacy_service: Arc<PrivacyService>,
    ) -> Self {
        Self { learner_service, analytics_service, privacy_service }
    }
}
```

### Error Handling Strategy
```rust
// All services use consistent error handling
impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound => AppError::NotFound,
            ServiceError::Privacy(e) => AppError::PrivacyViolation(e),
            ServiceError::Federation(e) => AppError::FederationError(e),
            _ => AppError::InternalServer,
        }
    }
}
```

### Authentication Service Integration
```rust
// OAuth services provide unified authentication
pub async fn authenticate_user(
    provider: &str,
    token: &str,
) -> AppResult<AuthenticatedUser> {
    match provider {
        "github" => github_oauth_service.authenticate(token).await,
        "apple" => apple_auth_service.authenticate(token).await,
        _ => oauth_service.authenticate(provider, token).await,
    }
}
```

## Architectural Decisions and Constraints

### Domain-Driven Design
- Each service encapsulates a specific business domain
- Services maintain their own data models and validation rules
- Cross-cutting concerns (monitoring, privacy, audit) are handled consistently
- Clean interfaces between services prevent tight coupling

### Privacy-by-Design Architecture
- All services implement privacy accounting and differential privacy
- Data minimization principles applied at service boundaries
- Anonymization and pseudonymization built into data processing
- Privacy budgets enforced across all analytical operations

### Scalability Considerations
- Services designed for horizontal scaling with stateless operations
- Batch processing for computationally intensive operations
- Async/await patterns throughout for non-blocking I/O
- Connection pooling and resource management optimized per service

### Security Framework
- Authentication services provide secure token validation
- Audit service logs all significant operations with tamper-proof records
- Federation service implements mutual TLS and signature verification
- Privacy service ensures compliance with regulatory requirements

## Security and Performance Considerations

### Security Measures
- OAuth integration with multiple providers (GitHub, Apple, custom)
- Audit trails for all user actions and system operations
- Privacy-preserving analytics with differential privacy guarantees
- Secure federation protocols with digital signatures and encryption

### Performance Optimization
- Learner service uses efficient ML model caching and incremental updates
- Analytics service implements query optimization and result caching
- Batch job service queues expensive operations for background processing
- Federation service uses connection pooling for inter-institutional communication

### Resource Management
- Connection pools sized appropriately for each service's needs
- Memory management for ML models and analytics computations
- CPU scheduling for batch jobs to avoid impacting real-time operations
- Network optimization for federation and external API calls

## Common Usage Patterns and Examples

### Adaptive Learning Workflow
```rust
pub async fn adapt_learning_experience(
    user_id: Uuid,
    current_task: &Task,
    response_history: &[Response],
) -> AppResult<AdaptedExperience> {
    // Update learner model based on performance
    let learner_model = learner_service
        .update_model(user_id, response_history)
        .await?;
    
    // Determine if intervention is needed
    let struggle_level = adaptation_service
        .assess_struggle_level(&learner_model, current_task)
        .await?;
    
    // Apply appropriate intervention
    let intervention = match struggle_level {
        StruggleLevel::Severe => InterventionAction::ProvideHint(HintLevel::Worked),
        StruggleLevel::Moderate => InterventionAction::ProvideHint(HintLevel::Scaffold),
        StruggleLevel::Mild => InterventionAction::ProvideHint(HintLevel::Partial),
        StruggleLevel::None => InterventionAction::IncreaseDifficulty,
    };
    
    adaptation_service.apply_intervention(current_task, intervention).await
}
```

### Privacy-Preserving Analytics
```rust
pub async fn generate_analytics_report(
    cohort_params: CohortParams,
) -> AppResult<AnalyticsReport> {
    // Check privacy budget availability
    let privacy_budget = privacy_accounting_service
        .get_cohort_budget(&cohort_params)
        .await?;
    
    // Generate analytics with differential privacy
    let raw_analytics = analytics_service
        .compute_cohort_metrics(&cohort_params)
        .await?;
    
    let private_analytics = privacy_service
        .add_differential_privacy(&raw_analytics, privacy_budget)
        .await?;
    
    // Log privacy budget consumption
    privacy_accounting_service
        .consume_budget(&cohort_params, privacy_budget)
        .await?;
    
    Ok(AnalyticsReport::from(private_analytics))
}
```

### Federation Data Sharing
```rust
pub async fn share_research_data(
    protocol_id: Uuid,
    target_institutions: Vec<Uuid>,
    data_classification: DataClassification,
) -> AppResult<ShareResponse> {
    // Verify sharing agreements
    federation_service
        .verify_sharing_agreements(&target_institutions)
        .await?;
    
    // Apply privacy protection
    let protected_data = privacy_service
        .protect_shared_data(&raw_data, data_classification)
        .await?;
    
    // Create sharing request with digital signature
    let share_request = federation_service
        .create_signed_share_request(protected_data, target_institutions)
        .await?;
    
    // Log sharing action for audit
    audit_service
        .log_data_sharing(protocol_id, share_request.clone())
        .await?;
    
    federation_service.execute_share_request(share_request).await
}
```

### Batch Analytics Processing
```rust
pub async fn schedule_cohort_analysis(
    experiment_id: Uuid,
    analysis_params: AnalysisParams,
) -> AppResult<JobId> {
    // Schedule comprehensive analysis as batch job
    let job_id = batch_job_service
        .schedule_job(JobType::CohortAnalysis {
            experiment_id,
            params: analysis_params.clone(),
        })
        .await?;
    
    // Set up progress monitoring
    batch_job_service
        .set_progress_callback(job_id, |progress| {
            tracing::info!("Analysis progress: {}%", progress.percentage_complete);
        })
        .await?;
    
    Ok(job_id)
}
```

## Configuration Examples

### Service Configuration
```toml
[services]
# Adaptive learning configuration
adaptation_enabled = true
intervention_threshold = 0.3
hint_progression_timeout_seconds = 30

# Privacy settings
differential_privacy_enabled = true
global_privacy_budget = 1.0
budget_renewal_period_hours = 24

# Federation settings
federation_enabled = true
max_concurrent_shares = 5
signature_algorithm = "ED25519"

# Batch processing
batch_queue_size = 1000
max_concurrent_jobs = 4
job_timeout_minutes = 60

[services.oauth]
github_client_id = "your_github_client_id"
apple_client_id = "your_apple_client_id"
token_expiry_hours = 24

[services.analytics]
query_cache_ttl_minutes = 15
max_cohort_size = 10000
enable_real_time_analytics = true
```

This services layer provides the core business functionality while maintaining clean architecture, strong privacy guarantees, and excellent performance characteristics. Each service is designed to be independently testable and deployable while integrating seamlessly with the overall system architecture.