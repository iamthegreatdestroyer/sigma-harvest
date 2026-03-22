pub mod schema;

use rusqlite::Connection;
use std::path::Path;

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

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("migration error: {0}")]
    Migration(String),
}
