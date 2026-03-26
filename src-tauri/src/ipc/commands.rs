//! Tauri IPC command handlers.
//! These are the bridge between the React frontend and Rust backend.
//! SECURITY: Private keys and seed material never cross the IPC boundary.

use crate::chain::provider::{AddressBalance, GasPriceResult};
use crate::chain::registry::{ChainConfig, SUPPORTED_CHAINS};
use crate::core::sigma::compression::LogEntry;
use crate::core::sigma::dynamics::EcosystemState;
use crate::core::sigma::memory::SimilarityResult;
use crate::core::sigma::swarm::{ConsensusResult, SwarmSummary};
use crate::core::sigma::vectors::encode_opportunity;
use crate::core::sigma::SigmaCoreStatus;
use crate::db::{self, DbState};
use crate::vault::derivation::{Chain, DerivedWallet};
use crate::vault::keystore::VaultState;
use crate::vault::VaultStatus;
use crate::{ChainClientState, SigmaCoreState};
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

// ── Chain Connectivity Commands ─────────────────────────────────────────────

/// Get native token balances for an address across all supported chains.
#[tauri::command]
pub async fn get_balances(
    address: String,
    chain_client: State<'_, ChainClientState>,
) -> Result<Vec<AddressBalance>, String> {
    let results = chain_client.0.get_all_balances(&address).await;
    let mut balances = Vec::new();
    for result in results {
        match result {
            Ok(balance) => balances.push(balance),
            Err(e) => {
                tracing::warn!("Failed to fetch balance: {}", e);
                // Continue fetching other chains
            }
        }
    }
    Ok(balances)
}

/// Get current gas prices for all supported chains.
#[tauri::command]
pub async fn get_gas_prices(
    chain_client: State<'_, ChainClientState>,
) -> Result<Vec<GasPriceResult>, String> {
    let results = chain_client.0.get_all_gas_prices().await;
    let mut prices = Vec::new();
    for result in results {
        match result {
            Ok(price) => prices.push(price),
            Err(e) => {
                tracing::warn!("Failed to fetch gas price: {}", e);
            }
        }
    }
    Ok(prices)
}

/// Get the configuration for all supported chains.
#[tauri::command]
pub fn get_chain_configs() -> Vec<ChainConfig> {
    SUPPORTED_CHAINS.clone()
}

// ── ΣCORE Nervous System Commands ───────────────────────────────────────────

/// Get the ΣCORE status — memory, swarm, compression metrics.
#[tauri::command]
pub fn get_sigma_status(sigma: State<'_, SigmaCoreState>) -> Result<SigmaCoreStatus, String> {
    let memory = sigma.memory.lock().map_err(|e| e.to_string())?;
    let swarm = sigma.swarm.lock().map_err(|e| e.to_string())?;
    let compression = sigma.compression.lock().map_err(|e| e.to_string())?;
    let dynamics_on = *sigma.dynamics_enabled.lock().map_err(|e| e.to_string())?;

    Ok(SigmaCoreStatus {
        memory_entries: memory.len(),
        memory_bytes: memory.memory_bytes(),
        active_agents: swarm.active_count(),
        attractor_strength: memory.attractor_strength(),
        compression_ratio: compression.overall_compression_ratio(),
        dynamics_enabled: dynamics_on,
    })
}

/// Get swarm performance summary.
#[tauri::command]
pub fn get_swarm_summary(sigma: State<'_, SigmaCoreState>) -> Result<SwarmSummary, String> {
    let swarm = sigma.swarm.lock().map_err(|e| e.to_string())?;
    Ok(swarm.performance_summary())
}

/// Calculate wave score for an opportunity given its ecosystem state.
#[tauri::command]
pub fn sigma_wave_score(
    gas_pressure: f64,
    community_signal: f64,
    deadline_urgency: f64,
    value_estimate: f64,
    sigma: State<'_, SigmaCoreState>,
) -> Result<f64, String> {
    let state = EcosystemState::new(gas_pressure, community_signal, deadline_urgency, value_estimate);
    Ok(sigma.dynamics.wave_score(&state))
}

// ── ΣCORE Mutation Commands ─────────────────────────────────────────────────

