# Compliance and Experimental Framework Design for Xilem-App and Web-Backend

## Overview

This document outlines how the xilem-app and web-backend should implement the compliance infrastructure and experimental framework described in the core-abs IMPLICATIONS.

## Architecture Split

### Frontend (xilem-app)
- User consent collection and display
- Real-time compliance status indicators
- Experiment participation UI
- Local audit event generation
- Session recording metadata

### Backend (web-backend)
- Centralized audit trail storage
- IRB document generation
- Pre-registration management (already partial)
- Experiment randomization service
- Compliance validation and reporting

---

## 1. Compliance Infrastructure Implementation

### 1.1 Audit Trail System

#### Backend Implementation
```rust
// web-backend/src/services/audit_trail.rs
use abcdeez_core::compliance::audit_trail::{AuditTrailManager, AuditEvent};

pub struct AuditService {
    manager: AuditTrailManager,
    db_pool: PgPool,
}

impl AuditService {
    pub async fn log_event(&self, event: AuditEvent) -> Result<()> {
        // Store in database with immutability guarantee
        sqlx::query!(
            "INSERT INTO audit_events (id, session_id, user_id, event_type, 
             details, outcome, timestamp, hash, previous_hash) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            event.id,
            event.session_id,
            event.user_id,
            event.event_type.to_string(),
            serde_json::to_value(&event.details)?,
            event.outcome.to_string(),
            event.timestamp,
            self.calculate_hash(&event),
            self.get_previous_hash().await?
        )
        .execute(&self.db_pool)
        .await?;
        
        // Also log to core manager for in-memory analysis
        self.manager.log_event(event);
        Ok(())
    }
    
    fn calculate_hash(&self, event: &AuditEvent) -> String {
        // Cryptographic hash for tamper detection
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(event).unwrap());
        format!("{:x}", hasher.finalize())
    }
}
```

#### App Integration
```rust
// xilem-app/src/services/audit_client.rs
pub struct AuditClient {
    local_queue: VecDeque<AuditEvent>,
    sync_service: Arc<SyncService>,
}

impl AuditClient {
    pub fn log_user_action(&mut self, action: &str, details: Value) {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            session_id: self.current_session_id(),
            user_id: self.current_user_id(),
            event_type: EventType::UserAction,
            action: action.to_string(),
            details,
            outcome: Outcome::Success,
            timestamp: Utc::now(),
        };
        
        // Queue locally
        self.local_queue.push_back(event.clone());
        
        // Schedule sync
        self.sync_service.queue_audit_event(event);
    }
}

// In AppState
impl AppState {
    pub fn track_action(&mut self, action: &str) {
        if let Some(audit) = &mut self.audit_client {
            audit.log_user_action(action, json!({
                "screen": self.current_screen.title(),
                "task_id": self.session.as_ref().map(|s| s.current_task_id),
            }));
        }
    }
}
```

### 1.2 IRB Compliance Workflow

#### Backend IRB Document Generation
```rust
// web-backend/src/services/irb.rs
use abcdeez_core::compliance::irb::{IRBComplianceGenerator, IRBApplication};

pub struct IRBService {
    generator: IRBComplianceGenerator,
}

impl IRBService {
    pub async fn generate_application(
        &self,
        experiment_id: &str,
        state: &AppState,
    ) -> Result<IRBApplication> {
        // Fetch experiment details
        let experiment = self.get_experiment(experiment_id, &state.db_pool).await?;
        
        // Fetch pre-registration
        let prereg = self.get_preregistration(&experiment.prereg_id, &state.db_pool).await?;
        
        // Generate IRB application
        let application = self.generator.generate_irb_application(
            &experiment,
            &prereg,
            &self.get_risk_assessment(&experiment).await?,
        )?;
        
        // Store generated documents
        self.store_irb_documents(&application, &state.db_pool).await?;
        
        Ok(application)
    }
    
    pub async fn check_compliance_status(&self, experiment_id: &str) -> ComplianceStatus {
        // Check all compliance requirements
        ComplianceStatus {
            irb_approved: self.has_valid_irb_approval(experiment_id).await,
            consent_forms_ready: self.has_consent_forms(experiment_id).await,
            data_management_plan: self.has_data_plan(experiment_id).await,
            pre_registered: self.is_preregistered(experiment_id).await,
        }
    }
}
```

