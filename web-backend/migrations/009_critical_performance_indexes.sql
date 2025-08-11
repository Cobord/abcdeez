-- Critical Performance Indexes Migration
-- These indexes are required for production-level performance
-- Without these, the application will experience severe performance degradation under load

-- ============================================
-- RESPONSES TABLE INDEXES
-- ============================================
-- Most queries filter by session_id
CREATE INDEX IF NOT EXISTS idx_responses_session_id 
    ON responses(session_id);

-- Timestamp is used for analytics and ordering
CREATE INDEX IF NOT EXISTS idx_responses_timestamp 
    ON responses(timestamp DESC);

-- Composite index for session analytics queries
CREATE INDEX IF NOT EXISTS idx_responses_session_timestamp 
    ON responses(session_id, timestamp DESC);

-- Learner response queries
CREATE INDEX IF NOT EXISTS idx_responses_learner_session 
    ON responses(learner_id, session_id);

-- ============================================
-- SESSIONS TABLE INDEXES
-- ============================================
-- Most queries filter by learner_id
CREATE INDEX IF NOT EXISTS idx_sessions_learner_id 
    ON sessions(learner_id);

-- Status and date filtering for dashboard queries
CREATE INDEX IF NOT EXISTS idx_sessions_status_date 
    ON sessions(status, start_time DESC);

-- Composite index for learner session lookups
CREATE INDEX IF NOT EXISTS idx_sessions_learner_status 
    ON sessions(learner_id, status);

-- Session analytics by experiment
CREATE INDEX IF NOT EXISTS idx_sessions_experiment_status 
    ON sessions(experiment_id, status);

-- ============================================
-- AUDIT LOG INDEXES
-- ============================================
-- User activity tracking
CREATE INDEX IF NOT EXISTS idx_audit_log_user_timestamp 
    ON audit_log(user_id, timestamp DESC);

-- Security event monitoring
CREATE INDEX IF NOT EXISTS idx_audit_log_action_timestamp 
    ON audit_log(action, timestamp DESC);

-- Resource tracking
CREATE INDEX IF NOT EXISTS idx_audit_log_resource_type 
    ON audit_log(resource_type, resource_id);

-- ============================================
-- USERS TABLE INDEXES
-- ============================================
-- Email lookups for authentication
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email 
    ON users(email);

-- Username lookups
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username 
    ON users(username);

-- OAuth provider lookups
CREATE INDEX IF NOT EXISTS idx_users_oauth_provider 
    ON users(oauth_provider, oauth_id);

-- ============================================
-- LEARNERS TABLE INDEXES  
-- ============================================
-- User to learner mapping
CREATE INDEX IF NOT EXISTS idx_learners_user_id 
    ON learners(user_id);

-- Study/condition filtering
CREATE INDEX IF NOT EXISTS idx_learners_study_condition 
    ON learners(study_id, condition_assignment);

-- Active learner queries
CREATE INDEX IF NOT EXISTS idx_learners_status 
    ON learners(status) 
    WHERE status = 'active';

-- ============================================
-- PRIVACY BUDGETS INDEXES
-- ============================================
-- Principal ID lookups for budget checking
CREATE UNIQUE INDEX IF NOT EXISTS idx_privacy_budgets_principal 
    ON privacy_budgets(principal_id);

-- Budget monitoring queries
CREATE INDEX IF NOT EXISTS idx_privacy_budgets_spent 
    ON privacy_budgets(epsilon_spent, delta_spent);

-- ============================================
-- BATCH JOBS INDEXES
-- ============================================
-- Job queue processing
CREATE INDEX IF NOT EXISTS idx_batch_jobs_status_scheduled 
    ON batch_jobs(status, scheduled_at) 
    WHERE status IN ('pending', 'running');

-- Job history queries
CREATE INDEX IF NOT EXISTS idx_batch_jobs_created 
    ON batch_jobs(created_at DESC);

-- ============================================
-- FEDERATION NODES INDEXES
-- ============================================
-- Node status monitoring
CREATE INDEX IF NOT EXISTS idx_federation_nodes_status 
    ON federation_nodes(status, last_sync);

-- Node lookup by URL
CREATE UNIQUE INDEX IF NOT EXISTS idx_federation_nodes_url 
    ON federation_nodes(node_url);

-- ============================================
-- PARTIAL INDEXES FOR COMMON QUERIES
-- ============================================
-- Active sessions only (reduces index size)
CREATE INDEX IF NOT EXISTS idx_sessions_active_only 
    ON sessions(learner_id, start_time DESC) 
    WHERE status = 'active';

-- Recent responses only (last 30 days)
-- Note: This would need periodic rebuilding
-- CREATE INDEX IF NOT EXISTS idx_responses_recent 
--     ON responses(session_id, timestamp) 
--     WHERE timestamp > CURRENT_TIMESTAMP - INTERVAL '30 days';

-- ============================================
-- ANALYZE TABLES FOR QUERY PLANNER
-- ============================================
-- Update statistics for query optimizer
ANALYZE responses;
ANALYZE sessions;
ANALYZE learners;
ANALYZE users;
ANALYZE audit_log;
ANALYZE privacy_budgets;
ANALYZE batch_jobs;
ANALYZE federation_nodes;