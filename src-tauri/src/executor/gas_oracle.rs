//! Gas price monitoring across supported chains.
//! Delegates real RPC calls to ChainClient and adds ceiling/cap tracking.

use crate::chain::provider::{ChainClient, GasPriceResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPrice {
    pub chain: String,
    pub base_fee_gwei: f64,
    pub priority_fee_gwei: f64,
    pub timestamp: String,
}

impl From<GasPriceResult> for GasPrice {
    fn from(r: GasPriceResult) -> Self {
        Self {
            chain: r.chain,
            base_fee_gwei: r.base_fee_gwei,
            priority_fee_gwei: r.priority_fee_gwei,
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

/// Fetch current gas prices for a chain via ChainClient.
pub async fn fetch_gas_price(chain: &str, client: &ChainClient) -> Result<GasPrice, GasOracleError> {
    let result = client
        .get_gas_price(chain)
        .await
        .map_err(|e| GasOracleError::RpcError(e.to_string()))?;
    Ok(GasPrice::from(result))
}

/// Check if current gas is below the configured ceiling.
pub fn gas_below_ceiling(current: &GasPrice, ceiling_gwei: f64) -> bool {
    current.base_fee_gwei + current.priority_fee_gwei <= ceiling_gwei
}

/// Tracks cumulative gas spending per chain for daily/weekly cap enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingTracker {
    /// Map of chain → cumulative gas cost in USD for the current period.
    pub daily_spend: HashMap<String, f64>,
    pub daily_cap_usd: f64,
    pub last_reset: String,
}

impl SpendingTracker {
    pub fn new(daily_cap_usd: f64) -> Self {
        Self {
            daily_spend: HashMap::new(),
            daily_cap_usd,
            last_reset: Utc::now().format("%Y-%m-%d").to_string(),
        }
    }

    /// Check if spending a given amount would exceed the daily cap.
    pub fn would_exceed_cap(&self, chain: &str, additional_usd: f64) -> bool {
        let current = self.daily_spend.get(chain).copied().unwrap_or(0.0);
        current + additional_usd > self.daily_cap_usd
    }

    /// Record a gas expenditure.
    pub fn record_spend(&mut self, chain: &str, amount_usd: f64) {
        *self.daily_spend.entry(chain.to_string()).or_insert(0.0) += amount_usd;
    }

    /// Reset if the day has changed.
    pub fn maybe_reset(&mut self) {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        if today != self.last_reset {
            self.daily_spend.clear();
            self.last_reset = today;
        }
    }

    /// Total spend across all chains.
    pub fn total_spend(&self) -> f64 {
        self.daily_spend.values().sum()
    }
}

/// Decision result from the gas oracle for a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasDecision {
    pub chain: String,
    pub current_gwei: f64,
    pub ceiling_gwei: f64,
    pub below_ceiling: bool,
    pub within_cap: bool,
    pub proceed: bool,
    pub reason: String,
}

/// Evaluate whether a claim should proceed based on gas + spending caps.
pub fn evaluate_gas_conditions(
    price: &GasPrice,
    ceiling_gwei: f64,
    tracker: &SpendingTracker,
    estimated_gas_usd: f64,
) -> GasDecision {
    let total_gwei = price.base_fee_gwei + price.priority_fee_gwei;
    let below = gas_below_ceiling(price, ceiling_gwei);
    let within_cap = !tracker.would_exceed_cap(&price.chain, estimated_gas_usd);
    let proceed = below && within_cap;

    let reason = if !below {
        format!("gas {:.1} gwei > ceiling {:.1} gwei", total_gwei, ceiling_gwei)
    } else if !within_cap {
        format!("would exceed daily cap of ${:.2}", tracker.daily_cap_usd)
    } else {
        "gas conditions met".to_string()
    };

    GasDecision {
        chain: price.chain.clone(),
        current_gwei: total_gwei,
        ceiling_gwei,
        below_ceiling: below,
        within_cap,
        proceed,
        reason,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GasOracleError {
    #[error("RPC error: {0}")]
    RpcError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_price(chain: &str, base: f64, priority: f64) -> GasPrice {
        GasPrice {
            chain: chain.to_string(),
            base_fee_gwei: base,
            priority_fee_gwei: priority,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn gas_below_ceiling_true() {
        let price = make_price("ethereum", 15.0, 2.0);
        assert!(gas_below_ceiling(&price, 30.0));
    }

    #[test]
    fn gas_below_ceiling_false() {
        let price = make_price("ethereum", 25.0, 10.0);
        assert!(!gas_below_ceiling(&price, 30.0));
    }

    #[test]
    fn gas_exactly_at_ceiling() {
        let price = make_price("ethereum", 28.0, 2.0);
        assert!(gas_below_ceiling(&price, 30.0));
    }

    #[test]
    fn gas_price_serializable() {
        let price = make_price("arbitrum", 0.1, 0.01);
        let json = serde_json::to_string(&price).unwrap();
        let roundtrip: GasPrice = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.chain, "arbitrum");
    }

    #[test]
    fn gas_oracle_error_display() {
        let err = GasOracleError::RpcError("timeout".to_string());
        assert!(format!("{}", err).contains("timeout"));
    }

    // ── SpendingTracker ──────────────────────────────────

    #[test]
    fn spending_tracker_new_starts_empty() {
        let tracker = SpendingTracker::new(50.0);
        assert_eq!(tracker.total_spend(), 0.0);
        assert_eq!(tracker.daily_cap_usd, 50.0);
    }

    #[test]
    fn spending_tracker_record_and_check() {
        let mut tracker = SpendingTracker::new(10.0);
        tracker.record_spend("ethereum", 5.0);
        assert!(!tracker.would_exceed_cap("ethereum", 4.0));
        assert!(tracker.would_exceed_cap("ethereum", 6.0));
    }

    #[test]
    fn spending_tracker_separate_chains() {
        let mut tracker = SpendingTracker::new(10.0);
        tracker.record_spend("ethereum", 8.0);
        assert!(!tracker.would_exceed_cap("arbitrum", 8.0));
    }

    #[test]
    fn spending_tracker_total_spend() {
        let mut tracker = SpendingTracker::new(100.0);
        tracker.record_spend("ethereum", 5.0);
        tracker.record_spend("arbitrum", 3.0);
        assert!((tracker.total_spend() - 8.0).abs() < f64::EPSILON);
    }

    // ── GasDecision ──────────────────────────────────────

    #[test]
    fn evaluate_gas_conditions_all_clear() {
        let price = make_price("ethereum", 15.0, 2.0);
        let tracker = SpendingTracker::new(50.0);
        let decision = evaluate_gas_conditions(&price, 30.0, &tracker, 2.0);
        assert!(decision.proceed);
        assert!(decision.below_ceiling);
        assert!(decision.within_cap);
    }

    #[test]
    fn evaluate_gas_conditions_above_ceiling() {
        let price = make_price("ethereum", 25.0, 10.0);
        let tracker = SpendingTracker::new(50.0);
        let decision = evaluate_gas_conditions(&price, 30.0, &tracker, 2.0);
        assert!(!decision.proceed);
        assert!(!decision.below_ceiling);
        assert!(decision.reason.contains("ceiling"));
    }

    #[test]
    fn evaluate_gas_conditions_exceeds_cap() {
        let price = make_price("ethereum", 15.0, 2.0);
        let mut tracker = SpendingTracker::new(10.0);
        tracker.record_spend("ethereum", 9.0);
        let decision = evaluate_gas_conditions(&price, 30.0, &tracker, 2.0);
        assert!(!decision.proceed);
        assert!(decision.below_ceiling);
        assert!(!decision.within_cap);
        assert!(decision.reason.contains("cap"));
    }

    #[test]
    fn gas_decision_serializable() {
        let decision = GasDecision {
            chain: "ethereum".to_string(),
            current_gwei: 17.0,
            ceiling_gwei: 30.0,
            below_ceiling: true,
            within_cap: true,
            proceed: true,
            reason: "ok".to_string(),
        };
        let json = serde_json::to_string(&decision).unwrap();
        let rt: GasDecision = serde_json::from_str(&json).unwrap();
        assert!(rt.proceed);
    }
}