/// Encode an opportunity into an HD vector and store it in associative memory.
/// Returns the number of similar opportunities found (cosine > 0.3).
#[tauri::command]
pub fn sigma_encode_and_store(
    chain: String,
    opportunity_type: String,
    risk_level: String,
    label: String,
    tags: Vec<String>,
    sigma: State<'_, SigmaCoreState>,
) -> Result<usize, String> {
    let mut codebook = sigma.codebook.lock().map_err(|e| e.to_string())?;
    let vec = encode_opportunity(&mut codebook, &chain, &opportunity_type, &risk_level);
    let mut memory = sigma.memory.lock().map_err(|e| e.to_string())?;

    // Check for similar existing entries before storing
    let similar = memory.query_threshold(&vec, 0.3);
    let similar_count = similar.len();

    memory.store(label, vec, tags);
    Ok(similar_count)
}

/// Query associative memory for similar opportunities.
#[tauri::command]
pub fn sigma_memory_query(
    chain: String,
    opportunity_type: String,
    risk_level: String,
    k: usize,
    sigma: State<'_, SigmaCoreState>,
) -> Result<Vec<SimilarityResult>, String> {
    let mut codebook = sigma.codebook.lock().map_err(|e| e.to_string())?;
    let vec = encode_opportunity(&mut codebook, &chain, &opportunity_type, &risk_level);
    let mut memory = sigma.memory.lock().map_err(|e| e.to_string())?;
    Ok(memory.query(&vec, k))
}

/// Reinforce a memory entry (strengthen its attractor pull).
#[tauri::command]
pub fn sigma_memory_reinforce(
    label: String,
    sigma: State<'_, SigmaCoreState>,
) -> Result<f64, String> {
    let mut memory = sigma.memory.lock().map_err(|e| e.to_string())?;
    memory.reinforce(&label);
    Ok(memory.attractor_strength())
}

/// Evict stale entries older than max_age_secs with fewer than min_reinforcement hits.
#[tauri::command]
pub fn sigma_memory_evict(
    max_age_secs: u64,
    min_reinforcement: u32,
    sigma: State<'_, SigmaCoreState>,
) -> Result<usize, String> {
    let mut memory = sigma.memory.lock().map_err(|e| e.to_string())?;
    let before = memory.len();
    memory.evict_stale(max_age_secs, min_reinforcement);
    Ok(before - memory.len())
}

/// List all memory entry labels.
#[tauri::command]
pub fn sigma_memory_labels(
    sigma: State<'_, SigmaCoreState>,
) -> Result<Vec<String>, String> {
    let memory = sigma.memory.lock().map_err(|e| e.to_string())?;
    Ok(memory.labels().into_iter().map(|s| s.to_string()).collect())
}

/// Run swarm consensus vote on an opportunity.
#[tauri::command]
pub fn sigma_swarm_vote(
    chain: String,
    opportunity_type: String,
    risk_level: String,
    sigma: State<'_, SigmaCoreState>,
) -> Result<ConsensusResult, String> {
    let mut codebook = sigma.codebook.lock().map_err(|e| e.to_string())?;
    let vec = encode_opportunity(&mut codebook, &chain, &opportunity_type, &risk_level);
    let swarm = sigma.swarm.lock().map_err(|e| e.to_string())?;
    Ok(swarm.consensus_vote(&vec))
}

/// Trigger evolutionary step on the swarm (replace worst with mutated best).
#[tauri::command]
pub fn sigma_swarm_evolve(
    sigma: State<'_, SigmaCoreState>,
) -> Result<SwarmSummary, String> {
    let mut swarm = sigma.swarm.lock().map_err(|e| e.to_string())?;
    swarm.evolve();
    Ok(swarm.performance_summary())
}

/// Set the swarm mutation rate.
#[tauri::command]
pub fn sigma_swarm_set_mutation_rate(
    rate: f64,
    sigma: State<'_, SigmaCoreState>,
) -> Result<f64, String> {
    let clamped = rate.clamp(0.0, 1.0);
    let mut swarm = sigma.swarm.lock().map_err(|e| e.to_string())?;
    swarm.mutation_rate = clamped;
    Ok(clamped)
}

/// Push a log entry into the ΣLANG compression pipeline.
#[tauri::command]
pub fn sigma_compression_push(
    source: String,
    level: String,
    message: String,
    sigma: State<'_, SigmaCoreState>,
) -> Result<f64, String> {
    let mut compression = sigma.compression.lock().map_err(|e| e.to_string())?;
    compression.push(LogEntry::new(&source, &level, &message));
    Ok(compression.overall_compression_ratio())
}

