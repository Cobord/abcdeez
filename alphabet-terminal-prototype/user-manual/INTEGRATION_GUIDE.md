# Research Platform Integration Guide

This guide shows how to integrate the comprehensive research features with the web backend and Xilem cross-platform UI for a complete research platform.

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Xilem UI     │    │  Web Backend    │    │ Research Core   │
│  (Cross-platform) │←→│   (Rust/API)    │←→│   (Engine)      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
    ┌────▼────┐             ┌────▼────┐             ┌────▼────┐
    │ iOS App │             │Database │             │Analytics│
    │Android  │             │Storage  │             │Export   │
    │Desktop  │             │APIs     │             │Compliance│
    └─────────┘             └─────────┘             └─────────┘
```

## Web Backend Integration

### API Endpoints for Research Features

Add these endpoints to your web backend (`web-backend/src/main.rs`):

```rust
// Research experiment management
#[get("/api/experiments")]
async fn list_experiments() -> Result<Json<Vec<MultiSessionExperiment>>, Status> {
    // Implementation
}

#[post("/api/experiments", data = "<experiment>")]
async fn create_experiment(experiment: Json<MultiSessionExperiment>) -> Result<Json<String>, Status> {
    // Implementation with validation
}

// IRB compliance documentation
#[post("/api/irb/generate", data = "<study_info>")]
async fn generate_irb_documents(study_info: Json<StudyInformation>) -> Result<Json<IRBCompliancePackage>, Status> {
    let generator = IRBComplianceGenerator::new(get_institution_settings());
    let package = generator.generate_compliance_package(study_info.into_inner(), &experiment, &design)?;
    Ok(Json(package))
}

// Data export endpoints
#[get("/api/export/<experiment_id>/<format>")]
async fn export_experiment_data(experiment_id: String, format: String) -> Result<NamedFile, Status> {
    let exporter = DataExporter::new();
    match format.as_str() {
        "r" => exporter.export_for_r(&path, true)?,
        "python" => exporter.export_for_python(&path, false)?,
        "spss" => exporter.export_for_spss(&path)?,
        _ => return Err(Status::BadRequest),
    }
    // Return generated files
}

