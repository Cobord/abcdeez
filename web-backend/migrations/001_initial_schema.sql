-- Users and Authentication
CREATE TABLE IF NOT EXISTS users (
    id BLOB PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    metadata TEXT
);

-- Learners (can be anonymous)
CREATE TABLE IF NOT EXISTS learners (
    id BLOB PRIMARY KEY,
    user_id BLOB REFERENCES users(id),
    display_name TEXT,
    created_at TIMESTAMP NOT NULL,
    last_active TIMESTAMP,
    total_practice_time_seconds INTEGER DEFAULT 0,
    metadata TEXT
);

-- Training Sessions
CREATE TABLE IF NOT EXISTS sessions (
    id BLOB PRIMARY KEY,
    learner_id BLOB NOT NULL REFERENCES learners(id),
    topology_type TEXT NOT NULL,
    topology_data TEXT NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    status TEXT NOT NULL,
    summary TEXT
);

-- Individual Task Responses (Event Store)
CREATE TABLE IF NOT EXISTS responses (
    id BLOB PRIMARY KEY,
    session_id BLOB NOT NULL REFERENCES sessions(id),
    sequence_number INTEGER NOT NULL,
    task_type TEXT NOT NULL,
    task_data TEXT NOT NULL,
    user_answer TEXT,
    correct INTEGER NOT NULL,
    response_time_ms INTEGER NOT NULL,
    hint_level INTEGER,
    timestamp TIMESTAMP NOT NULL
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_sessions_learner ON sessions(learner_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_responses_session ON responses(session_id, sequence_number);