/// Search compressed logs for a query string. Returns (batch_index, similarity) tuples.
#[tauri::command]
pub fn sigma_compression_search(
    query: String,
    top_k: usize,
    sigma: State<'_, SigmaCoreState>,
) -> Result<Vec<(usize, f64)>, String> {
    let compression = sigma.compression.lock().map_err(|e| e.to_string())?;
    Ok(compression.search(&query, top_k))
}

/// Calculate Hurst exponent for a time series (e.g., gas prices, opportunity values).
#[tauri::command]
pub fn sigma_hurst_analysis(
    series: Vec<f64>,
) -> Result<HurstResult, String> {
    use crate::core::sigma::dynamics::{hurst_exponent, hurst_regime};
    let h = hurst_exponent(&series).ok_or_else(|| "Series too short for R/S analysis".to_string())?;
    Ok(HurstResult {
        exponent: h,
        regime: hurst_regime(h).to_string(),
    })
}

/// Hurst exponent analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HurstResult {
    pub exponent: f64,
    pub regime: String,
}

// ── ΣCORE Pipeline Orchestrator ─────────────────────────────────────────────

/// Full opportunity pipeline: encode → dedup check → memory store → swarm vote
/// → wave score → combined ΣSCORE. This is the nervous system's "think" operation.
#[tauri::command]
pub fn sigma_evaluate_opportunity(
    chain: String,
    opportunity_type: String,
    risk_level: String,
    label: String,
    tags: Vec<String>,
    gas_pressure: f64,
    community_signal: f64,
    deadline_urgency: f64,
    value_estimate: f64,
    sigma: State<'_, SigmaCoreState>,
) -> Result<OpportunityEvaluation, String> {
    // 1. Encode opportunity into HD vector
    let mut codebook = sigma.codebook.lock().map_err(|e| e.to_string())?;
    let vec = encode_opportunity(&mut codebook, &chain, &opportunity_type, &risk_level);
    drop(codebook);

    // 2. Check memory for duplicates / similar past opportunities
    let mut memory = sigma.memory.lock().map_err(|e| e.to_string())?;
    let similar = memory.query_threshold(&vec, 0.3);
    let duplicate = similar.iter().any(|s| s.similarity > 0.95);
    let attractor_score = memory.attractor_score(&vec);

    // Store if not a duplicate
    if !duplicate {
        memory.store(label.clone(), vec.clone(), tags);
    }
    drop(memory);

    // 3. Swarm consensus vote
    let swarm = sigma.swarm.lock().map_err(|e| e.to_string())?;
    let consensus = swarm.consensus_vote(&vec);
    drop(swarm);

    // 4. Non-linear dynamics wave score
    let dynamics_on = *sigma.dynamics_enabled.lock().map_err(|e| e.to_string())?;
    let ecosystem = EcosystemState::new(gas_pressure, community_signal, deadline_urgency, value_estimate);
    let wave = if dynamics_on {
        sigma.dynamics.wave_score(&ecosystem)
    } else {
        0.5 // neutral
    };

    // 5. Combine into final ΣSCORE
    // Weights: attractor 25%, consensus 35%, wave 25%, value 15%
    let value_norm = (value_estimate / 100.0).clamp(0.0, 1.0);
    let consensus_norm = (consensus.score + 1.0) / 2.0; // map [-1,1] to [0,1]
    let sigma_score = attractor_score * 0.25
        + consensus_norm * 0.35
        + wave * 0.25
        + value_norm * 0.15;

    let should_proceed = sigma_score >= 0.5 && consensus.proceed && !duplicate;
    let consensus_score_raw = consensus.score;

    // 6. Log to compression pipeline
    let mut compression = sigma.compression.lock().map_err(|e| e.to_string())?;
    compression.push(LogEntry::new(
        "pipeline",
        if sigma_score >= 0.6 { "info" } else { "debug" },
        &format!(
            "{}|{}|{}|σ={:.3}|wave={:.3}|cons={:.3}",
            chain, opportunity_type, label, sigma_score, wave, consensus_score_raw
        ),
    ));

    Ok(OpportunityEvaluation {
        label,
        sigma_score,
        attractor_score,
        consensus,
        wave_score: wave,
        duplicate,
        similar_count: similar.len(),
        proceed: should_proceed,
    })
}

/// Full evaluation result from the ΣCORE pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityEvaluation {
    pub label: String,
    pub sigma_score: f64,
    pub attractor_score: f64,
    pub consensus: ConsensusResult,
    pub wave_score: f64,
    pub duplicate: bool,
    pub similar_count: usize,
    pub proceed: bool,
}