// Real-time monitoring
#[get("/api/experiments/<id>/metrics")]
async fn get_real_time_metrics(id: String) -> Result<Json<ExperimentMetrics>, Status> {
    // Implementation
}
```

### Database Schema Extensions

Add these tables to support research features:

```sql
-- Experiments table
CREATE TABLE experiments (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    description TEXT,
    design_config JSONB NOT NULL,
    scheduling_rules JSONB NOT NULL,
    status VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Protocol versions table
CREATE TABLE protocol_versions (
    id UUID PRIMARY KEY,
    experiment_id UUID REFERENCES experiments(id),
    version VARCHAR NOT NULL,
    commit_hash VARCHAR NOT NULL,
    author_name VARCHAR NOT NULL,
    author_email VARCHAR NOT NULL,
    changes JSONB NOT NULL,
    snapshot JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- IRB compliance documents
CREATE TABLE irb_documents (
    id UUID PRIMARY KEY,
    experiment_id UUID REFERENCES experiments(id),
    document_type VARCHAR NOT NULL,
    content JSONB NOT NULL,
    generated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Real-time metrics
CREATE TABLE experiment_metrics (
    id UUID PRIMARY KEY,
    experiment_id UUID REFERENCES experiments(id),
    participant_count INTEGER,
    completion_rate DECIMAL,
    effect_size DECIMAL,
    statistical_power DECIMAL,
    measured_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### Research Service Implementation

Create a research service in `web-backend/src/research_service.rs`:

```rust
use crate::database::DbPool;
use alphabet_terminal_prototype::*;
use uuid::Uuid;

pub struct ResearchService {
    pool: DbPool,
    session_manager: MultiSessionManager,
    irb_generator: IRBComplianceGenerator,
    version_control: ProtocolVersionControl,
}

impl ResearchService {
    pub fn new(pool: DbPool) -> Self {
        let author = Author {
            name: "System".to_string(),
            email: "system@example.com".to_string(),
            institution: Some("Research Institution".to_string()),
        };

        Self {
            pool,
            session_manager: MultiSessionManager::new(PathBuf::from("./experiments")),
            irb_generator: IRBComplianceGenerator::new(get_institution_settings()),
            version_control: ProtocolVersionControl::new(author),
        }
    }

    pub async fn create_experiment(&mut self, experiment: MultiSessionExperiment) -> Result<String, String> {
        // Store in database
        let experiment_id = self.session_manager.create_experiment(
            experiment.name.clone(),
            experiment.description.clone(),
            experiment.design.clone(),
            experiment.sessions.clone(),
            experiment.scheduling_rules.clone(),
        )?;

        // Initialize protocol versioning
        let repo_id = self.version_control.init_repository(
            format!("Experiment: {}", experiment.name),
            experiment.description.clone(),
            PathBuf::from(format!("./protocols/{}", experiment_id)),
            create_repository_metadata(&experiment),
        )?;

        Ok(experiment_id)
    }

    pub async fn generate_compliance_docs(&self, experiment_id: &str, study_info: StudyInformation) -> Result<IRBCompliancePackage, String> {
        // Get experiment details
        let experiment = self.get_experiment(experiment_id).await?;
        
        // Generate compliance package
        self.irb_generator.generate_compliance_package(
            study_info,
            &experiment,
            &experiment.design,
        )
    }

    pub async fn export_data(&self, experiment_id: &str, format: &str) -> Result<Vec<PathBuf>, String> {
        let data = self.get_experiment_data(experiment_id).await?;
        let exporter = DataExporter::new();
        
        match format {
            "r" => exporter.export_for_r(&data, true),
            "python" => exporter.export_for_python(&data, false),
            "spss" => exporter.export_for_spss(&data),
            _ => Err("Unsupported format".to_string()),
        }
    }

    pub async fn get_real_time_metrics(&self, experiment_id: &str) -> Result<ExperimentMetrics, String> {
        // Calculate real-time metrics
        let data = self.get_experiment_data(experiment_id).await?;
        let analyzer = PowerAnalyzer::new();
        
        // Implement metrics calculation
        Ok(ExperimentMetrics {
            participant_count: data.participants.len(),
            completion_rate: calculate_completion_rate(&data),
            current_effect_size: analyzer.calculate_cohens_d(&data.group_a, &data.group_b)?,
            statistical_power: analyzer.calculate_achieved_power(&data)?,
            last_updated: chrono::Utc::now(),
        })
    }
}
```

## Xilem UI Integration

### Research Dashboard Components

Create research-focused UI components in `xilem-cross-platform/app/src/screens/research_dashboard.rs`:

```rust
use xilem::prelude::*;
use crate::research_state::ResearchState;

pub fn research_dashboard() -> impl View<ResearchState> {
    v_stack((
        // Header
        h_stack((
            text("Research Dashboard").font_size(24),
            spacer(),
            button("New Experiment", |state: &mut ResearchState| {
                state.show_experiment_wizard = true;
            }),
        )),
        
        // Main content tabs
        tab_view((
            ("Experiments", experiment_list_view()),
            ("Analytics", analytics_view()),
            ("Compliance", compliance_view()),
            ("Export", export_view()),
            ("Version Control", version_control_view()),
        )),
    ))
    .padding(16)
}

fn experiment_list_view() -> impl View<ResearchState> {
    list(|state: &ResearchState| &state.experiments)
        .map_item(|experiment| {
            h_stack((
                v_stack((
                    text(&experiment.name).font_weight(FontWeight::Bold),
                    text(&experiment.description).font_size(12).color(Color::GRAY),
                    text(&format!("Status: {:?}", experiment.status)).font_size(10),
                )),
                spacer(),
                button("Open", move |state: &mut ResearchState| {
                    state.current_experiment = Some(experiment.id.clone());
                }),
            ))
            .padding(8)
            .background(Color::WHITE)
            .border_radius(4)
        })
}

fn analytics_view() -> impl View<ResearchState> {
    v_stack((
        text("Real-time Analytics").font_size(20),
        
        // Metrics cards
        h_stack((
            metrics_card("Participants", |state| state.current_metrics.participant_count.to_string()),
            metrics_card("Completion Rate", |state| format!("{:.1}%", state.current_metrics.completion_rate * 100.0)),
            metrics_card("Effect Size", |state| format!("{:.3}", state.current_metrics.current_effect_size)),
            metrics_card("Statistical Power", |state| format!("{:.1}%", state.current_metrics.statistical_power * 100.0)),
        )),
        
        // Charts and visualizations
        power_curve_chart(),
        effect_size_timeline(),
    ))
}

fn compliance_view() -> impl View<ResearchState> {
    v_stack((
        text("IRB Compliance").font_size(20),
        
        button("Generate Consent Forms", |state: &mut ResearchState| {
            // Trigger IRB document generation
            state.generate_irb_documents();
        }),
        
        button("Export Protocol Summary", |state: &mut ResearchState| {
            // Export protocol documentation
            state.export_protocol_summary();
        }),
        
        // Compliance checklist
        compliance_checklist(),
    ))
}

fn export_view() -> impl View<ResearchState> {
    v_stack((
        text("Data Export").font_size(20),
        
        h_stack((
            button("Export for R", |state: &mut ResearchState| {
                state.export_data("r");
            }),
            button("Export for Python", |state: &mut ResearchState| {
                state.export_data("python");
            }),
            button("Export for SPSS", |state: &mut ResearchState| {
                state.export_data("spss");
            }),
        )),
        
        // Export options
        checkbox("Include analysis scripts", |state: &ResearchState| state.export_options.include_scripts)
            .on_change(|state: &mut ResearchState, checked| {
                state.export_options.include_scripts = checked;
            }),
        
        checkbox("Generate Jupyter notebooks", |state: &ResearchState| state.export_options.jupyter_notebooks)
            .on_change(|state: &mut ResearchState, checked| {
                state.export_options.jupyter_notebooks = checked;
            }),
    ))
}

fn version_control_view() -> impl View<ResearchState> {
    v_stack((
        text("Protocol Version Control").font_size(20),
        
        h_stack((
            text(&format!("Current Version: {}", state.current_version)),
            spacer(),
            button("New Version", |state: &mut ResearchState| {
                state.show_version_dialog = true;
            }),
        )),
        
        // Version history
        list(|state: &ResearchState| &state.version_history)
            .map_item(|version| {
                h_stack((
                    v_stack((
                        text(&version.version.to_string()).font_weight(FontWeight::Bold),
                        text(&version.message).font_size(12),
                        text(&format!("By: {} on {}", version.author.name, version.timestamp.format("%Y-%m-%d"))).font_size(10),
                    )),
                    spacer(),
                    button("Revert", |state: &mut ResearchState| {
                        state.revert_to_version(version.version.clone());
                    }),
                ))
            }),
    ))
}
```

### State Management

Create a research state manager in `xilem-cross-platform/app/src/research_state.rs`:

```rust
use alphabet_terminal_prototype::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ResearchState {
    pub experiments: Vec<MultiSessionExperiment>,
    pub current_experiment: Option<String>,
    pub current_metrics: ExperimentMetrics,
    pub version_history: Vec<ProtocolVersion>,
    pub current_version: SemanticVersion,
    pub export_options: ExportOptions,
    pub show_experiment_wizard: bool,
    pub show_version_dialog: bool,
}

#[derive(Debug, Clone)]
pub struct ExperimentMetrics {
    pub participant_count: usize,
    pub completion_rate: f64,
    pub current_effect_size: f64,
    pub statistical_power: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub include_scripts: bool,
    pub jupyter_notebooks: bool,
    pub format: String,
}

impl ResearchState {
    pub fn new() -> Self {
        Self {
            experiments: Vec::new(),
            current_experiment: None,
            current_metrics: ExperimentMetrics::default(),
            version_history: Vec::new(),
            current_version: SemanticVersion { major: 1, minor: 0, patch: 0, pre_release: None },
            export_options: ExportOptions {
                include_scripts: true,
                jupyter_notebooks: false,
                format: "r".to_string(),
            },
            show_experiment_wizard: false,
            show_version_dialog: false,
        }
    }

    pub fn generate_irb_documents(&mut self) {
        // Trigger IRB document generation via API
        let study_info = self.get_current_study_info();
        // Make API call to generate documents
    }

    pub fn export_data(&mut self, format: &str) {
        // Trigger data export via API
        if let Some(experiment_id) = &self.current_experiment {
            // Make API call to export data
        }
    }

    pub fn revert_to_version(&mut self, version: SemanticVersion) {
        // Revert protocol to specified version
    }
}
```

### Mobile-Specific Features

Add mobile optimizations in `xilem-cross-platform/app/src/mobile_research.rs`:

```rust
#[cfg(target_os = "ios")]
pub fn ios_research_integration() {
    use crate::ios_auth::IOSAuthBridge;
    
    // iOS-specific research features
    // - CloudKit integration for data sync
    // - HealthKit integration for physiological data
    // - Core Motion for movement tracking
}

#[cfg(target_os = "android")]
pub fn android_research_integration() {
    // Android-specific research features
    // - Google Drive integration
    // - Health Connect API
    // - Sensor data collection
}

// Cross-platform audio recording for think-aloud protocols
pub fn setup_audio_recording() -> AudioRecorder {
    AudioRecorder::new()
        .with_quality(AudioQuality::High)
        .with_format(AudioFormat::WAV)
        .with_sample_rate(44100)
}
```

## API Integration Examples

### Frontend API Calls

```typescript
// TypeScript/JavaScript API integration for web frontend

class ResearchAPI {
    private baseUrl = '/api';

    async createExperiment(experiment: MultiSessionExperiment): Promise<string> {
        const response = await fetch(`${this.baseUrl}/experiments`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(experiment)
        });
        return response.json();
    }

    async generateIRBDocuments(studyInfo: StudyInformation): Promise<IRBCompliancePackage> {
        const response = await fetch(`${this.baseUrl}/irb/generate`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(studyInfo)
        });
        return response.json();
    }

    async exportData(experimentId: string, format: string): Promise<Blob> {
        const response = await fetch(`${this.baseUrl}/export/${experimentId}/${format}`);
        return response.blob();
    }

    async getRealTimeMetrics(experimentId: string): Promise<ExperimentMetrics> {
        const response = await fetch(`${this.baseUrl}/experiments/${experimentId}/metrics`);
        return response.json();
    }
}
```

### Real-Time Updates

```rust
// WebSocket integration for real-time updates
use rocket_ws::{WebSocket, Stream};

#[get("/ws/experiments/<id>")]
fn experiment_websocket(id: String, ws: WebSocket) -> Stream!['static] {
    Stream! { ws =>
        for await message in ws {
            match message? {
                Message::Text(text) => {
                    // Handle real-time metric requests
                    let metrics = get_current_metrics(&id).await?;
                    yield Message::Text(serde_json::to_string(&metrics)?);
                }
                Message::Binary(_) => {
                    // Handle binary data (e.g., sensor data)
                }
                Message::Close(_) => break,
            }
        }
    }
}
```

## Configuration and Deployment

### Environment Configuration

Create `config/research.toml`:

```toml
[research]
data_directory = "./research_data"
export_directory = "./exports"
protocol_repository = "./protocols"

[irb]
institution_name = "University Name"
irb_contact = "irb@university.edu"
compliance_level = "full"

[export]
default_format = "r"
include_scripts = true
generate_notebooks = false

[sensors]
enabled = ["audio", "keyboard", "mouse"]
mock_mode = false

[version_control]
auto_commit = true
require_message = true
```

### Docker Integration

Update `Dockerfile` to include research features:

```dockerfile
FROM rust:1.70 as builder

# Copy research dependencies
COPY ./user-manual ./user-manual
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml

# Build with research features
RUN cargo build --release --features "research,irb,sensors"

FROM debian:bullseye-slim

# Install R and Python for export functionality
RUN apt-get update && apt-get install -y \
    r-base \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Python packages for export
RUN pip3 install pandas numpy scipy matplotlib seaborn jupyter statsmodels

COPY --from=builder /app/target/release/alphabet-terminal-prototype /usr/local/bin/
COPY ./config /app/config

EXPOSE 8000
CMD ["alphabet-terminal-prototype", "--config", "/app/config/research.toml"]
```

## Testing Integration

### Unit Tests for Research Features

```rust
#[cfg(test)]
mod research_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_experiment_creation_flow() {
        let mut service = ResearchService::new(test_db_pool()).await;
        
        let experiment = create_test_experiment();
        let experiment_id = service.create_experiment(experiment).await.unwrap();
        
        assert!(!experiment_id.is_empty());
        
        // Test IRB document generation
        let study_info = create_test_study_info();
        let compliance_docs = service.generate_compliance_docs(&experiment_id, study_info).await.unwrap();
        assert!(!compliance_docs.consent_form.study_purpose.is_empty());
        
        // Test data export
        let export_files = service.export_data(&experiment_id, "r").await.unwrap();
        assert!(!export_files.is_empty());
    }

    #[test]
    fn test_version_control_integration() {
        let author = create_test_author();
        let mut vc = ProtocolVersionControl::new(author);
        
        let repo_id = vc.init_repository(
            "Test Protocol".to_string(),
            "Test description".to_string(),
            PathBuf::from("/tmp/test"),
            create_test_metadata(),
        ).unwrap();
        
        // Test commit creation
        let version = vc.commit(
            "Test commit".to_string(),
            vec![create_test_change()],
            create_test_experiment(),
            create_test_design(),
            HashMap::new(),
        ).unwrap();
        
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 1);
    }
}
```

## Performance Considerations

### Database Optimization

- Index frequently queried fields (experiment_id, participant_id)
- Use connection pooling for concurrent access
- Consider partitioning for large datasets
- Implement caching for read-heavy operations

### Real-Time Features

- Use WebSockets for live updates
- Implement efficient delta updates
- Cache computed metrics
- Consider message queues for heavy processing

### Mobile Performance

- Implement efficient data synchronization
- Use local storage for offline capability  
- Optimize sensor data collection
- Implement background processing

This integration guide provides a comprehensive framework for connecting all the research features with your web backend and cross-platform UI, creating a complete research platform suitable for cognitive and learning science studies.