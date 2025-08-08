# Data Export and Analysis

The data export system captures comprehensive learning analytics for research, assessment, and system improvement. Data is exported in multiple formats for different analysis workflows.

## Export Architecture

```rust
pub struct DataExporter {
    pub format: ExportFormat,
    pub anonymizer: Option<DataAnonymizer>,
    pub compression: CompressionLevel,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    JSON,
    CSV,
    Parquet,
    SQLite,
    HDF5,
    Custom(Box<dyn Formatter>),
}

pub struct ExportSession {
    pub learner_data: LearnerDataExport,
    pub task_data: Vec<TaskRecord>,
    pub response_data: Vec<ResponseRecord>,
    pub metadata: SessionMetadata,
}
```

## Comprehensive Data Capture

### Learner Data Export

```rust
#[derive(Debug, Clone, Serialize)]
pub struct LearnerDataExport {
    pub learner_id: String,
    pub timestamp: DateTime<Utc>,
    
    // Knowledge state
    pub node_embeddings: HashMap<String, f64>,
    pub operation_proficiencies: HashMap<String, f64>,
    pub memory_strengths: HashMap<String, f64>,
    
    // Cognitive parameters
    pub learning_rate: f64,
    pub forgetting_rate: f64,
    pub response_time_params: ExGaussianParameters,
    
    // Strategy information
    pub detected_strategy: StrategyType,
    pub strategy_confidence: f64,
    
    // Performance metrics
    pub overall_accuracy: f64,
    pub average_response_time: f64,
    pub total_practice_time: Duration,
    pub mastery_level: f64,
}

impl LearnerDataExport {
    pub fn from_model(model: &LearnerModel) -> Self {
        LearnerDataExport {
            learner_id: model.learner_id.clone(),
            timestamp: Utc::now(),
            
            node_embeddings: model.node_embeddings.iter()
                .map(|(k, v)| (k.clone(), v.position))
                .collect(),
                
            operation_proficiencies: model.operation_proficiencies.iter()
                .map(|(k, v)| (k.clone(), sigmoid(v.theta)))
                .collect(),
                
            memory_strengths: model.memory_strengths.iter()
                .map(|(k, v)| (k.clone(), v.strength))
                .collect(),
                
            learning_rate: model.learning_rate,
            forgetting_rate: model.forgetting_rate,
            response_time_params: model.response_time_distribution.clone(),
            
            detected_strategy: model.strategy.clone(),
            strategy_confidence: model.strategy_confidence,
            
            overall_accuracy: model.calculate_overall_accuracy(),
            average_response_time: model.calculate_average_rt(),
            total_practice_time: model.total_practice_time,
            mastery_level: model.calculate_mastery(),
        }
    }
}
```

### Response-Level Data

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ResponseRecord {
    pub response_id: Uuid,
    pub learner_id: String,
    pub task_id: String,
    pub timestamp: DateTime<Utc>,
    
    // Task details
    pub task_type: String,
    pub task_difficulty: f64,
    pub correct_answer: String,
    pub user_answer: String,
    
    // Performance
    pub correct: bool,
    pub response_time_ms: u64,
    pub confidence: Option<f64>,
    
    // Context
    pub trial_number: usize,
    pub session_time_elapsed: Duration,
    pub hints_used: Vec<HintRecord>,
    
    // Cognitive state
    pub predicted_probability: f64,
    pub prediction_error: f64,
    pub information_gain: f64,
}
```

## Export Formats

### JSON Export

```rust
impl JsonExporter {
    pub fn export(&self, session: &ExportSession) -> Result<String, ExportError> {
        let json = serde_json::to_string_pretty(session)?;
        
        if self.compression != CompressionLevel::None {
            self.compress(json)
        } else {
            Ok(json)
        }
    }
    
    pub fn export_streaming<W: Write>(&self, writer: W, sessions: impl Iterator<Item = ExportSession>) -> Result<(), ExportError> {
        let mut writer = BufWriter::new(writer);
        
        writer.write_all(b"[")?;
        let mut first = true;
        
        for session in sessions {
            if !first {
                writer.write_all(b",")?;
            }
            
            let json = serde_json::to_string(&session)?;
            writer.write_all(json.as_bytes())?;
            first = false;
        }
        
        writer.write_all(b"]")?;
        writer.flush()?;
        
        Ok(())
    }
}
```

### CSV Export

```rust
impl CsvExporter {
    pub fn export_wide_format(&self, session: &ExportSession) -> Result<String, ExportError> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        
        // Write header
        wtr.write_record(&[
            "learner_id", "timestamp", "task_id", "task_type",
            "correct", "response_time", "difficulty", "strategy",
            "mastery", "trial_number"
        ])?;
        
        // Write records
        for response in &session.response_data {
            wtr.write_record(&[
                response.learner_id.clone(),
                response.timestamp.to_rfc3339(),
                response.task_id.clone(),
                response.task_type.clone(),
                response.correct.to_string(),
                response.response_time_ms.to_string(),
                response.task_difficulty.to_string(),
                session.learner_data.detected_strategy.to_string(),
                session.learner_data.mastery_level.to_string(),
                response.trial_number.to_string(),
            ])?;
        }
        
