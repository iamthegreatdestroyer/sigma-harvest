//! Transaction simulation via eth_call before real execution.
//! Ensures every claim is validated before spending gas.

use super::transaction::{
    build_simulation_params, build_estimate_gas_params, parse_simulation_response,
    PreparedTransaction, SimulationResult, TransactionError,
};
use crate::chain::provider::ChainClient;
use crate::chain::registry::SUPPORTED_CHAINS;
use serde::{Deserialize, Serialize};

/// Suspicious outcome flags detected during simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousFlags {
    /// Transaction simulated as a revert
    pub reverted: bool,
    /// Gas estimate is unusually high (> 500k)
    pub high_gas: bool,
    /// Gas estimate diverges significantly from oracle estimate
    pub gas_divergence: bool,
    /// Return data suggests an approval call (potential phishing)
    pub unexpected_approval: bool,
}

impl SuspiciousFlags {
    pub fn any_flagged(&self) -> bool {
        self.reverted || self.high_gas || self.gas_divergence || self.unexpected_approval
    }
}

/// Full simulation result with safety analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    pub simulation: SimulationResult,
    pub flags: SuspiciousFlags,
    pub safe_to_proceed: bool,
    pub gas_savings_pct: f64,
    pub message: String,
}

/// High gas threshold (500k gas units).
const HIGH_GAS_THRESHOLD: u64 = 500_000;
/// If estimated gas diverges from oracle by more than 50%, flag it.
const GAS_DIVERGENCE_THRESHOLD: f64 = 0.5;
/// ERC-20 approve function selector.
const APPROVE_SELECTOR: &str = "0x095ea7b3";
/// Unlimited approval value (2^256 - 1 prefix).
const UNLIMITED_APPROVAL_PREFIX: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

