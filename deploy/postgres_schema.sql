-- NodeUnion PostgreSQL schema bootstrap/migration script
-- Generated from crates/orchestrator/src/db/mod.rs init_schema

BEGIN;

CREATE TABLE IF NOT EXISTS networks (
    network_id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    orchestrator_url VARCHAR(512),
    status VARCHAR(50) NOT NULL DEFAULT 'Active',
    created_at_epoch_secs BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE networks ADD COLUMN IF NOT EXISTS orchestrator_url VARCHAR(512);
ALTER TABLE networks ADD COLUMN IF NOT EXISTS status VARCHAR(50) NOT NULL DEFAULT 'Active';
ALTER TABLE networks ADD COLUMN IF NOT EXISTS created_at_epoch_secs BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT;
ALTER TABLE networks ADD COLUMN IF NOT EXISTS created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

UPDATE networks SET status = 'Active' WHERE status IS NULL;
UPDATE networks SET created_at_epoch_secs = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE created_at_epoch_secs IS NULL;

CREATE TABLE IF NOT EXISTS nodes (
    node_id VARCHAR(255) PRIMARY KEY,
    network_id VARCHAR(255) NOT NULL DEFAULT 'default',
    agent_url VARCHAR(512) NOT NULL,
    provider_wallet VARCHAR(255),
    region VARCHAR(100),
    labels TEXT NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'Idle',
    is_idle BOOLEAN NOT NULL DEFAULT true,
    cpu_available_pct FLOAT NOT NULL DEFAULT 0.0,
    ram_available_mb BIGINT NOT NULL DEFAULT 0,
    disk_available_gb BIGINT NOT NULL DEFAULT 0,
    running_chunks INTEGER NOT NULL DEFAULT 0,
    last_seen_epoch_secs BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE nodes ADD COLUMN IF NOT EXISTS provider_wallet VARCHAR(255);

CREATE TABLE IF NOT EXISTS jobs (
    job_id VARCHAR(255) PRIMARY KEY,
    network_id VARCHAR(255) NOT NULL DEFAULT 'default',
    user_wallet VARCHAR(255),
    image VARCHAR(512) NOT NULL,
    command TEXT,
    cpu_limit FLOAT NOT NULL,
    ram_limit_mb BIGINT NOT NULL,
    exposed_port BIGINT,
    status VARCHAR(50) NOT NULL DEFAULT 'Pending',
    assigned_node_id VARCHAR(255),
    created_at_epoch_secs BIGINT NOT NULL,
    error_detail TEXT,
    deploy_url VARCHAR(512),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (assigned_node_id) REFERENCES nodes(node_id) ON DELETE SET NULL
);

ALTER TABLE jobs ADD COLUMN IF NOT EXISTS user_wallet VARCHAR(255);
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS error_detail TEXT;
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS exposed_port BIGINT;
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS deploy_url VARCHAR(512);

CREATE TABLE IF NOT EXISTS attempts (
    attempt_id VARCHAR(255) PRIMARY KEY,
    job_id VARCHAR(255) NOT NULL,
    attempt_number INTEGER NOT NULL,
    assigned_node_id VARCHAR(255),
    last_error TEXT,
    next_retry_at_epoch_secs BIGINT,
    created_at_epoch_secs BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (job_id) REFERENCES jobs(job_id) ON DELETE CASCADE,
    FOREIGN KEY (assigned_node_id) REFERENCES nodes(node_id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS user_entitlements (
    entitlement_id VARCHAR(255) PRIMARY KEY,
    user_wallet VARCHAR(255) NOT NULL,
    network_id VARCHAR(255) NOT NULL,
    bought_units BIGINT NOT NULL,
    used_units BIGINT NOT NULL DEFAULT 0,
    escrow_account VARCHAR(255),
    escrow_tx_hash VARCHAR(255),
    expiry_epoch_secs BIGINT,
    created_at_epoch_secs BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (network_id) REFERENCES networks(network_id) ON DELETE CASCADE,
    UNIQUE(user_wallet, network_id)
);

CREATE TABLE IF NOT EXISTS settlements (
    settlement_id VARCHAR(255) PRIMARY KEY,
    job_id VARCHAR(255) NOT NULL,
    user_wallet VARCHAR(255) NOT NULL,
    provider_wallet VARCHAR(255),
    network_id VARCHAR(255) NOT NULL,
    units_metered BIGINT NOT NULL,
    amount_tokens BIGINT NOT NULL,
    tx_hash VARCHAR(255),
    tx_status VARCHAR(50),
    settlement_type VARCHAR(50),
    created_at_epoch_secs BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (job_id) REFERENCES jobs(job_id) ON DELETE CASCADE,
    FOREIGN KEY (network_id) REFERENCES networks(network_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS provider_settlements (
    provider_settlement_id VARCHAR(255) PRIMARY KEY,
    job_id VARCHAR(255) NOT NULL,
    provider_wallet VARCHAR(255) NOT NULL,
    network_id VARCHAR(255) NOT NULL,
    units_earned BIGINT NOT NULL,
    amount_tokens BIGINT NOT NULL,
    tx_hash VARCHAR(255),
    tx_status VARCHAR(50),
    created_at_epoch_secs BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (job_id) REFERENCES jobs(job_id) ON DELETE CASCADE,
    FOREIGN KEY (network_id) REFERENCES networks(network_id) ON DELETE CASCADE
);

INSERT INTO networks (network_id, name, description, created_at_epoch_secs)
VALUES ('default', 'Default Network', 'Auto-created fallback network', EXTRACT(EPOCH FROM NOW())::BIGINT)
ON CONFLICT (network_id) DO NOTHING;

COMMIT;