        String::from_utf8(wtr.into_inner()?)
            .map_err(|e| ExportError::Encoding(e.to_string()))
    }
    
    pub fn export_long_format(&self, session: &ExportSession) -> Result<String, ExportError> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        
        wtr.write_record(&["learner_id", "timestamp", "variable", "value"])?;
        
        // Export each measurement as a row
        let timestamp = Utc::now().to_rfc3339();
        
        for (node, strength) in &session.learner_data.memory_strengths {
            wtr.write_record(&[
                session.learner_data.learner_id.clone(),
                timestamp.clone(),
                format!("memory_{}", node),
                strength.to_string(),
            ])?;
        }
        
        // Continue for other variables...
        
        String::from_utf8(wtr.into_inner()?)
            .map_err(|e| ExportError::Encoding(e.to_string()))
    }
}
```

### Database Export

```rust
impl SqliteExporter {
    pub fn export(&self, session: &ExportSession, db_path: &Path) -> Result<(), ExportError> {
        let conn = Connection::open(db_path)?;
        
        // Create tables
        self.create_schema(&conn)?;
        
        // Insert learner data
        conn.execute(
            "INSERT INTO learners (id, learning_rate, strategy, mastery) VALUES (?1, ?2, ?3, ?4)",
            params![
                session.learner_data.learner_id,
                session.learner_data.learning_rate,
                session.learner_data.detected_strategy.to_string(),
                session.learner_data.mastery_level,
            ],
        )?;
        
        // Insert responses
        let mut stmt = conn.prepare(
            "INSERT INTO responses (learner_id, task_id, correct, response_time, timestamp) 
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )?;
        
        for response in &session.response_data {
            stmt.execute(params![
                response.learner_id,
                response.task_id,
                response.correct,
                response.response_time_ms,
                response.timestamp.timestamp(),
            ])?;
        }
        
        Ok(())
    }
    
    fn create_schema(&self, conn: &Connection) -> Result<(), ExportError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS learners (
                id TEXT PRIMARY KEY,
                learning_rate REAL,
                strategy TEXT,
                mastery REAL
            );
            
            CREATE TABLE IF NOT EXISTS responses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                learner_id TEXT,
                task_id TEXT,
                correct BOOLEAN,
                response_time INTEGER,
                timestamp INTEGER,
                FOREIGN KEY (learner_id) REFERENCES learners(id)
            );
            
            CREATE INDEX idx_responses_learner ON responses(learner_id);
            CREATE INDEX idx_responses_timestamp ON responses(timestamp);"
        )?;
        
        Ok(())
    }
}
```

## Population Analysis

### Aggregate Statistics

```rust
pub struct PopulationAnalyzer {
    pub sessions: Vec<ExportSession>,
}

impl PopulationAnalyzer {
    pub fn compute_population_statistics(&self) -> PopulationStats {
        let all_accuracies: Vec<f64> = self.sessions.iter()
            .map(|s| s.learner_data.overall_accuracy)
            .collect();
        
        let all_rts: Vec<f64> = self.sessions.iter()
            .flat_map(|s| s.response_data.iter().map(|r| r.response_time_ms as f64))
            .collect();
        
        let strategy_distribution = self.compute_strategy_distribution();
        
        PopulationStats {
            n_learners: self.sessions.len(),
            mean_accuracy: mean(&all_accuracies),
            sd_accuracy: std_dev(&all_accuracies),
            mean_rt: mean(&all_rts),
            sd_rt: std_dev(&all_rts),
            strategy_distribution,
            learning_curves: self.compute_population_learning_curves(),
        }
    }
    
    fn compute_population_learning_curves(&self) -> Vec<LearningCurve> {
        let mut curves = Vec::new();
        
        for session in &self.sessions {
            let performance_by_trial = self.compute_trial_performance(&session.response_data);
            
            curves.push(LearningCurve {
                learner_id: session.learner_data.learner_id.clone(),
                data_points: performance_by_trial,
                fitted_params: self.fit_learning_curve(&performance_by_trial),
            });
        }
        
        curves
    }
}
```

## Privacy and Anonymization

```rust
pub struct DataAnonymizer {
    pub anonymization_level: AnonymizationLevel,
    pub id_mapping: HashMap<String, String>,
}

impl DataAnonymizer {
    pub fn anonymize_session(&mut self, session: &mut ExportSession) {
        match self.anonymization_level {
            AnonymizationLevel::RemoveIdentifiers => {
                session.learner_data.learner_id = self.generate_anonymous_id();
            }
            
            AnonymizationLevel::KAnonymity(k) => {
                self.apply_k_anonymity(session, k);
            }
            
            AnonymizationLevel::DifferentialPrivacy { epsilon } => {
                self.apply_differential_privacy(session, epsilon);
            }
        }
        
        // Remove sensitive metadata
        session.metadata.remove_sensitive_fields();
    }
    