#### App Consent Collection
```rust
// xilem-app/src/views/consent.rs
pub fn consent_view(state: &mut AppState) -> impl WidgetView<AppState> {
    flex((
        // IRB-approved consent form display
        scroll(
            label(&state.experiment.consent_text)
                .text_size(14.0)
        ).expand(),
        
        // Consent checkboxes
        checkbox(
            "I understand the study procedures",
            state.consent.understands_procedures
        ),
        checkbox(
            "I consent to data collection",
            state.consent.data_collection
        ),
        checkbox(
            "I understand I can withdraw at any time",
            state.consent.withdrawal_rights
        ),
        
        // Consent buttons
        flex_row((
            button("Decline", |state: &mut AppState| {
                state.audit_client.log_user_action("consent_declined", json!({}));
                state.navigate(Screen::Exit);
            }),
            button("Accept", |state: &mut AppState| {
                if state.consent.is_complete() {
                    state.audit_client.log_user_action("consent_granted", json!({
                        "timestamp": Utc::now(),
                        "version": state.experiment.consent_version,
                    }));
                    state.navigate(Screen::Learning);
                }
            }).enabled(state.consent.all_checked()),
        )),
    ))
}
```

### 1.3 Pre-Registration Integration

#### Enhanced Backend Pre-Registration
```rust
// web-backend/src/handlers/preregistration.rs
pub async fn finalize_preregistration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PreRegistrationResponse>, AppError> {
    // Generate immutable hash
    let prereg = get_preregistration_data(&id, &state.db_pool).await?;
    let hash = calculate_registration_hash(&prereg);
    
    // Lock the pre-registration
    sqlx::query!(
        "UPDATE preregistrations 
         SET status = 'registered', 
             registration_hash = $1,
             registered_at = $2,
             locked = true
         WHERE id = $3 AND status = 'draft'",
        hash,
        Utc::now(),
        id
    )
    .execute(&state.db_pool)
    .await?;
    
    // Create audit entry
    state.audit_service.log_event(AuditEvent {
        event_type: EventType::PreRegistrationFinalized,
        details: json!({ "prereg_id": id, "hash": hash }),
        ..Default::default()
    }).await?;
    
    // Generate OSF registration (if configured)
    if state.config.osf_integration_enabled {
        submit_to_osf(&prereg, &hash).await?;
    }
    
    Ok(Json(response))
}
```

#### App Pre-Registration Status Display
```rust
// xilem-app/src/components/prereg_status.rs
pub fn prereg_status_widget(state: &AppState) -> impl WidgetView<AppState> {
    if let Some(prereg) = &state.experiment.preregistration {
        flex_row((
            // Status icon
            icon(match prereg.status {
                "registered" => "✓",
                "draft" => "📝",
                _ => "⚠"
            }),
            
            // Registration info
            label(format!("Pre-registered: {}", 
                prereg.registered_at.map_or("Draft".to_string(), |d| d.format("%Y-%m-%d").to_string())
            )),
            
            // Hash verification
            if let Some(hash) = &prereg.hash {
                Either::A(
                    button("Verify", move |_state: &mut AppState| {
                        // Open hash verification dialog
                    })
                )
            } else {
                Either::B(empty())
            }
        ))
    } else {
        label("Not pre-registered").color(Color::rgb8(255, 0, 0))
    }
}
```

---

## 2. Experimental Framework Implementation

### 2.1 Experiment Configuration and Randomization