/// Record outcome feedback — updates swarm agent and memory reinforcement.
/// Call this after a claim succeeds or fails.
#[tauri::command]
pub fn sigma_record_outcome(
    label: String,
    chain: String,
    opportunity_type: String,
    risk_level: String,
    success: bool,
    sigma: State<'_, SigmaCoreState>,
) -> Result<SwarmSummary, String> {
    // 1. Encode the opportunity vector
    let mut codebook = sigma.codebook.lock().map_err(|e| e.to_string())?;
    let vec = encode_opportunity(&mut codebook, &chain, &opportunity_type, &risk_level);
    drop(codebook);

    // 2. Find the best agent for this opportunity and record outcome
    let mut swarm = sigma.swarm.lock().map_err(|e| e.to_string())?;
    if let Some(idx) = swarm.best_agent_index(&vec) {
        if success {
            swarm.agents[idx].record_success(Some(&vec));
        } else {
            swarm.agents[idx].record_failure();
        }
    }

    // 3. Reinforce memory if successful
    if success {
        let mut memory = sigma.memory.lock().map_err(|e| e.to_string())?;
        memory.reinforce(&label);
    }

    // 4. Log to compression
    let mut compression = sigma.compression.lock().map_err(|e| e.to_string())?;
    compression.push(LogEntry::new(
        "feedback",
        if success { "info" } else { "warn" },
        &format!("outcome|{}|{}|{}", label, chain, if success { "success" } else { "failure" }),
    ));

    Ok(swarm.performance_summary())
}

/// Toggle the non-linear dynamics engine on/off.
#[tauri::command]
pub fn sigma_toggle_dynamics(
    enabled: bool,
    sigma: State<'_, SigmaCoreState>,
) -> Result<bool, String> {
    let mut flag = sigma.dynamics_enabled.lock().map_err(|e| e.to_string())?;
    *flag = enabled;
    Ok(*flag)
}

// ── DISCOVERY + EVALUATION PIPELINE ────────────────────────────

/// Run discovery across selected sources. Returns raw opportunities.
/// Sources: "rss", "dappradar", "galxe", "onchain", "social"
#[tauri::command]
pub async fn discover_opportunities(
    sources: Vec<String>,
    rss_feeds: Option<Vec<String>>,
    dappradar_key: Option<String>,
) -> Result<Vec<crate::discovery::RawOpportunity>, String> {
    use crate::discovery::DiscoverySource;

    let mut all_opportunities = Vec::new();

    for source_name in &sources {
        let result = match source_name.as_str() {
            "rss" => {
                let feeds = rss_feeds.clone().unwrap_or_else(|| {
                    vec![
                        "https://rss.app/feeds/v1.1/crypto-airdrops.xml".to_string(),
                    ]
                });
                let source = crate::discovery::rss::RssSource { feed_urls: feeds };
                source.discover().await
            }
            "dappradar" => {
                let key = dappradar_key
                    .clone()
                    .or_else(|| std::env::var("DAPPRADAR_API_KEY").ok());
                let source = crate::discovery::dappradar::DappRadarSource {
                    api_key: key,
                };
                source.discover().await
            }
            "galxe" => {
                let source = crate::discovery::galxe::GalxeSource;
                source.discover().await
            }
            "onchain" => {
                let rpc = std::env::var("ETHEREUM_RPC_URL")
                    .unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());
                let source = crate::discovery::onchain::OnChainSource {
                    rpc_url: rpc,
                    chain: "ethereum".to_string(),
                    watchlist: vec![],
                    lookback_blocks: 100,
                };
                source.discover().await
            }
            "social" => {
                let bearer = std::env::var("TWITTER_BEARER_TOKEN").ok();
                let source = crate::discovery::social::SocialSource {
                    bearer_token: bearer,
                };
                source.discover().await
            }
            unknown => {
                tracing::warn!("Unknown discovery source: {}", unknown);
                continue;
            }
        };

        match result {
            Ok(mut opps) => {
                tracing::info!("Discovered {} opportunities from {}", opps.len(), source_name);
                all_opportunities.append(&mut opps);
            }
            Err(e) => {
                tracing::warn!("Discovery source {} failed: {}", source_name, e);
            }
        }
    }

    Ok(all_opportunities)
}

