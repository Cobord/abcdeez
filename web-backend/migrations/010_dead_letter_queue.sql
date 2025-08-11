-- Dead Letter Queue for Failed Batch Jobs
-- This table stores jobs that have failed permanently after exhausting retries
-- These jobs require manual intervention or investigation

CREATE TABLE IF NOT EXISTS dead_letter_queue (
    id BLOB PRIMARY KEY,
    job_id BLOB NOT NULL,
    job_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    failure_reason TEXT NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    original_created_at TIMESTAMP NOT NULL,
    moved_to_dlq_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_error TEXT,
    stack_trace TEXT,
    retry_attempted BOOLEAN DEFAULT FALSE,
    resolution_notes TEXT,
    resolved_at TIMESTAMP,
    resolved_by TEXT,
    
    -- Index for finding unresolved entries
    CHECK (retry_count >= 0)
);

-- Indexes for DLQ queries
CREATE INDEX idx_dlq_job_id ON dead_letter_queue(job_id);
CREATE INDEX idx_dlq_moved_at ON dead_letter_queue(moved_to_dlq_at);
CREATE INDEX idx_dlq_job_type ON dead_letter_queue(job_type);
CREATE INDEX idx_dlq_unresolved ON dead_letter_queue(resolved_at) WHERE resolved_at IS NULL;

-- Update batch_jobs table to support DLQ
ALTER TABLE batch_jobs ADD COLUMN IF NOT EXISTS max_retries INTEGER DEFAULT 3;
ALTER TABLE batch_jobs ADD COLUMN IF NOT EXISTS backoff_seconds INTEGER DEFAULT 10;
ALTER TABLE batch_jobs ADD COLUMN IF NOT EXISTS scheduled_at TIMESTAMP;

-- Add status for dead letter
-- Note: SQLite doesn't support ALTER COLUMN, so we need to be careful with enum values
-- The application should handle 'dead_letter' as a valid status

-- Distributed lock tracking table
CREATE TABLE IF NOT EXISTS distributed_locks (
    lock_key TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    acquired_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    lock_type TEXT NOT NULL, -- 'job', 'federation', 'migration', etc.
    metadata TEXT -- JSON with additional lock information
);

-- Index for expired locks cleanup
CREATE INDEX idx_locks_expires_at ON distributed_locks(expires_at);

-- Federation sync status table for tracking
CREATE TABLE IF NOT EXISTS federation_sync_status (
    node_id BLOB PRIMARY KEY,
    last_sync_start TIMESTAMP,
    last_sync_complete TIMESTAMP,
    last_sync_status TEXT, -- 'success', 'failed', 'in_progress'
    last_error TEXT,
    consecutive_failures INTEGER DEFAULT 0,
    total_syncs INTEGER DEFAULT 0,
    total_failures INTEGER DEFAULT 0,
    metadata TEXT -- JSON with sync statistics
);

-- Job execution history for analytics
CREATE TABLE IF NOT EXISTS job_execution_history (
    id BLOB PRIMARY KEY,
    job_id BLOB NOT NULL,
    job_type TEXT NOT NULL,
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    duration_ms INTEGER,
    status TEXT NOT NULL, -- 'success', 'failed', 'timeout'
    error_message TEXT,
    worker_id TEXT,
    memory_used_mb INTEGER,
    cpu_time_ms INTEGER,
    
    FOREIGN KEY (job_id) REFERENCES batch_jobs(id)
);

-- Indexes for job history analytics
CREATE INDEX idx_job_history_job_id ON job_execution_history(job_id);
CREATE INDEX idx_job_history_started_at ON job_execution_history(started_at);
CREATE INDEX idx_job_history_job_type_status ON job_execution_history(job_type, status);

-- DLQ metrics view for monitoring
CREATE VIEW IF NOT EXISTS dlq_metrics AS
SELECT 
    job_type,
    COUNT(*) as total_entries,
    COUNT(CASE WHEN resolved_at IS NULL THEN 1 END) as unresolved_count,
    COUNT(CASE WHEN retry_attempted = TRUE THEN 1 END) as retry_attempted_count,
    MIN(moved_to_dlq_at) as oldest_entry,
    MAX(moved_to_dlq_at) as newest_entry
FROM dead_letter_queue
GROUP BY job_type;

-- Job performance metrics view
CREATE VIEW IF NOT EXISTS job_performance_metrics AS
SELECT 
    job_type,
    COUNT(*) as total_executions,
    AVG(duration_ms) as avg_duration_ms,
    MAX(duration_ms) as max_duration_ms,
    MIN(duration_ms) as min_duration_ms,
    COUNT(CASE WHEN status = 'success' THEN 1 END) as success_count,
    COUNT(CASE WHEN status = 'failed' THEN 1 END) as failure_count,
    COUNT(CASE WHEN status = 'timeout' THEN 1 END) as timeout_count,
    CAST(COUNT(CASE WHEN status = 'success' THEN 1 END) AS FLOAT) / 
        NULLIF(COUNT(*), 0) * 100 as success_rate
FROM job_execution_history
WHERE started_at >= datetime('now', '-7 days')
GROUP BY job_type;