#### Backend Experiment Service
```rust
// web-backend/src/services/experiment.rs
use abcdeez_core::experiments::{
    design::{ExperimentDesigner, Factor},
    ab_testing::ABTest,
    multi_session::MultiSessionManager,
};

pub struct ExperimentService {
    db_pool: PgPool,
    randomizer: StdRng,
}

impl ExperimentService {
    pub async fn create_experiment(&self, config: ExperimentConfig) -> Result<Experiment> {
        // Design experiment with core module
        let design = ExperimentDesigner::new()
            .add_between_subjects_factor("condition", config.conditions.clone())
            .add_within_subjects_factor("session", (1..=config.n_sessions).collect())
            .with_counterbalancing(config.counterbalance)
            .build()?;
        
        // Store experiment
        let experiment = Experiment {
            id: Uuid::new_v4(),
            design: serde_json::to_value(&design)?,
            config,
            status: ExperimentStatus::Setup,
            created_at: Utc::now(),
        };
        
        self.store_experiment(&experiment).await?;
        Ok(experiment)
    }
    
    pub async fn assign_participant(&self, 
        experiment_id: &str, 
        participant_id: &str
    ) -> Result<ConditionAssignment> {
        // Get experiment design
        let experiment = self.get_experiment(experiment_id).await?;
        let design: ExperimentDesign = serde_json::from_value(experiment.design)?;
        
        // Check current condition balance
        let condition_counts = self.get_condition_counts(experiment_id).await?;
        
        // Assign to condition with minimum participants (balanced assignment)
        let assigned_condition = design.conditions
            .iter()
            .min_by_key(|c| condition_counts.get(*c).unwrap_or(&0))
            .unwrap()
            .clone();
        
        // Generate randomized trial order if needed
        let trial_order = if experiment.config.randomize_trials {
            let mut trials: Vec<usize> = (0..experiment.config.n_trials).collect();
            trials.shuffle(&mut self.randomizer);
            Some(trials)
        } else {
            None
        };
        
        // Store assignment
        let assignment = ConditionAssignment {
            participant_id: participant_id.to_string(),
            experiment_id: experiment_id.to_string(),
            condition: assigned_condition,
            trial_order,
            session_schedule: self.generate_session_schedule(&experiment.config),
            assigned_at: Utc::now(),
        };
        
        self.store_assignment(&assignment).await?;
        
        // Audit trail
        self.audit_service.log_event(AuditEvent {
            event_type: EventType::ParticipantRandomized,
            details: json!({
                "participant": participant_id,
                "condition": assignment.condition,
                "method": "balanced_random"
            }),
            ..Default::default()
        }).await?;
        
        Ok(assignment)
    }
}
```

#### App Experiment Participation
```rust
// xilem-app/src/services/experiment_client.rs
pub struct ExperimentClient {
    current_experiment: Option<ExperimentInfo>,
    condition_assignment: Option<ConditionAssignment>,
    session_manager: MultiSessionManager,
}

impl ExperimentClient {
    pub async fn join_experiment(&mut self, code: &str, api: &ApiClient) -> Result<()> {
        // Fetch experiment details
        let experiment = api.get_experiment_by_code(code).await?;
        
        // Check eligibility
        if !self.check_eligibility(&experiment).await? {
            return Err(anyhow!("Not eligible for this experiment"));
        }
        
        // Get condition assignment from backend
        let assignment = api.request_condition_assignment(
            &experiment.id,
            &self.participant_id
        ).await?;
        
        // Initialize session manager
        self.session_manager = MultiSessionManager::new(
            experiment.config.n_sessions,
            assignment.session_schedule.clone(),
        );
        
        self.current_experiment = Some(experiment);
        self.condition_assignment = Some(assignment);
        
        Ok(())
    }
    
    pub fn get_next_task(&mut self) -> Option<Task> {
        let assignment = self.condition_assignment.as_ref()?;
        
        // Apply condition-specific task selection
        match assignment.condition.as_str() {
            "adaptive" => self.adaptive_scheduler.select_next_task(),
            "linear" => self.linear_scheduler.select_next_task(),
            "random" => self.random_scheduler.select_next_task(),
            _ => None
        }
    }
}
```

### 2.2 Multi-Session Management

