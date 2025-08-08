-- Users for authentication
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Learners (can be linked to users or anonymous)
CREATE TABLE IF NOT EXISTS learners (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    display_name TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON
);

-- Training sessions
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    learner_id TEXT NOT NULL REFERENCES learners(id) ON DELETE CASCADE,
    topology_type TEXT NOT NULL,
    topology_data JSON,
    start_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP,
    status TEXT DEFAULT 'active', -- active, completed, abandoned
    summary JSON
);

-- Task responses
CREATE TABLE IF NOT EXISTS responses (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    task_type TEXT NOT NULL,
    task_data JSON NOT NULL,
    user_answer TEXT,
    correct BOOLEAN NOT NULL,
    response_time_ms INTEGER NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Model snapshots for analysis
CREATE TABLE IF NOT EXISTS model_snapshots (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    learner_id TEXT NOT NULL REFERENCES learners(id) ON DELETE CASCADE,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    parameters JSON NOT NULL,
    metrics JSON
);

-- Experiments for research
CREATE TABLE IF NOT EXISTS experiments (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    description TEXT,
    config JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT REFERENCES users(id)
);

-- Link learners to experiments
CREATE TABLE IF NOT EXISTS experiment_participants (
    experiment_id TEXT NOT NULL REFERENCES experiments(id) ON DELETE CASCADE,
    learner_id TEXT NOT NULL REFERENCES learners(id) ON DELETE CASCADE,
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON,
    PRIMARY KEY (experiment_id, learner_id)
);

-- Indexes for performance
CREATE INDEX idx_sessions_learner ON sessions(learner_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_responses_session ON responses(session_id);
CREATE INDEX idx_responses_timestamp ON responses(timestamp);
CREATE INDEX idx_model_snapshots_learner ON model_snapshots(learner_id);
CREATE INDEX idx_model_snapshots_timestamp ON model_snapshots(timestamp);