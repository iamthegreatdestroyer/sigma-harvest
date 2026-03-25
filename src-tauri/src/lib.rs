pub mod analytics;
pub mod chain;
pub mod core;
pub mod db;
pub mod discovery;
pub mod evaluation;
pub mod executor;
pub mod ipc;
pub mod scraper;
pub mod vault;

use std::sync::Mutex;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

/// Load .env.local and .env files for API keys and RPC overrides.
fn load_env_files() {
    // .env.local takes priority
    let _ = dotenvy::from_filename(".env.local");
    let _ = dotenvy::dotenv();
}

/// Shared state wrapper for the chain RPC client.
pub struct ChainClientState(pub chain::ChainClient);

/// Shared state wrapper for the CoinGecko price client.
pub struct PriceClientState(pub chain::PriceClient);

/// Shared state for the ΣCORE nervous system.
pub struct SigmaCoreState {
    pub memory: Mutex<core::sigma::memory::AssociativeMemory>,
    pub swarm: Mutex<core::sigma::swarm::Swarm>,
    pub dynamics: core::sigma::dynamics::DynamicsEngine,
    pub compression: Mutex<core::sigma::compression::CompressionPipeline>,
    pub codebook: Mutex<core::sigma::vectors::Codebook>,
    pub dynamics_enabled: Mutex<bool>,
}

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

    // Load environment variables from .env.local / .env
    load_env_files();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
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

            // Register chain client state (4 concurrent RPC calls max for free tiers)
            app.manage(ChainClientState(chain::ChainClient::new(4)));

            // Register CoinGecko price client
            let coingecko_key = std::env::var("COINGECKO_API_KEY").ok();
            app.manage(PriceClientState(chain::PriceClient::new(coingecko_key)));

            // Initialize ΣCORE nervous system
            app.manage(SigmaCoreState {
                memory: Mutex::new(core::sigma::memory::AssociativeMemory::default_dim()),
                swarm: Mutex::new(core::sigma::swarm::Swarm::default_harvest()),
                dynamics: core::sigma::dynamics::DynamicsEngine::new(),
                compression: Mutex::new(core::sigma::compression::CompressionPipeline::new(50)),
                codebook: Mutex::new(core::sigma::vectors::Codebook::new(256)),
                dynamics_enabled: Mutex::new(true),
            });

            tracing::info!("ΣCORE nervous system initialized (8 swarm agents, dim=256)");
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
            ipc::commands::get_balances,
            ipc::commands::get_gas_prices,
            ipc::commands::get_chain_configs,
            // ΣCORE read-only
            ipc::commands::get_sigma_status,
            ipc::commands::get_swarm_summary,
            ipc::commands::sigma_wave_score,
            // ΣCORE mutations
            ipc::commands::sigma_encode_and_store,
            ipc::commands::sigma_memory_query,
            ipc::commands::sigma_memory_reinforce,
            ipc::commands::sigma_memory_evict,
            ipc::commands::sigma_memory_labels,
            ipc::commands::sigma_swarm_vote,
            ipc::commands::sigma_swarm_evolve,
            ipc::commands::sigma_swarm_set_mutation_rate,
            ipc::commands::sigma_compression_push,
            ipc::commands::sigma_compression_search,
            ipc::commands::sigma_hurst_analysis,
            // ΣCORE orchestration
            ipc::commands::sigma_evaluate_opportunity,
            ipc::commands::sigma_record_outcome,
            ipc::commands::sigma_toggle_dynamics,
            // Discovery + Evaluation Pipeline
            ipc::commands::discover_opportunities,
            ipc::commands::evaluate_full_pipeline,
            ipc::commands::run_hunt_cycle,
            // Analytics
            ipc::commands::get_analytics_summary,
            ipc::commands::get_source_attribution,
            ipc::commands::get_chain_breakdown,
            // Executor
            ipc::commands::check_gas_conditions,
            ipc::commands::process_claim_batch,
            // Config
            ipc::commands::get_config,
            ipc::commands::set_config,
            ipc::commands::get_all_config,
            // Simulation
            ipc::commands::simulate_claim,
            // Consolidation
            ipc::commands::plan_consolidation,
            // Prices
            ipc::commands::get_token_prices,
            ipc::commands::get_chain_price_usd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ΣHARVEST");
}