#### Backend Session Coordination
```rust
// web-backend/src/services/session_manager.rs
pub struct SessionCoordinator {
    db_pool: PgPool,
}

impl SessionCoordinator {
    pub async fn start_session(
        &self,
        participant_id: &str,
        experiment_id: &str,
        session_number: usize,
    ) -> Result<SessionInfo> {
        // Verify session is scheduled and ready
        let assignment = self.get_assignment(participant_id, experiment_id).await?;
        
        if !assignment.is_session_ready(session_number) {
            return Err(AppError::SessionNotReady);
        }
        
        // Check for incomplete previous sessions
        if session_number > 1 {
            let prev_complete = self.is_session_complete(
                participant_id, 
                experiment_id, 
                session_number - 1
            ).await?;
            
            if !prev_complete {
                return Err(AppError::PreviousSessionIncomplete);
            }
        }
        
        // Create session
        let session = Session {
            id: Uuid::new_v4(),
            participant_id: participant_id.to_string(),
            experiment_id: experiment_id.to_string(),
            session_number,
            condition: assignment.condition.clone(),
            started_at: Utc::now(),
            status: SessionStatus::Active,
        };
        
        self.store_session(&session).await?;
        
        Ok(SessionInfo::from(session))
    }
    
    pub async fn complete_session(
        &self,
        session_id: &str,
        summary: SessionSummary,
    ) -> Result<()> {
        // Update session status
        sqlx::query!(
            "UPDATE sessions 
             SET status = 'completed',
                 ended_at = $1,
                 summary = $2
             WHERE id = $3",
            Utc::now(),
            serde_json::to_value(&summary)?,
            session_id
        )
        .execute(&self.db_pool)
        .await?;
        
        // Check if experiment is complete for participant
        self.check_experiment_completion(session_id).await?;
        
        Ok(())
    }
}
```

#### App Multi-Session UI
```rust
// xilem-app/src/views/session_manager.rs
pub fn session_overview(state: &AppState) -> impl WidgetView<AppState> {
    if let Some(experiment) = &state.experiment_info {
        flex((
            // Experiment header
            label(&experiment.title).text_size(20.0),
            
            // Session progress
            flex_row(
                experiment.sessions.iter().enumerate().map(|(i, session)| {
                    let status = match session.status {
                        SessionStatus::Completed => "✓",
                        SessionStatus::Active => "→",
                        SessionStatus::Scheduled => "○",
                        SessionStatus::Locked => "🔒",
                    };
                    
                    button(
                        format!("Session {} {}", i + 1, status),
                        move |state: &mut AppState| {
                            if session.is_available() {
                                state.start_session(i + 1);
                            }
                        }
                    )
                    .enabled(session.is_available())
                })
                .collect::<Vec<_>>()
            ),
            
            // Next session info
            if let Some(next) = experiment.get_next_session() {
                Either::A(
                    label(format!("Next session available: {}", 
                        next.scheduled_for.format("%Y-%m-%d %H:%M")))
                )
            } else {
                Either::B(label("All sessions completed!"))
            }
        ))
    } else {
        label("No active experiment")
    }
}
```

### 2.3 A/B Testing Framework

#### Backend A/B Test Analysis
```rust
// web-backend/src/services/ab_testing.rs
use abcdeez_core::experiments::ab_testing::{ABTest, ABTestResults};

pub struct ABTestService {
    db_pool: PgPool,
}

impl ABTestService {
    pub async fn analyze_ab_test(
        &self,
        experiment_id: &str,
    ) -> Result<ABTestResults> {
        // Fetch all response data
        let responses = self.get_all_responses(experiment_id).await?;
        
        // Separate by condition
        let control_data = responses.iter()
            .filter(|r| r.condition == "control")
            .collect();
        let treatment_data = responses.iter()
            .filter(|r| r.condition == "treatment")
            .collect();
        
        // Run statistical analysis
        let ab_test = ABTest::new(control_data, treatment_data);
        let results = ab_test.analyze()?;
        
        // Store results
        self.store_results(&results, experiment_id).await?;
        
        Ok(results)
    }
    
    pub async fn get_interim_results(
        &self,
        experiment_id: &str,
    ) -> Result<InterimAnalysis> {
        let current_n = self.get_participant_count(experiment_id).await?;
        let target_n = self.get_target_sample_size(experiment_id).await?;
        
        // Run interim analysis if we have enough data
        if current_n >= target_n / 2 {
            let results = self.analyze_ab_test(experiment_id).await?;
            
            Ok(InterimAnalysis {
                current_n,
                target_n,
                effect_size: results.effect_size,
                p_value: results.p_value,
                confidence_interval: results.confidence_interval,
                recommendation: self.get_recommendation(&results, current_n, target_n),
            })
        } else {
            Ok(InterimAnalysis {
                current_n,
                target_n,
                recommendation: "Continue data collection".to_string(),
                ..Default::default()
            })
        }
    }
}
```