/// Evaluate a single raw opportunity through the full pipeline:
/// HarvestScore → Risk Assessment → ΣCORE integration.
#[tauri::command]
pub fn evaluate_full_pipeline(
    opportunity: crate::discovery::RawOpportunity,
    sigma: State<'_, SigmaCoreState>,
) -> Result<FullEvaluation, String> {
    use crate::evaluation::{harvest_score, risk};

    // 1. Harvest Score calculation
    let score_result = harvest_score::calculate(&opportunity);

    // 2. Risk assessment
    let risk_assessment = risk::assess_opportunity(&opportunity);

    // 3. ΣCORE integration — encode + evaluate
    let risk_level_str = format!("{:?}", risk_assessment.level);

    let mut codebook = sigma.codebook.lock().map_err(|e| e.to_string())?;
    let vec = encode_opportunity(&mut codebook, &opportunity.chain, &opportunity.source, &risk_level_str);

    let mut memory = sigma.memory.lock().map_err(|e| e.to_string())?;
    let similar = memory.query_threshold(&vec, 0.3);
    let duplicate = similar.iter().any(|s| s.similarity > 0.95);
    let attractor_score = memory.attractor_score(&vec);

    let label = opportunity.title.clone();
    let tags = vec![opportunity.chain.clone(), opportunity.source.clone()];
    if !duplicate {
        memory.store(label.clone(), vec.clone(), tags);
    }
    drop(memory);

    // Swarm consensus
    let swarm = sigma.swarm.lock().map_err(|e| e.to_string())?;
    let consensus = swarm.consensus_vote(&vec);
    drop(swarm);

    // Dynamics wave
    let dynamics_on = *sigma.dynamics_enabled.lock().map_err(|e| e.to_string())?;
    let value = opportunity.estimated_value_usd.unwrap_or(0.0);
    let gas = opportunity.gas_cost_estimate.unwrap_or(1.0);
    let ecosystem = EcosystemState::new(
        gas.min(100.0) / 100.0,           // gas_pressure
        score_result.breakdown.community_size as f64 / 10.0, // community_signal
        score_result.breakdown.time_urgency as f64 / 15.0,   // deadline_urgency
        value.min(100.0) / 100.0,         // value_estimate
    );
    let wave = if dynamics_on {
        sigma.dynamics.wave_score(&ecosystem)
    } else {
        0.5
    };

    // Combined ΣSCORE
    let value_norm = (value / 100.0).clamp(0.0, 1.0);
    let harvest_norm = score_result.score as f64 / 100.0;
    let consensus_norm = (consensus.score + 1.0) / 2.0;

    // Blended: harvest 30%, attractor 20%, consensus 25%, wave 15%, value 10%
    let sigma_score = harvest_norm * 0.30
        + attractor_score * 0.20
        + consensus_norm * 0.25
        + wave * 0.15
        + value_norm * 0.10;

    let is_critical_risk = matches!(
        risk_assessment.level,
        crate::evaluation::risk::RiskLevel::Critical
    );
    let proceed = sigma_score >= 0.4 && consensus.proceed && !duplicate && !is_critical_risk;

    // Determine evaluation status
    let status = if is_critical_risk {
        crate::evaluation::OpportunityStatus::Rejected
    } else if proceed {
        crate::evaluation::OpportunityStatus::Qualified
    } else {
        crate::evaluation::OpportunityStatus::Discovered
    };

    // Log to compression pipeline
    let mut compression = sigma.compression.lock().map_err(|e| e.to_string())?;
    compression.push(LogEntry::new(
        "hunt",
        if proceed { "info" } else { "debug" },
        &format!(
            "{}|harvest={}|σ={:.3}|risk={:?}|{}",
            label,
            score_result.score,
            sigma_score,
            risk_assessment.level,
            if proceed { "PROCEED" } else { "HOLD" }
        ),
    ));

    Ok(FullEvaluation {
        id: uuid::Uuid::new_v4().to_string(),
        title: opportunity.title,
        chain: opportunity.chain,
        source: opportunity.source,
        harvest_score: score_result,
        risk: risk_assessment,
        sigma_score,
        attractor_score,
        wave_score: wave,
        consensus,
        duplicate,
        similar_count: similar.len(),
        proceed,
        status,
        estimated_value_usd: opportunity.estimated_value_usd,
        gas_cost_estimate: opportunity.gas_cost_estimate,
        url: opportunity.url,
    })
}

/// Full evaluation result combining HarvestScore + Risk + ΣCORE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullEvaluation {
    pub id: String,
    pub title: String,
    pub chain: String,
    pub source: String,
    pub harvest_score: crate::evaluation::harvest_score::HarvestScoreResult,
    pub risk: crate::evaluation::risk::RiskAssessment,
    pub sigma_score: f64,
    pub attractor_score: f64,
    pub wave_score: f64,
    pub consensus: ConsensusResult,
    pub duplicate: bool,
    pub similar_count: usize,
    pub proceed: bool,
    pub status: crate::evaluation::OpportunityStatus,
    pub estimated_value_usd: Option<f64>,
    pub gas_cost_estimate: Option<f64>,
    pub url: Option<String>,
}

