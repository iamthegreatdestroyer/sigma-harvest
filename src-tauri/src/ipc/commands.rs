//! Tauri IPC command handlers.
//! These are the bridge between the React frontend and Rust backend.
//! SECURITY: Private keys and seed material never cross the IPC boundary.

use crate::db::{self, DbState};
use crate::vault::derivation::{Chain, DerivedWallet};
use crate::vault::keystore::VaultState;
use crate::vault::VaultStatus;
use serde::{Deserialize, Serialize};
use tauri::State;

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
pub fn get_app_status(vault: State<'_, VaultState>) -> AppStatus {
    let keystore = vault.0.lock().unwrap();
    AppStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        vault_locked: keystore.is_locked(),
        pipeline_running: false,
        active_opportunities: 0,
        pending_claims: 0,
    }
}

/// Get the vault lock status.
#[tauri::command]
pub fn get_vault_status(vault: State<'_, VaultState>, db: State<'_, DbState>) -> VaultStatus {
    let keystore = vault.0.lock().unwrap();
    let conn = db.0.lock().unwrap();

    // Get actual wallet count from DB even when locked
    let wallet_count = db::wallet_count(&conn).unwrap_or(0);

    VaultStatus {
        locked: keystore.is_locked(),
        wallet_count,
        last_unlock: keystore.status().last_unlock.clone(),
    }
}

/// Create a new vault with a fresh mnemonic.
/// Returns the mnemonic phrase (user must back it up) and the first wallet address.
#[derive(Debug, Serialize)]
pub struct CreateWalletResult {
    pub mnemonic: String,
    pub first_address: String,
}

#[tauri::command]
pub fn create_wallet(
    passphrase: String,
    vault: State<'_, VaultState>,
    db: State<'_, DbState>,
) -> Result<CreateWalletResult, String> {
    if passphrase.len() < 8 {
        return Err("Passphrase must be at least 8 characters".to_string());
    }

    let mut keystore = vault.0.lock().unwrap();
    let conn = db.0.lock().unwrap();

    // Check if vault already exists
    if db::load_encrypted_seed(&conn).map_err(|e| e.to_string())?.is_some() {
        return Err("Vault already exists. Lock and re-create not yet supported.".to_string());
    }

    let (mnemonic, encrypted_seed) = keystore
        .create(&passphrase, 12)
        .map_err(|e| e.to_string())?;

    // Persist encrypted seed to database
    db::save_encrypted_seed(&conn, &encrypted_seed).map_err(|e| e.to_string())?;

    // Persist the first wallet
    let first_wallet = &keystore.wallets()[0];
    let wallet_id = uuid::Uuid::new_v4().to_string();
    db::save_wallet(
        &conn,
        &wallet_id,
        &first_wallet.path,
        &first_wallet.address,
        first_wallet.chain.name(),
        first_wallet.label.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    // Save the next derivation index
    db::save_config(&conn, "next_derivation_index", "1").map_err(|e| e.to_string())?;

    tracing::info!("New vault created with first wallet: {}", first_wallet.address);

    Ok(CreateWalletResult {
        mnemonic,
        first_address: first_wallet.address.clone(),
    })
}

/// Unlock the vault with a passphrase.
#[derive(Debug, Serialize)]
pub struct UnlockResult {
    pub success: bool,
    pub wallet_count: u32,
}

#[tauri::command]
pub fn unlock_vault(
    passphrase: String,
    vault: State<'_, VaultState>,
    db: State<'_, DbState>,
) -> Result<UnlockResult, String> {
    let mut keystore = vault.0.lock().unwrap();
    let conn = db.0.lock().unwrap();

    let encrypted_seed = db::load_encrypted_seed(&conn)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No vault found. Create one first.".to_string())?;

    // Restore the next derivation index
    let next_index = db::load_config(&conn, "next_derivation_index")
        .map_err(|e| e.to_string())?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1);
    keystore.set_next_index(next_index);

    keystore
        .unlock(&passphrase, &encrypted_seed)
        .map_err(|e| e.to_string())?;

    tracing::info!("Vault unlocked, {} wallets available", keystore.wallets().len());

    Ok(UnlockResult {
        success: true,
        wallet_count: keystore.wallets().len() as u32,
    })
}

/// Lock the vault.
#[tauri::command]
pub fn lock_vault(vault: State<'_, VaultState>) -> Result<bool, String> {
    let mut keystore = vault.0.lock().unwrap();
    keystore.lock();
    Ok(true)
}

/// List all wallets (public data only).
#[tauri::command]
pub fn list_wallets(vault: State<'_, VaultState>) -> Result<Vec<DerivedWallet>, String> {
    let keystore = vault.0.lock().unwrap();
    if keystore.is_locked() {
        return Err("Vault is locked".to_string());
    }
    Ok(keystore.wallets().to_vec())
}

/// Derive the next wallet for a given chain.
#[tauri::command]
pub fn derive_next_wallet(
    chain: String,
    vault: State<'_, VaultState>,
    db: State<'_, DbState>,
) -> Result<DerivedWallet, String> {
    let chain = match chain.to_lowercase().as_str() {
        "ethereum" => Chain::Ethereum,
        "arbitrum" => Chain::Arbitrum,
        "optimism" => Chain::Optimism,
        "base" => Chain::Base,
        "polygon" => Chain::Polygon,
        "zksync" => Chain::ZkSync,
        _ => return Err(format!("Unsupported chain: {}", chain)),
    };

    let mut keystore = vault.0.lock().unwrap();
    let conn = db.0.lock().unwrap();

    let wallet = keystore.derive_next(&chain).map_err(|e| e.to_string())?;

    // Persist the new wallet
    let wallet_id = uuid::Uuid::new_v4().to_string();
    db::save_wallet(
        &conn,
        &wallet_id,
        &wallet.path,
        &wallet.address,
        wallet.chain.name(),
        wallet.label.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    // Update next derivation index
    let next_index = wallet.index + 1;
    db::save_config(&conn, "next_derivation_index", &next_index.to_string())
        .map_err(|e| e.to_string())?;

    tracing::info!("Derived new {} wallet: {}", chain, wallet.address);

    Ok(wallet)
}

/// Check if a vault has been created (encrypted seed exists in DB).
#[tauri::command]
pub fn has_vault(db: State<'_, DbState>) -> Result<bool, String> {
    let conn = db.0.lock().unwrap();
    let exists = db::load_encrypted_seed(&conn)
        .map_err(|e| e.to_string())?
        .is_some();
    Ok(exists)
}
