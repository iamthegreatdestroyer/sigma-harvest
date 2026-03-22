//! Database schema and migrations.

use rusqlite::Connection;
use super::DbError;

/// Run all pending migrations.
pub fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_001)?;
    tracing::info!("Database migrations complete");
    Ok(())
}

const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS wallets (
    id TEXT PRIMARY KEY,
    derivation_path TEXT NOT NULL,
    public_address TEXT NOT NULL UNIQUE,
    chain TEXT NOT NULL,
    label TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS opportunities (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    chain TEXT NOT NULL,
    opportunity_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    url TEXT,
    contract_address TEXT,
    estimated_value_usd REAL,
    gas_cost_estimate REAL,
    harvest_score INTEGER DEFAULT 0,
    risk_level TEXT DEFAULT 'Medium',
    status TEXT DEFAULT 'Discovered',
    deadline TEXT,
    discovered_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS claims (
    id TEXT PRIMARY KEY,
    opportunity_id TEXT NOT NULL REFERENCES opportunities(id),
    wallet_id TEXT NOT NULL REFERENCES wallets(id),
    tx_hash TEXT,
    chain TEXT NOT NULL,
    gas_cost_wei TEXT,
    gas_cost_usd REAL,
    value_received_usd REAL,
    status TEXT NOT NULL DEFAULT 'Pending',
    retry_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS scraper_state (
    source TEXT PRIMARY KEY,
    last_run TEXT,
    cursor TEXT,
    error_count INTEGER DEFAULT 0,
    last_error TEXT
);

CREATE INDEX IF NOT EXISTS idx_opportunities_chain ON opportunities(chain);
CREATE INDEX IF NOT EXISTS idx_opportunities_status ON opportunities(status);
CREATE INDEX IF NOT EXISTS idx_opportunities_score ON opportunities(harvest_score DESC);
CREATE INDEX IF NOT EXISTS idx_claims_status ON claims(status);
CREATE INDEX IF NOT EXISTS idx_wallets_chain ON wallets(chain);
"#;
