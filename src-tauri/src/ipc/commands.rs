//! Tauri IPC command handlers.
//! These are the bridge between the React frontend and Rust backend.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppStatus {
    pub version: String,
    pub vault_locked: bool,
    pub pipeline_running: bool,
    pub active_opportunities: u32,
    pub pending_claims: u32,
}

/// Get the current application status.
#[tauri::command]
pub fn get_app_status() -> AppStatus {
    AppStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        vault_locked: true,
        pipeline_running: false,
        active_opportunities: 0,
        pending_claims: 0,
    }
}

/// Get the vault lock status.
#[tauri::command]
pub fn get_vault_status() -> crate::vault::VaultStatus {
    crate::vault::VaultStatus::default()
}
