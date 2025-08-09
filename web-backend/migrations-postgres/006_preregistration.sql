-- PostgreSQL: Pre-registration tables for scientific integrity

-- Main pre-registration table
CREATE TABLE IF NOT EXISTS preregistrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    experiment_id UUID NOT NULL,
    researcher_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,

    -- Registration metadata
    registered_at TIMESTAMP NOT NULL,
    registration_hash TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',

    -- JSON storage for complex structures
    study_metadata TEXT NOT NULL,
    hypotheses TEXT NOT NULL,
    analysis_plan TEXT NOT NULL,
    data_collection_plan TEXT NOT NULL,
    exclusion_criteria TEXT NOT NULL,
    decision_rules TEXT NOT NULL,

    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    parent_id UUID,

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (experiment_id) REFERENCES experiments(id),
    FOREIGN KEY (parent_id) REFERENCES preregistrations(id)
);

-- Deviations from pre-registration
CREATE TABLE IF NOT EXISTS preregistration_deviations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    preregistration_id UUID NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    description TEXT NOT NULL,
    justification TEXT NOT NULL,
    impact_assessment TEXT NOT NULL,
    created_by TEXT NOT NULL,

    FOREIGN KEY (preregistration_id) REFERENCES preregistrations(id)
);

-- Analysis validation log
CREATE TABLE IF NOT EXISTS analysis_validations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    preregistration_id UUID NOT NULL,
    analysis_name TEXT NOT NULL,
    validation_result TEXT NOT NULL,
    actual_test TEXT NOT NULL,
    actual_variables TEXT NOT NULL,
    deviation_reason TEXT,
    is_exploratory BOOLEAN DEFAULT FALSE,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (preregistration_id) REFERENCES preregistrations(id)
);

-- Transparency reports
CREATE TABLE IF NOT EXISTS transparency_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    preregistration_id UUID NOT NULL,
    generated_at TIMESTAMP NOT NULL,
    report_data TEXT NOT NULL,
    public_url TEXT,

    FOREIGN KEY (preregistration_id) REFERENCES preregistrations(id)
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_preregistrations_experiment ON preregistrations(experiment_id);
CREATE INDEX IF NOT EXISTS idx_preregistrations_researcher ON preregistrations(researcher_id);
CREATE INDEX IF NOT EXISTS idx_preregistrations_status ON preregistrations(status);
CREATE INDEX IF NOT EXISTS idx_deviations_preregistration ON preregistration_deviations(preregistration_id);
CREATE INDEX IF NOT EXISTS idx_validations_preregistration ON analysis_validations(preregistration_id);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_preregistrations_timestamp()
RETURNS trigger AS $$
BEGIN
    NEW.updated_at := CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_preregistrations_timestamp ON preregistrations;
CREATE TRIGGER update_preregistrations_timestamp
BEFORE UPDATE ON preregistrations
FOR EACH ROW
EXECUTE PROCEDURE update_preregistrations_timestamp();