/// Run a full hunt cycle: discover → evaluate all → return sorted results.
#[tauri::command]
pub async fn run_hunt_cycle(
    sources: Vec<String>,
    rss_feeds: Option<Vec<String>>,
    dappradar_key: Option<String>,
    sigma: State<'_, SigmaCoreState>,
) -> Result<HuntCycleResult, String> {
    let start = std::time::Instant::now();

    // 1. Discover
    let raw_opps = discover_opportunities(sources.clone(), rss_feeds, dappradar_key).await?;

    // 2. Evaluate each through the full pipeline
    let mut evaluations = Vec::new();
    for opp in raw_opps {
        match evaluate_single_opportunity(&opp, &sigma) {
            Ok(eval) => evaluations.push(eval),
            Err(e) => tracing::warn!("Evaluation failed for {}: {}", opp.title, e),
        }
    }

    // 3. Sort by sigma_score descending
    evaluations.sort_by(|a, b| b.sigma_score.partial_cmp(&a.sigma_score).unwrap_or(std::cmp::Ordering::Equal));

    let qualified = evaluations.iter().filter(|e| e.proceed).count();
    let duplicates = evaluations.iter().filter(|e| e.duplicate).count();
    let duration_ms = start.elapsed().as_millis() as u64;

    // Log cycle summary
    let mut compression = sigma.compression.lock().map_err(|e| e.to_string())?;
    compression.push(LogEntry::new(
        "hunt",
        "info",
        &format!(
            "cycle|sources={:?}|discovered={}|qualified={}|dupes={}|{}ms",
            sources,
            evaluations.len(),
            qualified,
            duplicates,
            duration_ms
        ),
    ));

    Ok(HuntCycleResult {
        total_discovered: evaluations.len(),
        qualified,
        duplicates,
        evaluations,
        duration_ms,
    })
}

/// Hunt cycle result containing all evaluations and summary stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntCycleResult {
    pub total_discovered: usize,
    pub qualified: usize,
    pub duplicates: usize,
    pub evaluations: Vec<FullEvaluation>,
    pub duration_ms: u64,
}

/// Internal: evaluate a single opportunity (non-command helper).
fn evaluate_single_opportunity(
    opp: &crate::discovery::RawOpportunity,
    sigma: &SigmaCoreState,
) -> Result<FullEvaluation, String> {
    use crate::evaluation::{harvest_score, risk};

    let score_result = harvest_score::calculate(opp);
    let risk_assessment = risk::assess_opportunity(opp);

    let risk_level_str = format!("{:?}", risk_assessment.level);

    let mut codebook = sigma.codebook.lock().map_err(|e| e.to_string())?;
    let vec = encode_opportunity(&mut codebook, &opp.chain, &opp.source, &risk_level_str);

    let mut memory = sigma.memory.lock().map_err(|e| e.to_string())?;
    let similar = memory.query_threshold(&vec, 0.3);
    let duplicate = similar.iter().any(|s| s.similarity > 0.95);
    let attractor_score = memory.attractor_score(&vec);

    let label = opp.title.clone();
    let tags = vec![opp.chain.clone(), opp.source.clone()];
    if !duplicate {
        memory.store(label.clone(), vec.clone(), tags);
    }
    drop(memory);

    let swarm = sigma.swarm.lock().map_err(|e| e.to_string())?;
    let consensus = swarm.consensus_vote(&vec);
    drop(swarm);

    let dynamics_on = *sigma.dynamics_enabled.lock().map_err(|e| e.to_string())?;
    let value = opp.estimated_value_usd.unwrap_or(0.0);
    let gas = opp.gas_cost_estimate.unwrap_or(1.0);
    let ecosystem = EcosystemState::new(
        gas.min(100.0) / 100.0,
        score_result.breakdown.community_size as f64 / 10.0,
        score_result.breakdown.time_urgency as f64 / 15.0,
        value.min(100.0) / 100.0,
    );
    let wave = if dynamics_on {
        sigma.dynamics.wave_score(&ecosystem)
    } else {
        0.5
    };

    let value_norm = (value / 100.0).clamp(0.0, 1.0);
    let harvest_norm = score_result.score as f64 / 100.0;
    let consensus_norm = (consensus.score + 1.0) / 2.0;

    let sigma_score = harvest_norm * 0.30
        + attractor_score * 0.20
        + consensus_norm * 0.25
        + wave * 0.15
        + value_norm * 0.10;

    let is_critical = matches!(risk_assessment.level, crate::evaluation::risk::RiskLevel::Critical);
    let proceed = sigma_score >= 0.4 && consensus.proceed && !duplicate && !is_critical;

    let status = if is_critical {
        crate::evaluation::OpportunityStatus::Rejected
    } else if proceed {
        crate::evaluation::OpportunityStatus::Qualified
    } else {
        crate::evaluation::OpportunityStatus::Discovered
    };

    Ok(FullEvaluation {
        id: uuid::Uuid::new_v4().to_string(),
        title: opp.title.clone(),
        chain: opp.chain.clone(),
        source: opp.source.clone(),
        harvest_score: score_result,
        risk: risk_assessment,
        sigma_score,
        attractor_score,
        wave_score: wave,
        consensus,
        duplicate,
        similar_count: similar.len(),
        proceed,
        status,
        estimated_value_usd: opp.estimated_value_usd,
        gas_cost_estimate: opp.gas_cost_estimate,
        url: opp.url.clone(),
    })
}

