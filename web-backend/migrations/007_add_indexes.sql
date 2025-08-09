-- Additional performance indexes for common query patterns

-- Optimize responses lookups within a session ordered by timestamp
CREATE INDEX IF NOT EXISTS idx_responses_session_timestamp
ON responses(session_id, timestamp);

-- Optimize sessions lookups by learner ordered by start_time
CREATE INDEX IF NOT EXISTS idx_sessions_learner_start_time
ON sessions(learner_id, start_time);

-- Optimize experiment participants lookups by learner
CREATE INDEX IF NOT EXISTS idx_experiment_participants_learner
ON experiment_participants(learner_id);


