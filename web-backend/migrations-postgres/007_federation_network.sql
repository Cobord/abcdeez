-- Federation Network Tables (PostgreSQL)

-- Federation nodes (partner institutions)
CREATE TABLE IF NOT EXISTS federation_nodes (
    id UUID PRIMARY KEY,
    institution_id TEXT NOT NULL UNIQUE,
    institution_name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_version TEXT NOT NULL,
    public_key TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('Active', 'Inactive', 'Suspended', 'Pending')),
    last_heartbeat TIMESTAMP WITH TIME ZONE,
    capabilities JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB,
    last_audit_date TIMESTAMP WITH TIME ZONE,
    next_audit_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_federation_nodes_status ON federation_nodes(status);
CREATE INDEX idx_federation_nodes_institution ON federation_nodes(institution_id);
CREATE INDEX idx_federation_nodes_capabilities ON federation_nodes USING gin(capabilities);

-- Federation API keys for node authentication
CREATE TABLE IF NOT EXISTS federation_api_keys (
    node_id UUID NOT NULL REFERENCES federation_nodes(id) ON DELETE CASCADE,
    api_key TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMP WITH TIME ZONE,
    PRIMARY KEY (node_id, api_key)
);

-- Federation agreements between institutions
CREATE TABLE IF NOT EXISTS federation_agreements (
    id UUID PRIMARY KEY,
    initiator_node_id UUID NOT NULL REFERENCES federation_nodes(id),
    partner_node_id UUID NOT NULL REFERENCES federation_nodes(id),
    agreement_type TEXT NOT NULL CHECK (agreement_type IN ('DataSharing', 'ResearchCollaboration', 'ProtocolExchange', 'Full')),
    data_sharing_rules JSONB NOT NULL DEFAULT '{}'::jsonb,
    compliance_requirements JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL CHECK (status IN ('Proposed', 'Negotiating', 'Active', 'Expired', 'Terminated')),
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(initiator_node_id, partner_node_id)
);

CREATE INDEX idx_federation_agreements_status ON federation_agreements(status);
CREATE INDEX idx_federation_agreements_nodes ON federation_agreements(initiator_node_id, partner_node_id);
CREATE INDEX idx_federation_agreements_data_rules ON federation_agreements USING gin(data_sharing_rules);

-- Federation heartbeats for monitoring
CREATE TABLE IF NOT EXISTS federation_heartbeats (
    node_id UUID NOT NULL REFERENCES federation_nodes(id) ON DELETE CASCADE,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    active_experiments INTEGER NOT NULL DEFAULT 0,
    active_learners INTEGER NOT NULL DEFAULT 0,
    api_version TEXT NOT NULL,
    PRIMARY KEY (node_id, timestamp)
);

CREATE INDEX idx_federation_heartbeats_timestamp ON federation_heartbeats(timestamp);

-- Federation data transfers
CREATE TABLE IF NOT EXISTS federation_transfers (
    id UUID PRIMARY KEY,
    source_node_id UUID NOT NULL REFERENCES federation_nodes(id),
    target_node_id UUID NOT NULL REFERENCES federation_nodes(id),
    data_type TEXT NOT NULL,
    data JSONB NOT NULL, -- Encrypted JSON
    data_size BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('Queued', 'InProgress', 'Completed', 'Failed')),
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_federation_transfers_status ON federation_transfers(status);
CREATE INDEX idx_federation_transfers_nodes ON federation_transfers(source_node_id, target_node_id);
CREATE INDEX idx_federation_transfers_created ON federation_transfers(created_at);

-- Federation compliance records
CREATE TABLE IF NOT EXISTS federation_compliance (
    node_id UUID NOT NULL REFERENCES federation_nodes(id) ON DELETE CASCADE,
    compliance_type TEXT NOT NULL,
    is_compliant BOOLEAN NOT NULL,
    expiry_date TIMESTAMP WITH TIME ZONE,
    evidence_url TEXT,
    notes TEXT,
    verified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    verified_by UUID REFERENCES users(id),
    PRIMARY KEY (node_id, compliance_type)
);

CREATE INDEX idx_federation_compliance_expiry ON federation_compliance(expiry_date);

-- Create update trigger for updated_at columns
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_federation_nodes_updated_at BEFORE UPDATE ON federation_nodes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_federation_agreements_updated_at BEFORE UPDATE ON federation_agreements
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();