// ═══════════════════════════════════════════════════════════
// ── Analytics IPC Commands ────────────────────────────────
// ═══════════════════════════════════════════════════════════

/// Get the current analytics summary (ROI, claims, etc.).
#[tauri::command]
pub fn get_analytics_summary(
    db: State<'_, DbState>,
) -> Result<crate::analytics::AnalyticsSummary, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(crate::analytics::reports::generate_summary(&conn))
}

/// Get source attribution — which discovery sources produce the most value.
#[tauri::command]
pub fn get_source_attribution(
    db: State<'_, DbState>,
) -> Result<Vec<crate::analytics::SourceAttribution>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(crate::analytics::reports::source_attribution(&conn))
}

/// Get chain breakdown — claim stats per chain.
#[tauri::command]
pub fn get_chain_breakdown(
    db: State<'_, DbState>,
) -> Result<Vec<crate::analytics::ChainBreakdown>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(crate::analytics::reports::chain_breakdown(&conn))
}

// ═══════════════════════════════════════════════════════════
// ── Executor IPC Commands ─────────────────────────────────
// ═══════════════════════════════════════════════════════════

/// Evaluate gas conditions for a chain before claiming.
#[tauri::command]
pub async fn check_gas_conditions(
    chain: String,
    ceiling_gwei: f64,
    estimated_gas_usd: f64,
    client: State<'_, ChainClientState>,
) -> Result<crate::executor::gas_oracle::GasDecision, String> {
    let price = crate::executor::gas_oracle::fetch_gas_price(&chain, &client.0)
        .await
        .map_err(|e| e.to_string())?;
    let tracker = crate::executor::gas_oracle::SpendingTracker::new(50.0);
    Ok(crate::executor::gas_oracle::evaluate_gas_conditions(
        &price,
        ceiling_gwei,
        &tracker,
        estimated_gas_usd,
    ))
}

/// Process a batch of claims through the execution pipeline.
#[tauri::command]
pub fn process_claim_batch(
    mut claims: Vec<crate::executor::ClaimOperation>,
    gas_ok: bool,
    simulation_ok: bool,
) -> Result<crate::executor::ExecutionBatchResult, String> {
    Ok(crate::executor::process_batch(&mut claims, gas_ok, simulation_ok))
}

// ═══════════════════════════════════════════════════════════
// ── Config IPC Commands ───────────────────────────────────
// ═══════════════════════════════════════════════════════════

/// Get a configuration value by key.
#[tauri::command]
pub fn get_config(
    key: String,
    db: State<'_, DbState>,
) -> Result<Option<String>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::load_config(&conn, &key).map_err(|e| e.to_string())
}

/// Set a configuration value.
#[tauri::command]
pub fn set_config(
    key: String,
    value: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::save_config(&conn, &key, &value).map_err(|e| e.to_string())
}

/// Get all configuration key-value pairs.
#[tauri::command]
pub fn get_all_config(
    db: State<'_, DbState>,
) -> Result<Vec<(String, String)>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::load_all_config(&conn).map_err(|e| e.to_string())
}

// ═══════════════════════════════════════════════════════════
// ── Simulation IPC Commands ───────────────────────────────
// ═══════════════════════════════════════════════════════════

