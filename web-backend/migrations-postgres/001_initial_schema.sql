-- PostgreSQL Migration: Initial Schema
-- Enable UUID extension for generating UUIDs
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users and Authentication
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT -- JSON blob for additional user data
);

-- Learners (can be anonymous)
CREATE TABLE IF NOT EXISTS learners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    display_name TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_active TIMESTAMP,
    total_practice_time_seconds BIGINT DEFAULT 0,
    metadata TEXT -- JSON blob for learner-specific data
);

-- Training Sessions
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    learner_id UUID NOT NULL REFERENCES learners(id),
    topology_type TEXT NOT NULL,
    topology_data TEXT NOT NULL, -- JSON blob with topology definition
    start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP,
    status TEXT NOT NULL CHECK (status IN ('active', 'completed', 'abandoned')),
    summary TEXT -- JSON blob with session statistics
);

-- Individual Task Responses (Event Store)
CREATE TABLE IF NOT EXISTS responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id),
    sequence_number INTEGER NOT NULL,
    task_type TEXT NOT NULL,
    task_data TEXT NOT NULL, -- JSON blob with task parameters
    user_answer TEXT,
    correct BOOLEAN NOT NULL,
    response_time_ms INTEGER NOT NULL,
    hint_level INTEGER, -- 0=none, 1-4=hint levels
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Model Snapshots (for analysis and recovery)
CREATE TABLE IF NOT EXISTS model_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    learner_id UUID NOT NULL REFERENCES learners(id),
    session_id UUID REFERENCES sessions(id),
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    parameters TEXT NOT NULL, -- JSON blob with full learner model state
    metrics TEXT -- JSON blob with calculated metrics
);

-- Experiments and Studies
CREATE TABLE IF NOT EXISTS experiments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    config TEXT NOT NULL, -- JSON blob with experiment configuration
    start_date DATE,
    end_date DATE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'completed', 'archived'))
);

CREATE TABLE IF NOT EXISTS experiment_participants (
    experiment_id UUID REFERENCES experiments(id),
    learner_id UUID REFERENCES learners(id),
    condition TEXT, -- Control, treatment_a, treatment_b, etc.
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT, -- JSON blob for participant-specific data
    PRIMARY KEY (experiment_id, learner_id)
);

-- Intervention Events
CREATE TABLE IF NOT EXISTS interventions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID REFERENCES sessions(id),
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    intervention_type TEXT NOT NULL, -- hint, difficulty_change, break_suggestion
    details TEXT, -- JSON blob with intervention-specific data
    effectiveness BOOLEAN -- Was the next response correct?
);

-- Audit Log for tracking all data access and mutations
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_id UUID REFERENCES users(id),
    action TEXT NOT NULL, -- create, read, update, delete
    resource_type TEXT NOT NULL, -- user, learner, session, etc.
    resource_id TEXT NOT NULL,
    changes TEXT, -- JSON blob with old/new values for updates
    ip_address TEXT,
    user_agent TEXT
);

-- Music domain specific tables
CREATE TABLE IF NOT EXISTS music_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    scale_type TEXT NOT NULL,
    key_signature TEXT NOT NULL,
    chord_progression TEXT, -- JSON array of chords
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Background job queue for batch processing
CREATE TABLE IF NOT EXISTS job_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type TEXT NOT NULL, -- statistics_update, leaderboard_refresh, etc.
    payload TEXT NOT NULL, -- JSON blob with job parameters
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_learners_user_id ON learners(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_learner ON sessions(learner_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_start_time ON sessions(start_time);
CREATE INDEX IF NOT EXISTS idx_responses_session ON responses(session_id, sequence_number);
CREATE INDEX IF NOT EXISTS idx_responses_timestamp ON responses(timestamp);
CREATE INDEX IF NOT EXISTS idx_model_snapshots_learner ON model_snapshots(learner_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_experiments_status ON experiments(status);
CREATE INDEX IF NOT EXISTS idx_interventions_session ON interventions(session_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_resource ON audit_log(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_job_queue_status ON job_queue(status, created_at);