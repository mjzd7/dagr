-- ====================================================================
-- DAGR Cloud Plane: System of Record & Transactional Outbox DDL
-- ====================================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- 1. Organizations Sharding Key
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 2. Repositories Table
CREATE TABLE IF NOT EXISTS repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    repo_slug VARCHAR(255) NOT NULL, -- e.g. "mjzd7/dagr"
    default_branch VARCHAR(64) DEFAULT 'main',
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(organization_id, repo_slug)
);

-- 3. Ingested Commits & CodeGraph Snapshots
CREATE TABLE IF NOT EXISTS commit_snapshots (
    commit_sha VARCHAR(64) PRIMARY KEY,
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    author_email VARCHAR(255) NOT NULL,
    commit_message TEXT,
    tree_blake3_hash VARCHAR(64) NOT NULL,
    total_symbols INT DEFAULT 0,
    ingested_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 4. Transactional Outbox Table for Debezium CDC WAL Streaming
CREATE TABLE IF NOT EXISTS outbox_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    event_type VARCHAR(128) NOT NULL, -- 'CommitIngested', 'ViolationDetected', 'ProofGenerated'
    aggregate_id VARCHAR(255) NOT NULL, -- commit_sha or pr_id
    payload JSONB NOT NULL,
    idempotency_key VARCHAR(255) UNIQUE NOT NULL,
    status VARCHAR(32) DEFAULT 'PENDING', -- 'PENDING', 'PROCESSED', 'FAILED'
    retry_count INT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX IF NOT EXISTS idx_outbox_unprocessed ON outbox_events(status, created_at) WHERE status = 'PENDING';
CREATE INDEX IF NOT EXISTS idx_outbox_repo_agg ON outbox_events(repository_id, aggregate_id);

-- 5. Seed default test organization and repository
INSERT INTO organizations (id, name) 
VALUES ('00000000-0000-0000-0000-000000000001', 'DAGR Open Source Org')
ON CONFLICT (name) DO NOTHING;

INSERT INTO repositories (id, organization_id, repo_slug)
VALUES ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', 'mjzd7/dagr')
ON CONFLICT (organization_id, repo_slug) DO NOTHING;