    fn apply_differential_privacy(&self, session: &mut ExportSession, epsilon: f64) {
        // Add Laplace noise to numeric values
        let sensitivity = 1.0; // Domain-specific
        let scale = sensitivity / epsilon;
        
        session.learner_data.overall_accuracy += sample_laplace(0.0, scale);
        session.learner_data.average_response_time += sample_laplace(0.0, scale * 1000.0);
        
        // Clip to valid ranges
        session.learner_data.overall_accuracy = session.learner_data.overall_accuracy.clamp(0.0, 1.0);
        session.learner_data.average_response_time = session.learner_data.average_response_time.max(0.0);
    }
}
```

## Real-Time Streaming

```rust
pub struct StreamingExporter {
    pub buffer_size: usize,
    pub flush_interval: Duration,
    pub destination: ExportDestination,
}

impl StreamingExporter {
    pub async fn stream_responses(&mut self, receiver: Receiver<ResponseRecord>) {
        let mut buffer = Vec::with_capacity(self.buffer_size);
        let mut last_flush = Instant::now();
        
        loop {
            tokio::select! {
                Some(response) = receiver.recv() => {
                    buffer.push(response);
                    
                    if buffer.len() >= self.buffer_size {
                        self.flush_buffer(&mut buffer).await;
                        last_flush = Instant::now();
                    }
                }
                
                _ = tokio::time::sleep(self.flush_interval) => {
                    if !buffer.is_empty() && last_flush.elapsed() > self.flush_interval {
                        self.flush_buffer(&mut buffer).await;
                        last_flush = Instant::now();
                    }
                }
            }
        }
    }
    
    async fn flush_buffer(&self, buffer: &mut Vec<ResponseRecord>) {
        match &self.destination {
            ExportDestination::File(path) => {
                self.append_to_file(path, buffer).await;
            }
            ExportDestination::Database(conn) => {
                self.batch_insert(conn, buffer).await;
            }
            ExportDestination::CloudStorage(client) => {
                self.upload_batch(client, buffer).await;
            }
        }
        
        buffer.clear();
    }
}
```

## Visualization Support

```rust
pub struct VisualizationExporter {
    pub plot_config: PlotConfig,
}

impl VisualizationExporter {
    pub fn export_for_plotting(&self, session: &ExportSession) -> PlotData {
        PlotData {
            learning_curve: self.prepare_learning_curve(session),
            rt_distribution: self.prepare_rt_distribution(session),
            accuracy_by_task_type: self.prepare_accuracy_breakdown(session),
            strategy_evolution: self.prepare_strategy_timeline(session),
            heatmap_data: self.prepare_confusion_matrix(session),
        }
    }
    
    fn prepare_learning_curve(&self, session: &ExportSession) -> Vec<(f64, f64)> {
        let window_size = 10;
        let mut curve = Vec::new();
        
        for (i, window) in session.response_data.windows(window_size).enumerate() {
            let accuracy = window.iter()
                .filter(|r| r.correct)
                .count() as f64 / window_size as f64;
            
            curve.push((i as f64, accuracy));
        }
        
        curve
    }
}
```

## Research Data Package

```rust
pub struct ResearchDataPackage {
    pub experiment_metadata: ExperimentMetadata,
    pub raw_data: Vec<ExportSession>,
    pub processed_data: ProcessedData,
    pub analysis_scripts: Vec<Script>,
    pub documentation: Documentation,
}

impl ResearchDataPackage {
    pub fn create_bids_format(&self) -> BIDSDataset {
        // Brain Imaging Data Structure adapted for behavioral data
        BIDSDataset {
            participants: self.create_participants_tsv(),
            sessions: self.create_sessions_structure(),
            derivatives: self.create_derivatives(),
            code: self.analysis_scripts.clone(),
            readme: self.documentation.readme.clone(),
        }
    }
    
    pub fn validate(&self) -> ValidationResult {
        let mut issues = Vec::new();
        
        // Check data completeness
        for session in &self.raw_data {
            if session.response_data.is_empty() {
                issues.push(ValidationIssue::EmptySession(session.learner_data.learner_id.clone()));
            }
        }
        
        // Check data quality
        let quality_metrics = self.compute_quality_metrics();
        if quality_metrics.missing_data_rate > 0.1 {
            issues.push(ValidationIssue::HighMissingData(quality_metrics.missing_data_rate));
        }
        
        ValidationResult {
            is_valid: issues.is_empty(),
            issues,
            quality_metrics,
        }
    }
}
```

## Summary

Data export and analysis provides:
- **Comprehensive capture**: All learning data and metadata
- **Multiple formats**: JSON, CSV, database, binary formats
- **Population analysis**: Aggregate statistics and patterns
- **Privacy protection**: Anonymization and differential privacy
- **Streaming export**: Real-time data pipeline
- **Visualization ready**: Prepared data for plotting
- **Research packages**: BIDS-like structure for sharing
- **Quality validation**: Data completeness and quality checks

This enables thorough analysis of learning data for research, assessment, and system improvement.