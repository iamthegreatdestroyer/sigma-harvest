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