/// Simulate a claim transaction via eth_call before real execution.
/// Returns a safety report — the frontend should block if safe_to_proceed is false.
#[tauri::command]
pub async fn simulate_claim(
    chain: String,
    chain_id: u64,
    from: String,
    to: String,
    calldata: String,
    nonce: u64,
    max_fee_gwei: f64,
    max_priority_gwei: f64,
    gas_limit: u64,
    oracle_gas_estimate: Option<u64>,
    client: State<'_, ChainClientState>,
) -> Result<crate::executor::simulation::SimulationReport, String> {
    let tx = crate::executor::transaction::build_claim_transaction(
        &chain,
        chain_id,
        &from,
        &to,
        &calldata,
        nonce,
        max_fee_gwei,
        max_priority_gwei,
        gas_limit,
    )
    .map_err(|e| e.to_string())?;

    crate::executor::simulation::simulate_transaction(&tx, &client.0, oracle_gas_estimate)
        .await
        .map_err(|e| e.to_string())
}

// ═══════════════════════════════════════════════════════════
// ── Consolidation IPC Commands ────────────────────────────
// ═══════════════════════════════════════════════════════════

/// Plan a token consolidation sweep (dry run — no transactions sent).
/// Scans all wallets for native + ERC-20 balances and determines which are worth sweeping.
#[tauri::command]
pub async fn plan_consolidation(
    destination: String,
    chain: String,
    min_native_wei: u128,
    min_erc20_units: u128,
    max_gas_gwei: f64,
    erc20_tokens: Vec<String>,
    gas_price_gwei: f64,
    vault: State<'_, VaultState>,
    client: State<'_, ChainClientState>,
) -> Result<crate::executor::consolidation::ConsolidationPlan, String> {
    // Extract wallet addresses synchronously before any await
    let wallet_addresses = {
        let keystore = vault.0.lock().map_err(|e| e.to_string())?;
        if keystore.is_locked() {
            return Err("Vault is locked".to_string());
        }
        keystore.wallets().iter().map(|w| w.address.clone()).collect::<Vec<String>>()
    };

    let config = crate::executor::consolidation::ConsolidationConfig {
        destination,
        chain,
        min_native_wei,
        min_erc20_units,
        max_gas_gwei,
        erc20_tokens,
    };

    crate::executor::consolidation::plan_consolidation(&config, &wallet_addresses, &client.0, gas_price_gwei)
        .await
        .map_err(|e| e.to_string())
}

/// Record a consolidation sweep transaction into the claims table.
#[tauri::command]
pub fn record_consolidation_sweep(
    wallet_id: String,
    chain: String,
    tx_hash: Option<String>,
    gas_cost_wei: Option<String>,
    gas_cost_usd: Option<f64>,
    value_received_usd: Option<f64>,
    db: State<'_, DbState>,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    crate::db::log_consolidation_sweep(
        &conn,
        &id,
        &wallet_id,
        &chain,
        tx_hash.as_deref(),
        gas_cost_wei.as_deref(),
        gas_cost_usd,
        value_received_usd,
    )
    .map_err(|e| e.to_string())?;
    Ok(id)
}

// ═══════════════════════════════════════════════════════════
// ── Price IPC Commands ────────────────────────────────────
// ═══════════════════════════════════════════════════════════

/// Get token prices for all supported native gas tokens (ETH, MATIC).
/// Results are cached for 5 minutes.
#[tauri::command]
pub async fn get_token_prices(
    price_client: State<'_, crate::PriceClientState>,
) -> Result<crate::chain::coingecko::PriceResponse, String> {
    price_client.0.get_native_prices().await.map_err(|e| e.to_string())
}

/// Get the USD price for a specific chain's native token.
#[tauri::command]
pub async fn get_chain_price_usd(
    chain: String,
    price_client: State<'_, crate::PriceClientState>,
) -> Result<f64, String> {
    price_client.0.get_chain_price(&chain).await.map_err(|e| e.to_string())
}

// ═══════════════════════════════════════════════════════════
// ── Time-Series Analytics ─────────────────────────────────
// ═══════════════════════════════════════════════════════════

/// Get daily time-series claim data for sparkline charts.
/// Returns data points for the specified number of days.
#[tauri::command]
pub fn get_time_series(
    days: u32,
    db: State<'_, DbState>,
) -> Result<Vec<crate::analytics::TimeSeriesPoint>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(crate::analytics::reports::time_series(&conn, days))
}
