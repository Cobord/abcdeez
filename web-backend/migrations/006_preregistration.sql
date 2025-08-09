-- Pre-registration tables for scientific integrity

-- Main pre-registration table
CREATE TABLE IF NOT EXISTS preregistrations (
    id TEXT PRIMARY KEY,
    experiment_id TEXT NOT NULL,
    researcher_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    
    -- Registration metadata
    registered_at TIMESTAMP NOT NULL,
    registration_hash TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    
    -- JSON storage for complex structures
    study_metadata TEXT NOT NULL, -- JSON
    hypotheses TEXT NOT NULL, -- JSON
    analysis_plan TEXT NOT NULL, -- JSON
    data_collection_plan TEXT NOT NULL, -- JSON
    exclusion_criteria TEXT NOT NULL, -- JSON
    decision_rules TEXT NOT NULL, -- JSON
    
    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    parent_id TEXT, -- For amendments
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (experiment_id) REFERENCES experiments(id),
    FOREIGN KEY (parent_id) REFERENCES preregistrations(id)
);

-- Deviations from pre-registration
CREATE TABLE IF NOT EXISTS preregistration_deviations (
    id TEXT PRIMARY KEY,
    preregistration_id TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    description TEXT NOT NULL,
    justification TEXT NOT NULL,
    impact_assessment TEXT NOT NULL,
    created_by TEXT NOT NULL,
    
    FOREIGN KEY (preregistration_id) REFERENCES preregistrations(id)
);

-- Analysis validation log
CREATE TABLE IF NOT EXISTS analysis_validations (
    id TEXT PRIMARY KEY,
    preregistration_id TEXT NOT NULL,
    analysis_name TEXT NOT NULL,
    validation_result TEXT NOT NULL, -- 'valid', 'deviation', 'not_preregistered'
    actual_test TEXT NOT NULL,
    actual_variables TEXT NOT NULL, -- JSON array
    deviation_reason TEXT,
    is_exploratory BOOLEAN DEFAULT FALSE,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (preregistration_id) REFERENCES preregistrations(id)
);

-- Transparency reports
CREATE TABLE IF NOT EXISTS transparency_reports (
    id TEXT PRIMARY KEY,
    preregistration_id TEXT NOT NULL,
    generated_at TIMESTAMP NOT NULL,
    report_data TEXT NOT NULL, -- JSON
    public_url TEXT,
    
    FOREIGN KEY (preregistration_id) REFERENCES preregistrations(id)
);

-- Indexes for efficient queries
CREATE INDEX idx_preregistrations_experiment ON preregistrations(experiment_id);
CREATE INDEX idx_preregistrations_researcher ON preregistrations(researcher_id);
CREATE INDEX idx_preregistrations_status ON preregistrations(status);
CREATE INDEX idx_deviations_preregistration ON preregistration_deviations(preregistration_id);
CREATE INDEX idx_validations_preregistration ON analysis_validations(preregistration_id);

-- Trigger to update updated_at timestamp
CREATE TRIGGER update_preregistrations_timestamp 
AFTER UPDATE ON preregistrations
BEGIN
    UPDATE preregistrations SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;