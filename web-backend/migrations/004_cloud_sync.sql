-- Cloud Sync Support Migration
-- Adds support for multi-device synchronization and conflict resolution

-- Sync metadata for tracking device synchronization
CREATE TABLE IF NOT EXISTS sync_metadata (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    device_name TEXT,
    platform TEXT, -- ios, android, web, macos, windows, linux
    last_sync_at TIMESTAMP NOT NULL,
    sync_version INTEGER NOT NULL DEFAULT 1,
    sync_token TEXT UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, device_id)
);

-- Sync queue for pending changes
CREATE TABLE IF NOT EXISTS sync_queue (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    entity_type TEXT NOT NULL, -- session, response, learner_model, achievement, etc.
    entity_id TEXT NOT NULL,
    operation TEXT NOT NULL, -- create, update, delete
    data TEXT NOT NULL, -- JSON blob with the actual data
    sync_version INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    synced BOOLEAN NOT NULL DEFAULT FALSE,
    synced_at TIMESTAMP,
    conflict BOOLEAN NOT NULL DEFAULT FALSE,
    conflict_data TEXT -- JSON with conflict information
);

-- Conflict resolution log
CREATE TABLE IF NOT EXISTS sync_conflicts (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    local_data TEXT NOT NULL, -- JSON
    remote_data TEXT NOT NULL, -- JSON
    resolution_strategy TEXT, -- local_wins, remote_wins, merge, manual
    resolved_data TEXT, -- JSON with resolved data
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_at TIMESTAMP,
    resolved_by TEXT, -- user, auto, admin
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Sync checkpoints for efficient syncing
CREATE TABLE IF NOT EXISTS sync_checkpoints (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    checkpoint_type TEXT NOT NULL, -- full, incremental
    checkpoint_data TEXT NOT NULL, -- JSON with checkpoint state
    entity_versions TEXT NOT NULL, -- JSON with entity version map
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, checkpoint_type)
);

-- Offline queue for operations performed while offline
CREATE TABLE IF NOT EXISTS offline_queue (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    operation_data TEXT NOT NULL, -- JSON
    priority INTEGER NOT NULL DEFAULT 0, -- Higher priority syncs first
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at TIMESTAMP,
    error_message TEXT
);

-- Cloud provider settings (per user)
CREATE TABLE IF NOT EXISTS cloud_providers (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL, -- icloud, google_drive, onedrive, dropbox
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    access_token TEXT, -- Encrypted
    refresh_token TEXT, -- Encrypted
    token_expiry TIMESTAMP,
    storage_quota_bytes BIGINT,
    storage_used_bytes BIGINT,
    last_sync_at TIMESTAMP,
    sync_folder_id TEXT, -- Provider-specific folder/container ID
    metadata TEXT, -- JSON with provider-specific settings
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, provider)
);

-- Sync history for audit and debugging
CREATE TABLE IF NOT EXISTS sync_history (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    sync_type TEXT NOT NULL, -- push, pull, bidirectional
    entities_synced INTEGER NOT NULL DEFAULT 0,
    conflicts_found INTEGER NOT NULL DEFAULT 0,
    conflicts_resolved INTEGER NOT NULL DEFAULT 0,
    data_uploaded_bytes BIGINT,
    data_downloaded_bytes BIGINT,
    duration_ms INTEGER,
    success BOOLEAN NOT NULL DEFAULT TRUE,
    error_message TEXT,
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP
);

-- Device registration
CREATE TABLE IF NOT EXISTS registered_devices (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    device_name TEXT NOT NULL,
    device_type TEXT NOT NULL, -- phone, tablet, desktop, watch
    platform TEXT NOT NULL,
    platform_version TEXT,
    app_version TEXT,
    push_token TEXT, -- For push notifications
    last_seen_at TIMESTAMP NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, device_id)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_sync_metadata_user ON sync_metadata(user_id);
CREATE INDEX IF NOT EXISTS idx_sync_metadata_device ON sync_metadata(device_id);
CREATE INDEX IF NOT EXISTS idx_sync_queue_user ON sync_queue(user_id);
CREATE INDEX IF NOT EXISTS idx_sync_queue_synced ON sync_queue(synced);
CREATE INDEX IF NOT EXISTS idx_sync_conflicts_user ON sync_conflicts(user_id);
CREATE INDEX IF NOT EXISTS idx_sync_conflicts_resolved ON sync_conflicts(resolved);
CREATE INDEX IF NOT EXISTS idx_offline_queue_user ON offline_queue(user_id);
CREATE INDEX IF NOT EXISTS idx_offline_queue_processed ON offline_queue(processed);
CREATE INDEX IF NOT EXISTS idx_cloud_providers_user ON cloud_providers(user_id);
CREATE INDEX IF NOT EXISTS idx_sync_history_user ON sync_history(user_id);
CREATE INDEX IF NOT EXISTS idx_registered_devices_user ON registered_devices(user_id);