/// Simulate a prepared transaction via eth_call and estimate gas.
/// Returns a full report with safety flags.
///
/// This is the safety gate — if `safe_to_proceed` is false, the executor
/// MUST NOT broadcast the transaction.
pub async fn simulate_transaction(
    tx: &PreparedTransaction,
    client: &ChainClient,
    oracle_gas_estimate: Option<u64>,
) -> Result<SimulationReport, TransactionError> {
    let chain = SUPPORTED_CHAINS
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(&tx.chain))
        .ok_or_else(|| TransactionError::SimulationFailed(format!("unknown chain: {}", tx.chain)))?;

    // 1. eth_call simulation
    let call_params = build_simulation_params(tx);
    let call_result = client
        .rpc_call_public(chain, "eth_call", call_params)
        .await
        .map_err(|e| TransactionError::SimulationFailed(e.to_string()))?;

    // 2. eth_estimateGas
    let gas_params = build_estimate_gas_params(tx);
    let gas_result = client
        .rpc_call_public(chain, "eth_estimateGas", gas_params)
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| parse_hex_u64(s)));

    // 3. Parse results
    let simulation = parse_simulation_response(&call_result, gas_result);

    // 4. Analyze for suspicious patterns
    let flags = analyze_flags(tx, &simulation, oracle_gas_estimate);

    // 5. Determine safety
    let safe = !flags.reverted && !flags.unexpected_approval;

    let message = if flags.reverted {
        format!(
            "BLOCKED: Transaction would revert{}",
            simulation
                .revert_reason
                .as_ref()
                .map(|r| format!(": {}", r))
                .unwrap_or_default()
        )
    } else if flags.unexpected_approval {
        "BLOCKED: Detected unlimited token approval — potential phishing".to_string()
    } else if flags.high_gas {
        format!(
            "WARNING: Unusually high gas estimate ({})",
            simulation.estimated_gas
        )
    } else if flags.gas_divergence {
        "WARNING: Gas estimate diverges significantly from oracle".to_string()
    } else {
        "Simulation passed — safe to proceed".to_string()
    };

    let gas_savings_pct = if let Some(oracle) = oracle_gas_estimate {
        if oracle > 0 {
            ((oracle as f64 - simulation.estimated_gas as f64) / oracle as f64) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    Ok(SimulationReport {
        simulation,
        flags,
        safe_to_proceed: safe,
        gas_savings_pct,
        message,
    })
}

/// Analyze a simulation result for suspicious patterns.
fn analyze_flags(
    tx: &PreparedTransaction,
    sim: &SimulationResult,
    oracle_gas: Option<u64>,
) -> SuspiciousFlags {
    let reverted = !sim.success;
    let high_gas = sim.estimated_gas > HIGH_GAS_THRESHOLD;

    let gas_divergence = if let Some(oracle) = oracle_gas {
        if oracle > 0 {
            let ratio = (sim.estimated_gas as f64 - oracle as f64).abs() / oracle as f64;
            ratio > GAS_DIVERGENCE_THRESHOLD
        } else {
            false
        }
    } else {
        false
    };

    // Check for unlimited approve() calls
    let unexpected_approval = tx.data.starts_with(APPROVE_SELECTOR)
        && tx
            .data
            .to_lowercase()
            .contains(UNLIMITED_APPROVAL_PREFIX);

    SuspiciousFlags {
        reverted,
        high_gas,
        gas_divergence,
        unexpected_approval,
    }
}

/// Parse a hex string to u64 for gas values.
fn parse_hex_u64(hex: &str) -> u64 {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    u64::from_str_radix(hex, 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(data: &str) -> PreparedTransaction {
        PreparedTransaction {
            chain: "ethereum".to_string(),
            chain_id: 1,
            from: "0xABC".to_string(),
            to: "0xDEF".to_string(),
            value_wei: "0x0".to_string(),
            data: data.to_string(),
            nonce: 0,
            max_fee_per_gas_gwei: 30.0,
            max_priority_fee_gwei: 2.0,
            gas_limit: 100_000,
        }
    }

    fn make_sim(success: bool, gas: u64) -> SimulationResult {
        SimulationResult {
            success,
            return_data: if success { "0x01" } else { "0x" }.to_string(),
            revert_reason: if success { None } else { Some("out of gas".to_string()) },
            estimated_gas: gas,
        }
    }

    #[test]
    fn flags_clean_transaction() {
        let tx = make_tx("0xa9059cbb");
        let sim = make_sim(true, 50_000);
        let flags = analyze_flags(&tx, &sim, Some(55_000));
        assert!(!flags.any_flagged());
    }

    #[test]
    fn flags_reverted_transaction() {
        let tx = make_tx("0xa9059cbb");
        let sim = make_sim(false, 21_000);
        let flags = analyze_flags(&tx, &sim, None);
        assert!(flags.reverted);
        assert!(flags.any_flagged());
    }

    #[test]
    fn flags_high_gas() {
        let tx = make_tx("0xa9059cbb");
        let sim = make_sim(true, 600_000);
        let flags = analyze_flags(&tx, &sim, None);
        assert!(flags.high_gas);
        assert!(flags.any_flagged());
    }

    #[test]
    fn flags_gas_divergence() {
        let tx = make_tx("0xa9059cbb");
        let sim = make_sim(true, 200_000);
        let flags = analyze_flags(&tx, &sim, Some(100_000));
        assert!(flags.gas_divergence);
    }

    #[test]
    fn flags_no_divergence_within_threshold() {
        let tx = make_tx("0xa9059cbb");
        let sim = make_sim(true, 110_000);
        let flags = analyze_flags(&tx, &sim, Some(100_000));
        assert!(!flags.gas_divergence);
    }

    #[test]
    fn flags_unlimited_approval() {
        let data = format!(
            "0x095ea7b3000000000000000000000000spender{}",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        );
        let tx = make_tx(&data);
        let sim = make_sim(true, 50_000);
        let flags = analyze_flags(&tx, &sim, None);
        assert!(flags.unexpected_approval);
        assert!(flags.any_flagged());
    }

    #[test]
    fn flags_normal_approval_not_flagged() {
        let data = "0x095ea7b3000000000000000000000000spender0000000000000000000000000000000000000000000000000de0b6b3a7640000";
        let tx = make_tx(data);
        let sim = make_sim(true, 50_000);
        let flags = analyze_flags(&tx, &sim, None);
        assert!(!flags.unexpected_approval);
    }

    #[test]
    fn suspicious_flags_serializable() {
        let flags = SuspiciousFlags {
            reverted: true,
            high_gas: false,
            gas_divergence: false,
            unexpected_approval: false,
        };
        let json = serde_json::to_string(&flags).unwrap();
        let roundtrip: SuspiciousFlags = serde_json::from_str(&json).unwrap();
        assert!(roundtrip.reverted);
    }

    #[test]
    fn simulation_report_serializable() {
        let report = SimulationReport {
            simulation: make_sim(true, 50_000),
            flags: SuspiciousFlags {
                reverted: false,
                high_gas: false,
                gas_divergence: false,
                unexpected_approval: false,
            },
            safe_to_proceed: true,
            gas_savings_pct: 5.0,
            message: "ok".to_string(),
        };
        let json = serde_json::to_string(&report).unwrap();
        let roundtrip: SimulationReport = serde_json::from_str(&json).unwrap();
        assert!(roundtrip.safe_to_proceed);
    }

    #[test]
    fn parse_hex_u64_values() {
        assert_eq!(parse_hex_u64("0x0"), 0);
        assert_eq!(parse_hex_u64("0x5208"), 21000);
        assert_eq!(parse_hex_u64("0x7a120"), 500_000);
        assert_eq!(parse_hex_u64("ff"), 255);
    }
}