---

## 3. Integration Points

### 3.1 App-Backend Communication

```rust
// xilem-app/src/services/research_api.rs
pub struct ResearchApiClient {
    base_client: ApiClient,
    compliance_endpoints: ComplianceEndpoints,
    experiment_endpoints: ExperimentEndpoints,
}

impl ResearchApiClient {
    // Compliance operations
    pub async fn submit_consent(&self, consent: ConsentData) -> Result<()> {
        self.base_client.post("/compliance/consent", &consent).await
    }
    
    pub async fn get_compliance_status(&self, experiment_id: &str) -> Result<ComplianceStatus> {
        self.base_client.get(&format!("/compliance/status/{}", experiment_id)).await
    }
    
    // Experiment operations
    pub async fn join_experiment(&self, code: &str) -> Result<ExperimentInfo> {
        self.base_client.post("/experiments/join", &json!({ "code": code })).await
    }
    
    pub async fn get_condition_assignment(&self, experiment_id: &str) -> Result<ConditionAssignment> {
        self.base_client.get(&format!("/experiments/{}/assignment", experiment_id)).await
    }
    
    // Pre-registration operations
    pub async fn get_preregistration(&self, experiment_id: &str) -> Result<PreRegistration> {
        self.base_client.get(&format!("/preregistrations/experiment/{}", experiment_id)).await
    }
}
```

### 3.2 State Synchronization

```rust
// xilem-app/src/state/research_state.rs
#[derive(Debug, Clone)]
pub struct ResearchState {
    // Compliance
    pub consent_status: ConsentStatus,
    pub audit_queue: VecDeque<AuditEvent>,
    pub compliance_indicators: ComplianceIndicators,
    
    // Experiment
    pub experiment_info: Option<ExperimentInfo>,
    pub condition_assignment: Option<ConditionAssignment>,
    pub session_schedule: Vec<SessionInfo>,
    pub current_session: Option<usize>,
    
    // Pre-registration
    pub preregistration: Option<PreRegistrationInfo>,
    pub deviation_log: Vec<Deviation>,
}

impl ResearchState {
    pub fn sync_with_backend(&mut self, backend_state: BackendResearchState) {
        self.compliance_indicators = backend_state.compliance;
        self.experiment_info = backend_state.experiment;
        self.preregistration = backend_state.preregistration;
        
        // Merge deviation logs
        for deviation in backend_state.deviations {
            if !self.deviation_log.contains(&deviation) {
                self.deviation_log.push(deviation);
            }
        }
    }
}
```

---

## 4. Database Schema Requirements

### Compliance Tables
```sql
-- Audit trail with cryptographic integrity
CREATE TABLE audit_events (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    user_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    action VARCHAR(255),
    details JSONB,
    outcome VARCHAR(50),
    timestamp TIMESTAMPTZ NOT NULL,
    hash VARCHAR(64) NOT NULL,
    previous_hash VARCHAR(64),
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- IRB documents and approvals
CREATE TABLE irb_documents (
    id UUID PRIMARY KEY,
    experiment_id UUID NOT NULL,
    document_type VARCHAR(100) NOT NULL,
    content TEXT NOT NULL,
    version INTEGER NOT NULL,
    approved BOOLEAN DEFAULT FALSE,
    approved_by VARCHAR(255),
    approved_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (experiment_id) REFERENCES experiments(id)
);

-- Consent records
CREATE TABLE consent_records (
    id UUID PRIMARY KEY,
    participant_id UUID NOT NULL,
    experiment_id UUID NOT NULL,
    consent_version VARCHAR(50) NOT NULL,
    consent_text TEXT NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL,
    withdrawn_at TIMESTAMPTZ,
    ip_address INET,
    signature_data JSONB,
    FOREIGN KEY (participant_id) REFERENCES participants(id),
    FOREIGN KEY (experiment_id) REFERENCES experiments(id)
);
```

