-- Protocol Versioning Tables (PostgreSQL)

-- Main protocols table
CREATE TABLE IF NOT EXISTS protocols (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    experiment_id UUID NOT NULL REFERENCES experiments(id),
    current_version_id UUID, -- Will reference protocol_versions(id)
    created_by UUID NOT NULL REFERENCES users(id),
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_protocols_experiment ON protocols(experiment_id);
CREATE INDEX idx_protocols_created_by ON protocols(created_by);
CREATE INDEX idx_protocols_public ON protocols(is_public);
CREATE INDEX idx_protocols_tags ON protocols USING gin(tags);

-- Protocol versions
CREATE TABLE IF NOT EXISTS protocol_versions (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    version TEXT NOT NULL, -- Semantic versioning: "1.0.0"
    parent_version_id UUID REFERENCES protocol_versions(id),
    status TEXT NOT NULL CHECK (status IN ('Draft', 'Review', 'Published', 'Deprecated', 'Archived')),
    definition JSONB NOT NULL,
    changelog TEXT NOT NULL,
    validation_rules JSONB,
    metadata JSONB,
    created_by UUID NOT NULL REFERENCES users(id),
    published_at TIMESTAMP WITH TIME ZONE,
    deprecated_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(protocol_id, version)
);

CREATE INDEX idx_protocol_versions_protocol ON protocol_versions(protocol_id);
CREATE INDEX idx_protocol_versions_status ON protocol_versions(status);
CREATE INDEX idx_protocol_versions_parent ON protocol_versions(parent_version_id);
CREATE INDEX idx_protocol_versions_created_by ON protocol_versions(created_by);
CREATE INDEX idx_protocol_versions_definition ON protocol_versions USING gin(definition);

-- Protocol branches for experimental variations
CREATE TABLE IF NOT EXISTS protocol_branches (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    base_version_id UUID NOT NULL REFERENCES protocol_versions(id),
    description TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    merged_into_version_id UUID REFERENCES protocol_versions(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    merged_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(protocol_id, name)
);

CREATE INDEX idx_protocol_branches_protocol ON protocol_branches(protocol_id);
CREATE INDEX idx_protocol_branches_base_version ON protocol_branches(base_version_id);

-- Protocol change history
CREATE TABLE IF NOT EXISTS protocol_changes (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    change_type TEXT NOT NULL CHECK (change_type IN ('Created', 'Updated', 'Published', 'Deprecated')),
    version TEXT NOT NULL,
    description TEXT NOT NULL,
    data JSONB NOT NULL,
    changed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    changed_by TEXT NOT NULL
);

CREATE INDEX idx_protocol_changes_protocol ON protocol_changes(protocol_id);
CREATE INDEX idx_protocol_changes_timestamp ON protocol_changes(changed_at);
CREATE INDEX idx_protocol_changes_data ON protocol_changes USING gin(data);

-- Protocol validations log
CREATE TABLE IF NOT EXISTS protocol_validations (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id),
    version_id UUID NOT NULL REFERENCES protocol_versions(id),
    is_valid BOOLEAN NOT NULL,
    errors JSONB DEFAULT '[]'::jsonb,
    warnings JSONB DEFAULT '[]'::jsonb,
    validated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    validated_by UUID REFERENCES users(id)
);

CREATE INDEX idx_protocol_validations_protocol ON protocol_validations(protocol_id);
CREATE INDEX idx_protocol_validations_version ON protocol_validations(version_id);

-- Protocol usage metrics
CREATE TABLE IF NOT EXISTS protocol_metrics (
    protocol_id UUID NOT NULL REFERENCES protocols(id),
    version_id UUID NOT NULL REFERENCES protocol_versions(id),
    date DATE NOT NULL,
    total_sessions INTEGER NOT NULL DEFAULT 0,
    unique_learners INTEGER NOT NULL DEFAULT 0,
    avg_completion_rate DECIMAL(5,4),
    avg_success_rate DECIMAL(5,4),
    PRIMARY KEY (protocol_id, version_id, date)
);

CREATE INDEX idx_protocol_metrics_date ON protocol_metrics(date);

-- Add protocol_id to sessions table if not exists
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='sessions' AND column_name='protocol_id') THEN
        ALTER TABLE sessions ADD COLUMN protocol_id UUID REFERENCES protocols(id);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='sessions' AND column_name='protocol_version_id') THEN
        ALTER TABLE sessions ADD COLUMN protocol_version_id UUID REFERENCES protocol_versions(id);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_sessions_protocol ON sessions(protocol_id);
CREATE INDEX IF NOT EXISTS idx_sessions_protocol_version ON sessions(protocol_version_id);

-- Add foreign key constraint for current_version_id
ALTER TABLE protocols 
    ADD CONSTRAINT fk_protocols_current_version 
    FOREIGN KEY (current_version_id) 
    REFERENCES protocol_versions(id);

-- Update triggers for updated_at columns
CREATE TRIGGER update_protocols_updated_at BEFORE UPDATE ON protocols
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_protocol_versions_updated_at BEFORE UPDATE ON protocol_versions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();