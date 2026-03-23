pub mod schema;

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

/// Initialize the database with schema migrations.
pub fn initialize(db_path: &Path) -> Result<Connection, DbError> {
    let conn = Connection::open(db_path)?;

    // Enable WAL mode for concurrent reads
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    // Run migrations
    schema::run_migrations(&conn)?;

    tracing::info!("Database initialized at {:?}", db_path);
    Ok(conn)
}

/// Thread-safe database state for Tauri managed state.
pub struct DbState(pub Mutex<Connection>);

impl DbState {
    pub fn new(conn: Connection) -> Self {
        Self(Mutex::new(conn))
    }
}

/// Save the encrypted vault seed to the config table.
pub fn save_encrypted_seed(conn: &Connection, encrypted: &[u8]) -> Result<(), DbError> {
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, encrypted);
    conn.execute(
        "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
        rusqlite::params!["vault_seed", encoded],
    )?;
    Ok(())
}

/// Load the encrypted vault seed from the config table.
pub fn load_encrypted_seed(conn: &Connection) -> Result<Option<Vec<u8>>, DbError> {
    let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?1")?;
    let result: Result<String, _> = stmt.query_row(rusqlite::params!["vault_seed"], |row| row.get(0));

    match result {
        Ok(encoded) => {
            let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded)
                .map_err(|e| DbError::Migration(format!("base64 decode failed: {}", e)))?;
            Ok(Some(decoded))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DbError::Sqlite(e)),
    }
}

/// Save a derived wallet record.
pub fn save_wallet(
    conn: &Connection,
    id: &str,
    path: &str,
    address: &str,
    chain: &str,
    label: Option<&str>,
) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO wallets (id, derivation_path, public_address, chain, label) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, path, address, chain, label],
    )?;
    Ok(())
}

