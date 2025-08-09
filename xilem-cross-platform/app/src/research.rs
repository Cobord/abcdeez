use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;
use crate::audio_recorder::{AudioRecorder, AudioConfig, create_audio_file_path};
use graph_learning_core::{
    IRBComplianceGenerator, StudySummary, ConsentTemplate, IRBApplication,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSession {
    pub session_id: String,
    pub participant_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub transcription: Option<String>,
    pub quality_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioRecordingState {
    Idle,
    Recording { start_time: DateTime<Utc>, duration: Duration },
    Paused { total_duration: Duration },
    Processing,
    Completed { file_path: String },
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub sensor_type: SensorType,
    pub enabled: bool,
    pub sample_rate: u32,
    pub status: SensorStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorType {
    EEG,
    GSR,
    EyeTracker,
    HeartRate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SensorStatus {
    Disconnected,
    Connected,
    Recording,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRBApplicationStatus {
    pub application_id: String,
    pub study_title: String,
    pub status: IRBStatus,
    pub submitted_date: Option<DateTime<Utc>>,
    pub approval_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub reviewer_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IRBStatus {
    Draft,
    Submitted,
    UnderReview,
    Approved,
    ConditionalApproval,
    Rejected,
    Expired,
    Withdrawn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRBDocument {
    pub document_id: String,
    pub document_type: IRBDocumentType,
    pub title: String,
    pub content: String,
    pub generated_date: DateTime<Utc>,
    pub file_path: Option<String>,
    pub status: DocumentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IRBDocumentType {
    Application,
    ConsentForm,
    StudyProtocol,
    RiskAssessment,
    DataManagementPlan,
    RecruitmentMaterial,
    Debriefing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentStatus {
    Draft,
    Generated,
    Reviewed,
    Approved,
    NeedsRevision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSession {
    pub id: String,
    pub participant_id: String,
    pub experiment_type: ExperimentType,
    pub condition: ExperimentCondition,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub data_points: Vec<DataPoint>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentType {
    LearningCurve,
    RetentionTest,
    InterferenceStudy,
    AdaptiveScheduling,
    CognitiveLoad,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentCondition {
    pub name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub control_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub trial_number: u32,
    pub stimulus: String,
    pub response: String,
    pub correct: bool,
    pub response_time_ms: u128,
    pub confidence: Option<f64>,
    pub eye_tracking: Option<EyeTrackingData>,
    pub physiological: Option<PhysiologicalData>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EyeTrackingData {
    pub fixation_count: u32,
    pub total_fixation_duration_ms: u128,
    pub saccade_count: u32,
    pub pupil_diameter: Option<f64>,
    pub gaze_path: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologicalData {
    pub heart_rate: Option<f64>,
    pub heart_rate_variability: Option<f64>,
    pub skin_conductance: Option<f64>,
    pub eeg_data: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchMetrics {
    pub learning_rate: f64,
    pub retention_score: f64,
    pub interference_index: f64,
    pub cognitive_load_estimate: f64,
    pub performance_consistency: f64,
    pub adaptation_effectiveness: f64,
}

#[derive(Debug)]
pub struct ResearchController {
    pub active_session: Option<ResearchSession>,
    pub sessions: Vec<ResearchSession>,
    pub participant_id: String,
    pub current_experiment: Option<ExperimentType>,
    pub data_collection_enabled: bool,
    pub privacy_mode: bool,
    // Audio recording for think-aloud protocols
    pub audio_recording_enabled: bool,
    pub current_audio_session: Option<AudioSession>,
    pub recording_state: AudioRecordingState,
    pub audio_recorder: Option<AudioRecorder>,
    pub audio_config: AudioConfig,
    // Sensor integration
    pub sensor_recording_enabled: bool,
    pub connected_sensors: Vec<SensorConfig>,
    // IRB compliance
    pub irb_generator: Option<IRBComplianceGenerator>,
    pub pending_irb_applications: Vec<IRBApplicationStatus>,
    pub generated_documents: Vec<IRBDocument>,
}

impl Clone for ResearchController {
    fn clone(&self) -> Self {
        Self {
            active_session: self.active_session.clone(),
            sessions: self.sessions.clone(),
            participant_id: self.participant_id.clone(),
            current_experiment: self.current_experiment.clone(),
            data_collection_enabled: self.data_collection_enabled,
            privacy_mode: self.privacy_mode,
            audio_recording_enabled: self.audio_recording_enabled,
            current_audio_session: self.current_audio_session.clone(),
            recording_state: self.recording_state.clone(),
            audio_recorder: None, // Don't clone the audio recorder
            audio_config: self.audio_config.clone(),
            sensor_recording_enabled: self.sensor_recording_enabled,
            connected_sensors: self.connected_sensors.clone(),
            irb_generator: self.irb_generator.clone(),
            pending_irb_applications: self.pending_irb_applications.clone(),
            generated_documents: self.generated_documents.clone(),
        }
    }
}

impl ResearchController {
    pub fn new(participant_id: String) -> Self {
        Self {
            active_session: None,
            sessions: Vec::new(),
            participant_id,
            current_experiment: None,
            data_collection_enabled: true,
            privacy_mode: false,
            audio_recording_enabled: false,
            current_audio_session: None,
            recording_state: AudioRecordingState::Idle,
            audio_recorder: None,
            audio_config: AudioConfig::default(),
            sensor_recording_enabled: false,
            connected_sensors: vec![
                SensorConfig {
                    sensor_type: SensorType::EEG,
                    enabled: false,
                    sample_rate: 250,
                    status: SensorStatus::Disconnected,
                },
                SensorConfig {
                    sensor_type: SensorType::GSR,
                    enabled: false,
                    sample_rate: 100,
                    status: SensorStatus::Disconnected,
                },
                SensorConfig {
                    sensor_type: SensorType::EyeTracker,
                    enabled: false,
                    sample_rate: 60,
                    status: SensorStatus::Disconnected,
                },
                SensorConfig {
                    sensor_type: SensorType::HeartRate,
                    enabled: false,
                    sample_rate: 1,
                    status: SensorStatus::Disconnected,
                },
            ],
            irb_generator: None,
            pending_irb_applications: Vec::new(),
            generated_documents: Vec::new(),
        }
    }

    pub fn start_experiment(
        &mut self,
        experiment_type: ExperimentType,
        condition: ExperimentCondition,
    ) -> Result<String, String> {
        if self.active_session.is_some() {
            return Err("An experiment is already in progress".to_string());
        }

        let session = ResearchSession {
            id: Uuid::new_v4().to_string(),
            participant_id: self.participant_id.clone(),
            experiment_type: experiment_type.clone(),
            condition,
            start_time: Utc::now(),
            end_time: None,
            data_points: Vec::new(),
            metadata: HashMap::new(),
        };

        let session_id = session.id.clone();
        self.active_session = Some(session);
        self.current_experiment = Some(experiment_type);
        
        Ok(session_id)
    }

    pub fn record_data_point(&mut self, data_point: DataPoint) -> Result<(), String> {
        if let Some(session) = &mut self.active_session {
            if self.data_collection_enabled && !self.privacy_mode {
                session.data_points.push(data_point);
                Ok(())
            } else {
                Err("Data collection is disabled or privacy mode is on".to_string())
            }
        } else {
            Err("No active research session".to_string())
        }
    }

    pub fn end_experiment(&mut self) -> Result<ResearchSession, String> {
        if let Some(mut session) = self.active_session.take() {
            session.end_time = Some(Utc::now());
            self.sessions.push(session.clone());
            self.current_experiment = None;
            Ok(session)
        } else {
            Err("No active experiment to end".to_string())
        }
    }

    pub fn calculate_metrics(&self, session: &ResearchSession) -> ResearchMetrics {
        let data_points = &session.data_points;
        
        if data_points.is_empty() {
            return ResearchMetrics {
                learning_rate: 0.0,
                retention_score: 0.0,
                interference_index: 0.0,
                cognitive_load_estimate: 0.0,
                performance_consistency: 0.0,
                adaptation_effectiveness: 0.0,
            };
        }

        let total_trials = data_points.len() as f64;
        let correct_trials = data_points.iter().filter(|dp| dp.correct).count() as f64;
        let accuracy = correct_trials / total_trials;

        let window_size = 5;
        let mut learning_curve = Vec::new();
        for window in data_points.windows(window_size) {
            let window_correct = window.iter().filter(|dp| dp.correct).count() as f64;
            learning_curve.push(window_correct / window_size as f64);
        }

        let learning_rate = if learning_curve.len() > 1 {
            let first_half = &learning_curve[..learning_curve.len()/2];
            let second_half = &learning_curve[learning_curve.len()/2..];
            let first_avg: f64 = first_half.iter().sum::<f64>() / first_half.len() as f64;
            let second_avg: f64 = second_half.iter().sum::<f64>() / second_half.len() as f64;
            (second_avg - first_avg).max(0.0)
        } else {
            0.0
        };

        let response_times: Vec<f64> = data_points
            .iter()
            .map(|dp| dp.response_time_ms as f64)
            .collect();
        
        let mean_rt = response_times.iter().sum::<f64>() / response_times.len() as f64;
        let variance = response_times
            .iter()
            .map(|rt| (rt - mean_rt).powi(2))
            .sum::<f64>() / response_times.len() as f64;
        let std_dev = variance.sqrt();
        let consistency = 1.0 - (std_dev / mean_rt).min(1.0);

        let cognitive_load = (mean_rt / 1000.0).min(10.0) / 10.0;

        ResearchMetrics {
            learning_rate,
            retention_score: accuracy,
            interference_index: 0.0,
            cognitive_load_estimate: cognitive_load,
            performance_consistency: consistency,
            adaptation_effectiveness: learning_rate * consistency,
        }
    }

    pub fn export_research_data(&self, format: ExportFormat) -> Result<String, String> {
        match format {
            ExportFormat::Json => {
                let data = serde_json::json!({
                    "participant_id": self.participant_id,
                    "sessions": self.sessions,
                    "total_sessions": self.sessions.len(),
                    "total_data_points": self.sessions.iter()
                        .map(|s| s.data_points.len())
                        .sum::<usize>(),
                });
                serde_json::to_string_pretty(&data)
                    .map_err(|e| format!("Failed to serialize: {}", e))
            }
            ExportFormat::Csv => {
                let mut csv_lines = vec![
                    "session_id,experiment_type,trial,stimulus,response,correct,rt_ms,timestamp".to_string()
                ];
                
                for session in &self.sessions {
                    for (i, dp) in session.data_points.iter().enumerate() {
                        csv_lines.push(format!(
                            "{},{},{},{},{},{},{},{}",
                            session.id,
                            format!("{:?}", session.experiment_type),
                            i + 1,
                            dp.stimulus,
                            dp.response,
                            dp.correct,
                            dp.response_time_ms,
                            dp.timestamp.to_rfc3339()
                        ));
                    }
                }
                
                Ok(csv_lines.join("\n"))
            }
            _ => Err("Unsupported export format".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Excel,
    Matlab,
    Python,
}

pub struct ResearchAnalyzer {
    sessions: Vec<ResearchSession>,
}

impl ResearchAnalyzer {
    pub fn new(sessions: Vec<ResearchSession>) -> Self {
        Self { sessions }
    }

    pub fn analyze_learning_curves(&self) -> HashMap<String, Vec<f64>> {
        let mut curves = HashMap::new();
        
        for session in &self.sessions {
            let window_size = 10;
            let mut session_curve = Vec::new();
            
            for window in session.data_points.windows(window_size) {
                let accuracy = window.iter().filter(|dp| dp.correct).count() as f64 
                    / window_size as f64;
                session_curve.push(accuracy);
            }
            
            curves.insert(session.id.clone(), session_curve);
        }
        
        curves
    }

    pub fn analyze_retention(&self, delay_minutes: u64) -> f64 {
        let mut retention_scores = Vec::new();
        
        for session in &self.sessions {
            if let Some(end_time) = session.end_time {
                let delayed_points: Vec<&DataPoint> = session.data_points
                    .iter()
                    .filter(|dp| {
                        let diff = dp.timestamp.signed_duration_since(end_time);
                        diff.num_minutes() >= delay_minutes as i64
                    })
                    .collect();
                
                if !delayed_points.is_empty() {
                    let accuracy = delayed_points.iter()
                        .filter(|dp| dp.correct)
                        .count() as f64 / delayed_points.len() as f64;
                    retention_scores.push(accuracy);
                }
            }
        }
        
        if retention_scores.is_empty() {
            0.0
        } else {
            retention_scores.iter().sum::<f64>() / retention_scores.len() as f64
        }
    }

    pub fn compare_conditions(&self) -> HashMap<String, ResearchMetrics> {
        let mut condition_metrics = HashMap::new();
        let mut condition_sessions: HashMap<String, Vec<&ResearchSession>> = HashMap::new();
        
        for session in &self.sessions {
            condition_sessions
                .entry(session.condition.name.clone())
                .or_insert_with(Vec::new)
                .push(session);
        }
        
        for (condition_name, sessions) in condition_sessions {
            let mut all_metrics = Vec::new();
            
            for session in sessions {
                let controller = ResearchController::new(session.participant_id.clone());
                all_metrics.push(controller.calculate_metrics(session));
            }
            
            if !all_metrics.is_empty() {
                let avg_metrics = ResearchMetrics {
                    learning_rate: all_metrics.iter().map(|m| m.learning_rate).sum::<f64>() 
                        / all_metrics.len() as f64,
                    retention_score: all_metrics.iter().map(|m| m.retention_score).sum::<f64>() 
                        / all_metrics.len() as f64,
                    interference_index: all_metrics.iter().map(|m| m.interference_index).sum::<f64>() 
                        / all_metrics.len() as f64,
                    cognitive_load_estimate: all_metrics.iter().map(|m| m.cognitive_load_estimate).sum::<f64>() 
                        / all_metrics.len() as f64,
                    performance_consistency: all_metrics.iter().map(|m| m.performance_consistency).sum::<f64>() 
                        / all_metrics.len() as f64,
                    adaptation_effectiveness: all_metrics.iter().map(|m| m.adaptation_effectiveness).sum::<f64>() 
                        / all_metrics.len() as f64,
                };
                
                condition_metrics.insert(condition_name, avg_metrics);
            }
        }
        
        condition_metrics
    }

    // Audio recording methods
    pub fn start_audio_recording(&mut self) -> Result<String, String> {
        if !self.audio_recording_enabled {
            return Err("Audio recording is not enabled".to_string());
        }

        if matches!(self.recording_state, AudioRecordingState::Recording { .. }) {
            return Err("Audio recording is already in progress".to_string());
        }

        // Initialize audio recorder if needed
        if self.audio_recorder.is_none() {
            match AudioRecorder::new() {
                Ok(mut recorder) => {
                    if let Err(e) = recorder.initialize(self.audio_config.clone()) {
                        return Err(format!("Failed to initialize audio recorder: {}", e));
                    }
                    self.audio_recorder = Some(recorder);
                }
                Err(e) => {
                    return Err(format!("Failed to create audio recorder: {}", e));
                }
            }
        }

        let session_id = Uuid::new_v4().to_string();
        let file_path = create_audio_file_path(&self.participant_id, &session_id);
        
        // Start recording
        if let Some(ref mut recorder) = self.audio_recorder {
            if let Err(e) = recorder.start_recording(file_path.clone()) {
                return Err(format!("Failed to start recording: {}", e));
            }
        }

        let audio_session = AudioSession {
            session_id: session_id.clone(),
            participant_id: self.participant_id.clone(),
            start_time: Utc::now(),
            end_time: None,
            file_path: Some(file_path.to_string_lossy().to_string()),
            transcription: None,
            quality_score: None,
        };

        self.current_audio_session = Some(audio_session);
        self.recording_state = AudioRecordingState::Recording {
            start_time: Utc::now(),
            duration: Duration::from_secs(0),
        };

        Ok(session_id)
    }

    pub fn stop_audio_recording(&mut self) -> Result<AudioSession, String> {
        if !matches!(self.recording_state, AudioRecordingState::Recording { .. }) {
            return Err("No recording in progress".to_string());
        }

        // Stop the actual recording
        if let Some(ref mut recorder) = self.audio_recorder {
            if let Err(e) = recorder.stop_recording() {
                return Err(format!("Failed to stop recording: {}", e));
            }
        }

        if let Some(mut session) = self.current_audio_session.take() {
            session.end_time = Some(Utc::now());
            
            let file_path = session.file_path.clone().unwrap_or_else(|| {
                create_audio_file_path(&self.participant_id, &session.session_id)
                    .to_string_lossy().to_string()
            });
            
            self.recording_state = AudioRecordingState::Completed { 
                file_path: file_path.clone() 
            };
            
            // Calculate a basic quality score based on duration
            let duration = session.end_time.unwrap()
                .signed_duration_since(session.start_time)
                .num_seconds() as f64;
            
            session.quality_score = Some(if duration > 5.0 { 0.9 } else { 0.7 });
            
            Ok(session)
        } else {
            Err("No active audio recording session".to_string())
        }
    }

    pub fn toggle_audio_recording(&mut self) -> Result<String, String> {
        match &self.recording_state {
            AudioRecordingState::Idle => self.start_audio_recording(),
            AudioRecordingState::Recording { .. } => {
                self.stop_audio_recording().map(|session| format!("Recording stopped: {}", session.session_id))
            }
            _ => Err("Cannot toggle recording in current state".to_string()),
        }
    }

    // Sensor management methods
    pub fn toggle_sensor(&mut self, sensor_type: &SensorType) -> Result<bool, String> {
        if let Some(sensor) = self.connected_sensors.iter_mut().find(|s| s.sensor_type == *sensor_type) {
            sensor.enabled = !sensor.enabled;
            sensor.status = if sensor.enabled {
                SensorStatus::Connected
            } else {
                SensorStatus::Disconnected
            };
            Ok(sensor.enabled)
        } else {
            Err(format!("Sensor {:?} not found", sensor_type))
        }
    }

    pub fn get_recording_duration(&self) -> Duration {
        match &self.recording_state {
            AudioRecordingState::Recording { start_time, .. } => {
                Utc::now().signed_duration_since(*start_time).to_std().unwrap_or_default()
            }
            AudioRecordingState::Paused { total_duration } => *total_duration,
            _ => Duration::from_secs(0),
        }
    }

    pub fn get_available_audio_devices(&self) -> Vec<String> {
        if let Some(ref recorder) = self.audio_recorder {
            recorder.get_available_devices()
        } else {
            match AudioRecorder::new() {
                Ok(recorder) => recorder.get_available_devices(),
                Err(_) => vec!["No audio devices available".to_string()],
            }
        }
    }

    pub fn get_supported_audio_configs(&self) -> Vec<(u32, u16)> {
        if let Some(ref recorder) = self.audio_recorder {
            recorder.get_supported_configs().unwrap_or_default()
        } else {
            vec![(44100, 1), (48000, 1), (22050, 1)] // Default common configs
        }
    }

    pub fn update_audio_config(&mut self, sample_rate: u32, channels: u16) -> Result<(), String> {
        self.audio_config.sample_rate = sample_rate;
        self.audio_config.channels = channels;
        
        // If recorder exists, reinitialize it with new config
        if let Some(ref mut recorder) = self.audio_recorder {
            recorder.initialize(self.audio_config.clone())
                .map_err(|e| format!("Failed to update audio config: {}", e))?;
        }
        
        Ok(())
    }

    // IRB Compliance Methods
    pub fn initialize_irb_generator(&mut self) -> Result<(), String> {
        if self.irb_generator.is_none() {
            self.irb_generator = Some(IRBComplianceGenerator::default());
        }
        Ok(())
    }

    pub fn create_irb_application(
        &mut self,
        study_title: String,
        principal_investigator: String,
        institution: String,
        study_purpose: String,
        participant_population: String,
        data_collection_methods: Vec<String>,
    ) -> Result<String, String> {
        self.initialize_irb_generator()?;

        let application_id = Uuid::new_v4().to_string();
        
        // Create the IRB application status
        let application_status = IRBApplicationStatus {
            application_id: application_id.clone(),
            study_title: study_title.clone(),
            status: IRBStatus::Draft,
            submitted_date: None,
            approval_date: None,
            expiration_date: None,
            reviewer_notes: Vec::new(),
        };

        self.pending_irb_applications.push(application_status);

        // Generate the application document
        if let Some(ref mut generator) = self.irb_generator {
            let study_summary = StudySummary {
                title: study_title.clone(),
                principal_investigator,
                institution,
                purpose: study_purpose,
                participant_population,
                data_collection_methods,
                estimated_participants: 100, // Default, can be configured
                study_duration_months: 12,   // Default, can be configured
                risk_level: "Minimal".to_string(),
            };

            // This would call the actual IRB generation method from the core library
            // For now, we'll create a basic document
            let document = IRBDocument {
                document_id: Uuid::new_v4().to_string(),
                document_type: IRBDocumentType::Application,
                title: format!("IRB Application: {}", study_title),
                content: format!(
                    "IRB APPLICATION\n\nStudy Title: {}\nPrincipal Investigator: {}\nInstitution: {}\nPurpose: {}",
                    study_summary.title, study_summary.principal_investigator, 
                    study_summary.institution, study_summary.purpose
                ),
                generated_date: Utc::now(),
                file_path: None,
                status: DocumentStatus::Draft,
            };

            self.generated_documents.push(document);
        }

        Ok(application_id)
    }

    pub fn generate_consent_form(
        &mut self,
        study_title: String,
        risks: Vec<String>,
        benefits: Vec<String>,
        procedures: Vec<String>,
    ) -> Result<String, String> {
        self.initialize_irb_generator()?;

        let document_id = Uuid::new_v4().to_string();
        
        let content = format!(
            "INFORMED CONSENT FORM\n\nStudy Title: {}\n\nYou are being invited to participate in a research study.\n\nPURPOSE:\nThis study aims to understand learning processes and cognitive performance.\n\nPROCEDURES:\n{}\n\nRISKS:\n{}\n\nBENEFITS:\n{}\n\nCONFIDENTIALITY:\nYour identity and data will be kept confidential. All data will be anonymized and stored securely.\n\nVOLUNTARY PARTICIPATION:\nYour participation is voluntary. You may withdraw at any time without penalty.\n\nCONTACT INFORMATION:\nIf you have questions, please contact the research team.\n\nI have read and understood the information provided. I agree to participate in this study.\n\nParticipant Signature: _________________ Date: _________\n\nResearcher Signature: _________________ Date: _________",
            study_title,
            procedures.join("\n• "),
            if risks.is_empty() { "This study involves minimal risk".to_string() } else { risks.join("\n• ") },
            benefits.join("\n• ")
        );

        let document = IRBDocument {
            document_id: document_id.clone(),
            document_type: IRBDocumentType::ConsentForm,
            title: format!("Consent Form: {}", study_title),
            content,
            generated_date: Utc::now(),
            file_path: None,
            status: DocumentStatus::Generated,
        };

        self.generated_documents.push(document);
        Ok(document_id)
    }

    pub fn generate_data_management_plan(&mut self, study_title: String) -> Result<String, String> {
        let document_id = Uuid::new_v4().to_string();
        
        let content = format!(
            "DATA MANAGEMENT PLAN\n\nStudy: {}\n\nDATA COLLECTION:\n• Audio recordings (if enabled) stored locally with encryption\n• Response time data collected during tasks\n• Physiological sensor data (if enabled)\n• All data anonymized with participant IDs\n\nDATA STORAGE:\n• Local encrypted storage during collection\n• Secure cloud backup with institutional approval\n• Data retention for 7 years as per research standards\n\nDATA SECURITY:\n• AES-256 encryption for all stored data\n• Secure transmission protocols (HTTPS/TLS)\n• Access controls with authentication\n• Regular security audits\n\nDATA SHARING:\n• Anonymized data may be shared for research purposes\n• Participants can request data deletion\n• Compliance with GDPR and local privacy laws\n\nDATA DESTRUCTION:\n• Automatic deletion after retention period\n• Secure deletion protocols for sensitive data\n• Audit trail of all data access and modifications",
            study_title
        );

        let document = IRBDocument {
            document_id: document_id.clone(),
            document_type: IRBDocumentType::DataManagementPlan,
            title: format!("Data Management Plan: {}", study_title),
            content,
            generated_date: Utc::now(),
            file_path: None,
            status: DocumentStatus::Generated,
        };

        self.generated_documents.push(document);
        Ok(document_id)
    }

    pub fn get_irb_applications(&self) -> &Vec<IRBApplicationStatus> {
        &self.pending_irb_applications
    }

    pub fn get_generated_documents(&self) -> &Vec<IRBDocument> {
        &self.generated_documents
    }

    pub fn update_application_status(&mut self, application_id: &str, status: IRBStatus) -> Result<(), String> {
        if let Some(application) = self.pending_irb_applications.iter_mut()
            .find(|app| app.application_id == application_id) {
            application.status = status.clone();
            
            match status {
                IRBStatus::Submitted => {
                    application.submitted_date = Some(Utc::now());
                }
                IRBStatus::Approved => {
                    application.approval_date = Some(Utc::now());
                    application.expiration_date = Some(Utc::now() + chrono::Duration::days(365));
                }
                _ => {}
            }
            
            Ok(())
        } else {
            Err("Application not found".to_string())
        }
    }

    pub fn export_irb_document(&mut self, document_id: &str) -> Result<String, String> {
        if let Some(document) = self.generated_documents.iter_mut()
            .find(|doc| doc.document_id == document_id) {
            
            let filename = format!("{}_{}.txt", 
                document.title.replace(" ", "_").replace(":", ""),
                document.document_id[..8].to_string()
            );
            
            // Create IRB documents directory
            std::fs::create_dir_all("irb_documents").map_err(|e| e.to_string())?;
            let file_path = format!("irb_documents/{}", filename);
            
            std::fs::write(&file_path, &document.content).map_err(|e| e.to_string())?;
            
            document.file_path = Some(file_path.clone());
            document.status = DocumentStatus::Approved;
            
            Ok(file_path)
        } else {
            Err("Document not found".to_string())
        }
    }
}