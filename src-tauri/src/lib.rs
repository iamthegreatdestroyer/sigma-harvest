pub mod analytics;
pub mod db;
pub mod discovery;
pub mod evaluation;
pub mod executor;
pub mod ipc;
pub mod scraper;
pub mod vault;

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
        .invoke_handler(tauri::generate_handler![
            ipc::commands::get_app_status,
            ipc::commands::get_vault_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ΣHARVEST");
}
