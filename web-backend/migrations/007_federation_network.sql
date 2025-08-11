-- Federation Network Tables

-- Federation nodes (partner institutions)
CREATE TABLE IF NOT EXISTS federation_nodes (
    id BLOB PRIMARY KEY,
    institution_id TEXT NOT NULL UNIQUE,
    institution_name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_version TEXT NOT NULL,
    public_key TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('Active', 'Inactive', 'Suspended', 'Pending')),
    last_heartbeat TIMESTAMP,
    capabilities TEXT NOT NULL, -- JSON array
    metadata TEXT, -- JSON object
    last_audit_date TIMESTAMP,
    next_audit_date TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_federation_nodes_status ON federation_nodes(status);
CREATE INDEX idx_federation_nodes_institution ON federation_nodes(institution_id);

-- Federation API keys for node authentication
CREATE TABLE IF NOT EXISTS federation_api_keys (
    node_id BLOB NOT NULL REFERENCES federation_nodes(id) ON DELETE CASCADE,
    api_key TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked_at TIMESTAMP,
    PRIMARY KEY (node_id, api_key)
);

-- Federation agreements between institutions
CREATE TABLE IF NOT EXISTS federation_agreements (
    id BLOB PRIMARY KEY,
    initiator_node_id BLOB NOT NULL REFERENCES federation_nodes(id),
    partner_node_id BLOB NOT NULL REFERENCES federation_nodes(id),
    agreement_type TEXT NOT NULL CHECK (agreement_type IN ('DataSharing', 'ResearchCollaboration', 'ProtocolExchange', 'Full')),
    data_sharing_rules TEXT NOT NULL, -- JSON object
    compliance_requirements TEXT NOT NULL, -- JSON array
    status TEXT NOT NULL CHECK (status IN ('Proposed', 'Negotiating', 'Active', 'Expired', 'Terminated')),
    expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(initiator_node_id, partner_node_id)
);

CREATE INDEX idx_federation_agreements_status ON federation_agreements(status);
CREATE INDEX idx_federation_agreements_nodes ON federation_agreements(initiator_node_id, partner_node_id);

-- Federation heartbeats for monitoring
CREATE TABLE IF NOT EXISTS federation_heartbeats (
    node_id BLOB NOT NULL REFERENCES federation_nodes(id) ON DELETE CASCADE,
    timestamp TIMESTAMP NOT NULL,
    active_experiments INTEGER NOT NULL DEFAULT 0,
    active_learners INTEGER NOT NULL DEFAULT 0,
    api_version TEXT NOT NULL,
    PRIMARY KEY (node_id, timestamp)
);

CREATE INDEX idx_federation_heartbeats_timestamp ON federation_heartbeats(timestamp);

-- Federation data transfers
CREATE TABLE IF NOT EXISTS federation_transfers (
    id BLOB PRIMARY KEY,
    source_node_id BLOB NOT NULL REFERENCES federation_nodes(id),
    target_node_id BLOB NOT NULL REFERENCES federation_nodes(id),
    data_type TEXT NOT NULL,
    data TEXT NOT NULL, -- Encrypted JSON
    data_size INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('Queued', 'InProgress', 'Completed', 'Failed')),
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE INDEX idx_federation_transfers_status ON federation_transfers(status);
CREATE INDEX idx_federation_transfers_nodes ON federation_transfers(source_node_id, target_node_id);

-- Federation compliance records
CREATE TABLE IF NOT EXISTS federation_compliance (
    node_id BLOB NOT NULL REFERENCES federation_nodes(id) ON DELETE CASCADE,
    compliance_type TEXT NOT NULL,
    is_compliant BOOLEAN NOT NULL,
    expiry_date TIMESTAMP,
    evidence_url TEXT,
    notes TEXT,
    verified_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    verified_by BLOB,
    PRIMARY KEY (node_id, compliance_type)
);

CREATE INDEX idx_federation_compliance_expiry ON federation_compliance(expiry_date);