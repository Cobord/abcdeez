-- OAuth Providers Support Migration
-- Add columns to support Apple Sign In, GitHub, and other OAuth authentication

-- Add OAuth provider specific columns
ALTER TABLE users ADD COLUMN apple_user_id TEXT;
ALTER TABLE users ADD COLUMN github_user_id TEXT;
ALTER TABLE users ADD COLUMN oauth_provider_id TEXT;
ALTER TABLE users ADD COLUMN auth_provider TEXT DEFAULT 'local' CHECK (auth_provider IN ('local', 'apple', 'github'));
ALTER TABLE users ADD COLUMN is_private_email BOOLEAN DEFAULT FALSE;

-- Create unique indexes for OAuth IDs
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_apple_user_id_unique ON users(apple_user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_github_user_id_unique ON users(github_user_id);

-- Make password_hash optional for OAuth users
-- Note: SQLite doesn't support ALTER COLUMN, so we'll handle this in the application logic
-- Password hash will be NULL for Apple Sign In users

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_apple_user_id ON users(apple_user_id);
CREATE INDEX IF NOT EXISTS idx_users_github_user_id ON users(github_user_id);
CREATE INDEX IF NOT EXISTS idx_users_oauth_provider_id ON users(oauth_provider_id);
CREATE INDEX IF NOT EXISTS idx_users_auth_provider ON users(auth_provider);

-- Add OAuth credential validation tracking table (generalized for all providers)
CREATE TABLE IF NOT EXISTS oauth_credential_checks (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider IN ('apple', 'github')),
    provider_user_id TEXT NOT NULL, -- apple_user_id or github_user_id
    last_check_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    credential_state TEXT NOT NULL CHECK (credential_state IN ('authorized', 'revoked', 'not_found', 'unknown')),
    error_details TEXT, -- JSON with error information if check failed
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(user_id, provider) -- One record per user per provider
);

CREATE INDEX IF NOT EXISTS idx_oauth_credential_checks_user ON oauth_credential_checks(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_credential_checks_provider_user ON oauth_credential_checks(provider, provider_user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_credential_checks_state ON oauth_credential_checks(credential_state);
CREATE INDEX IF NOT EXISTS idx_oauth_credential_checks_time ON oauth_credential_checks(last_check_time);

-- Add OAuth providers audit events
INSERT INTO audit_log (id, timestamp, user_id, action, resource_type, resource_id, changes, ip_address, user_agent)
VALUES (
    randomblob(16),
    CURRENT_TIMESTAMP,
    NULL,
    'create',
    'migration',
    '002_oauth_providers',
    '{"description": "Added OAuth providers support (Apple Sign In, GitHub) to users table"}',
    NULL,
    'database_migration'
);