### Experiment Tables
```sql
-- Experiment definitions
CREATE TABLE experiments (
    id UUID PRIMARY KEY,
    code VARCHAR(20) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    design JSONB NOT NULL,
    config JSONB NOT NULL,
    preregistration_id UUID,
    status VARCHAR(50) NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    FOREIGN KEY (preregistration_id) REFERENCES preregistrations(id)
);

-- Condition assignments
CREATE TABLE condition_assignments (
    id UUID PRIMARY KEY,
    experiment_id UUID NOT NULL,
    participant_id UUID NOT NULL,
    condition VARCHAR(100) NOT NULL,
    trial_order JSONB,
    session_schedule JSONB,
    assigned_at TIMESTAMPTZ NOT NULL,
    assignment_method VARCHAR(50) NOT NULL,
    UNIQUE(experiment_id, participant_id),
    FOREIGN KEY (experiment_id) REFERENCES experiments(id),
    FOREIGN KEY (participant_id) REFERENCES participants(id)
);

-- Multi-session tracking
CREATE TABLE experiment_sessions (
    id UUID PRIMARY KEY,
    experiment_id UUID NOT NULL,
    participant_id UUID NOT NULL,
    session_number INTEGER NOT NULL,
    condition VARCHAR(100) NOT NULL,
    scheduled_for TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL,
    summary JSONB,
    UNIQUE(experiment_id, participant_id, session_number),
    FOREIGN KEY (experiment_id) REFERENCES experiments(id),
    FOREIGN KEY (participant_id) REFERENCES participants(id)
);
```

---

## 5. Implementation Priority

### Phase 1: Foundation (Week 1-2)
1. **Audit Trail**: Basic event logging in app, storage in backend
2. **Consent Flow**: Simple consent collection and storage
3. **Experiment Join**: Basic experiment enrollment

### Phase 2: Core Features (Week 3-4)
1. **Condition Assignment**: Balanced randomization
2. **Session Management**: Multi-session scheduling
3. **Pre-registration Display**: Show pre-reg status in app

### Phase 3: Compliance (Week 5-6)
1. **IRB Workflow**: Document generation and tracking
2. **Audit Integrity**: Cryptographic hash chain
3. **Compliance Dashboard**: Status indicators in app

### Phase 4: Advanced (Week 7-8)
1. **A/B Testing**: Statistical analysis endpoints
2. **Deviation Tracking**: Pre-registration deviation logging
3. **Export Functions**: Research-grade data export

---

## 6. Testing Strategy

### Compliance Testing
```rust
#[cfg(test)]
mod compliance_tests {
    #[test]
    fn test_audit_trail_integrity() {
        // Verify hash chain is unbroken
        // Test tamper detection
    }
    
    #[test]
    fn test_consent_withdrawal() {
        // Verify data deletion on withdrawal
        // Check audit trail preserves withdrawal record
    }
}
```

### Experiment Testing
```rust
#[cfg(test)]
mod experiment_tests {
    #[test]
    fn test_balanced_randomization() {
        // Verify condition assignment is balanced
        // Test edge cases (odd numbers, dropouts)
    }
    
    #[test]
    fn test_session_scheduling() {
        // Verify sessions unlock on schedule
        // Test timezone handling
    }
}
```

---

## Conclusion

This design provides a clear path to implementing the compliance and experimental frameworks described in the core-abs IMPLICATIONS. The split between app and backend follows these principles:

1. **App handles**: User interaction, local data collection, offline queueing
2. **Backend handles**: Centralized compliance, randomization, statistical analysis
3. **Both share**: Audit responsibility, data integrity, state synchronization

The implementation can proceed incrementally, with Phase 1 providing immediate value for research use cases.