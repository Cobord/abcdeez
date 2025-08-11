-- Protocol Versioning Tables

-- Main protocols table
CREATE TABLE IF NOT EXISTS protocols (
    id BLOB PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    experiment_id BLOB NOT NULL REFERENCES experiments(id),
    current_version_id BLOB, -- Will reference protocol_versions(id)
    created_by BLOB NOT NULL REFERENCES users(id),
    tags TEXT NOT NULL, -- JSON array
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_protocols_experiment ON protocols(experiment_id);
CREATE INDEX idx_protocols_created_by ON protocols(created_by);
CREATE INDEX idx_protocols_public ON protocols(is_public);

-- Protocol versions
CREATE TABLE IF NOT EXISTS protocol_versions (
    id BLOB PRIMARY KEY,
    protocol_id BLOB NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    version TEXT NOT NULL, -- Semantic versioning: "1.0.0"
    parent_version_id BLOB REFERENCES protocol_versions(id),
    status TEXT NOT NULL CHECK (status IN ('Draft', 'Review', 'Published', 'Deprecated', 'Archived')),
    definition TEXT NOT NULL, -- JSON protocol definition
    changelog TEXT NOT NULL,
    validation_rules TEXT, -- JSON validation rules
    metadata TEXT, -- JSON metadata
    created_by BLOB NOT NULL REFERENCES users(id),
    published_at TIMESTAMP,
    deprecated_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(protocol_id, version)
);

CREATE INDEX idx_protocol_versions_protocol ON protocol_versions(protocol_id);
CREATE INDEX idx_protocol_versions_status ON protocol_versions(status);
CREATE INDEX idx_protocol_versions_parent ON protocol_versions(parent_version_id);
CREATE INDEX idx_protocol_versions_created_by ON protocol_versions(created_by);

-- Protocol branches for experimental variations
CREATE TABLE IF NOT EXISTS protocol_branches (
    id BLOB PRIMARY KEY,
    protocol_id BLOB NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    base_version_id BLOB NOT NULL REFERENCES protocol_versions(id),
    description TEXT,
    created_by BLOB NOT NULL REFERENCES users(id),
    merged_into_version_id BLOB REFERENCES protocol_versions(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    merged_at TIMESTAMP,
    UNIQUE(protocol_id, name)
);

CREATE INDEX idx_protocol_branches_protocol ON protocol_branches(protocol_id);
CREATE INDEX idx_protocol_branches_base_version ON protocol_branches(base_version_id);

-- Protocol change history
CREATE TABLE IF NOT EXISTS protocol_changes (
    id BLOB PRIMARY KEY,
    protocol_id BLOB NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    change_type TEXT NOT NULL CHECK (change_type IN ('Created', 'Updated', 'Published', 'Deprecated')),
    version TEXT NOT NULL,
    description TEXT NOT NULL,
    data TEXT NOT NULL, -- JSON change data
    changed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    changed_by TEXT NOT NULL
);

CREATE INDEX idx_protocol_changes_protocol ON protocol_changes(protocol_id);
CREATE INDEX idx_protocol_changes_timestamp ON protocol_changes(changed_at);

-- Protocol validations log
CREATE TABLE IF NOT EXISTS protocol_validations (
    id BLOB PRIMARY KEY,
    protocol_id BLOB NOT NULL REFERENCES protocols(id),
    version_id BLOB NOT NULL REFERENCES protocol_versions(id),
    is_valid BOOLEAN NOT NULL,
    errors TEXT, -- JSON array of errors
    warnings TEXT, -- JSON array of warnings
    validated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    validated_by BLOB REFERENCES users(id)
);

CREATE INDEX idx_protocol_validations_protocol ON protocol_validations(protocol_id);
CREATE INDEX idx_protocol_validations_version ON protocol_validations(version_id);

-- Protocol usage metrics
CREATE TABLE IF NOT EXISTS protocol_metrics (
    protocol_id BLOB NOT NULL REFERENCES protocols(id),
    version_id BLOB NOT NULL REFERENCES protocol_versions(id),
    date DATE NOT NULL,
    total_sessions INTEGER NOT NULL DEFAULT 0,
    unique_learners INTEGER NOT NULL DEFAULT 0,
    avg_completion_rate REAL,
    avg_success_rate REAL,
    PRIMARY KEY (protocol_id, version_id, date)
);

CREATE INDEX idx_protocol_metrics_date ON protocol_metrics(date);

-- Add protocol_id to sessions table if not exists
-- This links sessions to specific protocol versions
ALTER TABLE sessions ADD COLUMN protocol_id BLOB REFERENCES protocols(id);
ALTER TABLE sessions ADD COLUMN protocol_version_id BLOB REFERENCES protocol_versions(id);

CREATE INDEX idx_sessions_protocol ON sessions(protocol_id);
CREATE INDEX idx_sessions_protocol_version ON sessions(protocol_version_id);

-- Add foreign key constraint for current_version_id after protocol_versions table exists
-- SQLite doesn't support ALTER TABLE ADD CONSTRAINT, so we'd need to recreate the table
-- In production, use a migration tool that handles this properly