/// Load all wallet records from the database.
pub fn load_wallets(conn: &Connection) -> Result<Vec<WalletRecord>, DbError> {
    let mut stmt =
        conn.prepare("SELECT id, derivation_path, public_address, chain, label FROM wallets ORDER BY created_at")?;
    let wallets = stmt
        .query_map([], |row| {
            Ok(WalletRecord {
                id: row.get(0)?,
                derivation_path: row.get(1)?,
                public_address: row.get(2)?,
                chain: row.get(3)?,
                label: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(wallets)
}

/// Get the wallet count from the database.
pub fn wallet_count(conn: &Connection) -> Result<u32, DbError> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM wallets")?;
    let count: u32 = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

/// Save a config key-value pair.
pub fn save_config(conn: &Connection, key: &str, value: &str) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

/// Load a config value.
pub fn load_config(conn: &Connection, key: &str) -> Result<Option<String>, DbError> {
    let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?1")?;
    match stmt.query_row(rusqlite::params![key], |row| row.get::<_, String>(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DbError::Sqlite(e)),
    }
}

/// A wallet record from the database.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletRecord {
    pub id: String,
    pub derivation_path: String,
    pub public_address: String,
    pub chain: String,
    pub label: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("migration error: {0}")]
    Migration(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a temporary in-memory database for testing.
    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    // ── Initialization ────────────────────────────────────────════

    #[test]
    fn initialize_creates_tables() {
        let conn = test_db();
        // Check all 5 tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"wallets".to_string()));
        assert!(tables.contains(&"opportunities".to_string()));
        assert!(tables.contains(&"claims".to_string()));
        assert!(tables.contains(&"config".to_string()));
        assert!(tables.contains(&"scraper_state".to_string()));
    }

    #[test]
    fn migrations_are_idempotent() {
        let conn = test_db();
        // Running migrations again should not error
        schema::run_migrations(&conn).unwrap();
    }

    #[test]
    fn initialize_from_file() {
        let dir = std::env::temp_dir().join("sigma_test_db");
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test.db");
        let _ = std::fs::remove_file(&db_path);

        let conn = initialize(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM wallets", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);

        // Cleanup
        drop(conn);
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir(dir);
    }

    // ── Config CRUD ───────────────────────────────────────────════

    #[test]
    fn save_and_load_config() {
        let conn = test_db();
        save_config(&conn, "my_key", "my_value").unwrap();
        let value = load_config(&conn, "my_key").unwrap();
        assert_eq!(value, Some("my_value".to_string()));
    }

    #[test]
    fn load_missing_config_returns_none() {
        let conn = test_db();
        let value = load_config(&conn, "nonexistent").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn config_upsert_updates_value() {
        let conn = test_db();
        save_config(&conn, "key", "v1").unwrap();
        save_config(&conn, "key", "v2").unwrap();
        let value = load_config(&conn, "key").unwrap();
        assert_eq!(value, Some("v2".to_string()));
    }

    #[test]
    fn config_empty_value() {
        let conn = test_db();
        save_config(&conn, "key", "").unwrap();
        let value = load_config(&conn, "key").unwrap();
        assert_eq!(value, Some("".to_string()));
    }

    #[test]
    fn config_unicode_value() {
        let conn = test_db();
        save_config(&conn, "lang", "日本語テスト").unwrap();
        let value = load_config(&conn, "lang").unwrap();
        assert_eq!(value, Some("日本語テスト".to_string()));
    }

    // ── Encrypted seed storage ────────────────────────────────════

    #[test]
    fn save_and_load_encrypted_seed() {
        let conn = test_db();
        let seed_data = vec![0xABu8; 128]; // encrypted seed blob
        save_encrypted_seed(&conn, &seed_data).unwrap();
        let loaded = load_encrypted_seed(&conn).unwrap().unwrap();
        assert_eq!(loaded, seed_data);
    }

    #[test]
    fn load_encrypted_seed_when_none() {
        let conn = test_db();
        let loaded = load_encrypted_seed(&conn).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn encrypted_seed_upsert() {
        let conn = test_db();
        save_encrypted_seed(&conn, &[1, 2, 3]).unwrap();
        save_encrypted_seed(&conn, &[4, 5, 6]).unwrap();
        let loaded = load_encrypted_seed(&conn).unwrap().unwrap();
        assert_eq!(loaded, vec![4, 5, 6]);
    }

    #[test]
    fn encrypted_seed_large_blob() {
        let conn = test_db();
        let large = vec![0xFFu8; 100_000];
        save_encrypted_seed(&conn, &large).unwrap();
        let loaded = load_encrypted_seed(&conn).unwrap().unwrap();
        assert_eq!(loaded.len(), 100_000);
    }

    // ── Wallet CRUD ───────────────────────────────────────────════

    #[test]
    fn save_and_load_wallets() {
        let conn = test_db();
        save_wallet(&conn, "w1", "m/44'/60'/0'/0/0", "0xABC123", "ethereum", None).unwrap();
        let wallets = load_wallets(&conn).unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].public_address, "0xABC123");
        assert_eq!(wallets[0].chain, "ethereum");
    }

    #[test]
    fn save_wallet_with_label() {
        let conn = test_db();
        save_wallet(&conn, "w1", "m/44'/60'/0'/0/0", "0xABC", "ethereum", Some("Hot Wallet")).unwrap();
        let wallets = load_wallets(&conn).unwrap();
        assert_eq!(wallets[0].label, Some("Hot Wallet".to_string()));
    }

    #[test]
    fn wallet_count_empty() {
        let conn = test_db();
        assert_eq!(wallet_count(&conn).unwrap(), 0);
    }

    #[test]
    fn wallet_count_after_inserts() {
        let conn = test_db();
        save_wallet(&conn, "w1", "p1", "0xAAA", "ethereum", None).unwrap();
        save_wallet(&conn, "w2", "p2", "0xBBB", "arbitrum", None).unwrap();
        save_wallet(&conn, "w3", "p3", "0xCCC", "base", None).unwrap();
        assert_eq!(wallet_count(&conn).unwrap(), 3);
    }

    #[test]
    fn wallet_upsert_by_id() {
        let conn = test_db();
        save_wallet(&conn, "w1", "p1", "0xAAA", "ethereum", None).unwrap();
        save_wallet(&conn, "w1", "p1", "0xBBB", "arbitrum", None).unwrap();
        let wallets = load_wallets(&conn).unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].public_address, "0xBBB");
    }

    #[test]
    fn wallet_unique_address_replaces_on_conflict() {
        let conn = test_db();
        save_wallet(&conn, "w1", "p1", "0xSAME", "ethereum", None).unwrap();
        // INSERT OR REPLACE: same address with different ID replaces the old row
        save_wallet(&conn, "w2", "p2", "0xSAME", "arbitrum", None).unwrap();
        let wallets = load_wallets(&conn).unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].id, "w2"); // New ID replaced old
        assert_eq!(wallets[0].chain, "arbitrum");
    }

    #[test]
    fn load_wallets_ordered_by_created_at() {
        let conn = test_db();
        save_wallet(&conn, "w1", "p1", "0xAAA", "ethereum", None).unwrap();
        save_wallet(&conn, "w2", "p2", "0xBBB", "ethereum", None).unwrap();
        let wallets = load_wallets(&conn).unwrap();
        assert_eq!(wallets[0].id, "w1");
        assert_eq!(wallets[1].id, "w2");
    }

    #[test]
    fn wallet_record_serializable() {
        let record = WalletRecord {
            id: "test".to_string(),
            derivation_path: "m/44'/60'/0'/0/0".to_string(),
            public_address: "0xABC".to_string(),
            chain: "ethereum".to_string(),
            label: Some("Test".to_string()),
        };
        let json = serde_json::to_string(&record).unwrap();
        let deserialized: WalletRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test");
    }

    // ── Opportunities table ───────────────────────────────────════

    #[test]
    fn opportunities_table_exists() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO opportunities (id, source, chain, opportunity_type, title) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["op1", "rss", "ethereum", "Airdrop", "Test Airdrop"],
        ).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM opportunities", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn opportunities_default_values() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO opportunities (id, source, chain, opportunity_type, title) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["op1", "galxe", "arbitrum", "Quest", "Test Quest"],
        ).unwrap();
        let status: String = conn
            .query_row("SELECT status FROM opportunities WHERE id = 'op1'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(status, "Discovered");
        let score: i64 = conn
            .query_row("SELECT harvest_score FROM opportunities WHERE id = 'op1'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(score, 0);
    }

    // ── Claims table ──────────────────────────────────────────════

    #[test]
    fn claims_table_exists() {
        let conn = test_db();
        // Need wallet + opportunity first (foreign keys)
        save_wallet(&conn, "w1", "p1", "0xAAA", "ethereum", None).unwrap();
        conn.execute(
            "INSERT INTO opportunities (id, source, chain, opportunity_type, title) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["op1", "rss", "ethereum", "Airdrop", "Test"],
        ).unwrap();
        conn.execute(
            "INSERT INTO claims (id, opportunity_id, wallet_id, chain) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["c1", "op1", "w1", "ethereum"],
        ).unwrap();
        let status: String = conn
            .query_row("SELECT status FROM claims WHERE id = 'c1'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(status, "Pending");
    }

    // ── Scraper state table ───────────────────────────────────════

    #[test]
    fn scraper_state_table_exists() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO scraper_state (source) VALUES (?1)",
            rusqlite::params!["rss"],
        ).unwrap();
        let error_count: i64 = conn
            .query_row("SELECT error_count FROM scraper_state WHERE source = 'rss'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(error_count, 0);
    }

    // ── Indexes ───────────────────────────────────────────────════

    #[test]
    fn indexes_exist() {
        let conn = test_db();
        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(indexes.contains(&"idx_opportunities_chain".to_string()));
        assert!(indexes.contains(&"idx_opportunities_status".to_string()));
        assert!(indexes.contains(&"idx_opportunities_score".to_string()));
        assert!(indexes.contains(&"idx_claims_status".to_string()));
        assert!(indexes.contains(&"idx_wallets_chain".to_string()));
    }

    // ── Thread-safety ─────────────────────────────────────────════

    #[test]
    fn db_state_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DbState>();
    }
}
