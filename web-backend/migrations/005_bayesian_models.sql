-- Bayesian Models Persistence Migration
-- Adds tables to store Bayesian learner models for persistent adaptive learning

-- Store serialized Bayesian models for each learner
CREATE TABLE IF NOT EXISTS bayesian_models (
    id BLOB PRIMARY KEY,
    learner_id BLOB NOT NULL REFERENCES learners(id) ON DELETE CASCADE,
    model_data TEXT NOT NULL, -- JSON serialized BayesianLearnerModel
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for efficient model retrieval
CREATE INDEX IF NOT EXISTS idx_bayesian_models_learner_id ON bayesian_models(learner_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_bayesian_models_updated ON bayesian_models(updated_at DESC);

-- Model update history for analysis
CREATE TABLE IF NOT EXISTS bayesian_model_updates (
    id BLOB PRIMARY KEY,
    learner_id BLOB NOT NULL REFERENCES learners(id) ON DELETE CASCADE,
    session_id BLOB REFERENCES sessions(id) ON DELETE SET NULL,
    update_type TEXT NOT NULL CHECK (update_type IN ('response_update', 'session_complete', 'manual_update')),
    prior_entropy REAL, -- Model entropy before update
    posterior_entropy REAL, -- Model entropy after update
    information_gain REAL, -- Information gained from this update
    response_data TEXT, -- JSON with response that triggered update
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for tracking model learning progress
CREATE INDEX IF NOT EXISTS idx_bayesian_updates_learner ON bayesian_model_updates(learner_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_bayesian_updates_session ON bayesian_model_updates(session_id);
CREATE INDEX IF NOT EXISTS idx_bayesian_updates_type ON bayesian_model_updates(update_type, timestamp DESC);

-- Audit the migration
INSERT INTO audit_log (id, timestamp, user_id, action, resource_type, resource_id, changes, ip_address, user_agent)
VALUES (
    randomblob(16),
    CURRENT_TIMESTAMP,
    NULL,
    'create',
    'migration', 
    '005_bayesian_models',
    '{"description": "Added Bayesian model persistence tables for adaptive learning continuity"}',
    NULL,
    'database_migration'
);