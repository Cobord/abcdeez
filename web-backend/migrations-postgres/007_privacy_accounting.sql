-- Privacy budget accounting (Postgres)

CREATE TABLE IF NOT EXISTS privacy_budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    principal_type TEXT NOT NULL, -- user, api_key, global
    principal_id UUID,            -- nullable for global
    window_start TIMESTAMP NOT NULL,
    window_end TIMESTAMP NOT NULL,
    epsilon_total DOUBLE PRECISION NOT NULL,
    delta_total DOUBLE PRECISION NOT NULL,
    epsilon_spent DOUBLE PRECISION NOT NULL DEFAULT 0,
    delta_spent DOUBLE PRECISION NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_privacy_budgets_window
ON privacy_budgets(principal_type, principal_id, window_start, window_end);

CREATE TABLE IF NOT EXISTS privacy_spend_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    principal_type TEXT NOT NULL,
    principal_id UUID,
    endpoint TEXT NOT NULL,
    mechanism TEXT NOT NULL,
    epsilon_spent DOUBLE PRECISION NOT NULL,
    delta_spent DOUBLE PRECISION NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_privacy_spend_principal
ON privacy_spend_log(principal_type, principal_id, timestamp);


