pub mod analytics;
pub mod db;
pub mod discovery;
pub mod evaluation;
pub mod executor;
pub mod ipc;
pub mod scraper;
pub mod vault;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

/// Initialize the ΣHARVEST Tauri application.
pub fn run() {
    // Set up structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("sigma_harvest_lib=debug,tauri=info")),
        )
        .init();

    tracing::info!("ΣHARVEST v{} starting up", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize database in the app data directory
            let app_data = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            std::fs::create_dir_all(&app_data).expect("failed to create app data directory");

            let db_path = app_data.join("sigma-harvest.db");
            let conn = db::initialize(&db_path).expect("failed to initialize database");

            // Register database state
            app.manage(db::DbState::new(conn));

            // Register vault state
            app.manage(vault::keystore::VaultState::new());

            tracing::info!("Application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::get_app_status,
            ipc::commands::get_vault_status,
            ipc::commands::create_wallet,
            ipc::commands::unlock_vault,
            ipc::commands::lock_vault,
            ipc::commands::list_wallets,
            ipc::commands::derive_next_wallet,
            ipc::commands::has_vault,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ΣHARVEST");
}
