pub mod consolidation;
pub mod gas_oracle;
pub mod queue;
pub mod simulation;
pub mod transaction;

use serde::{Deserialize, Serialize};

use simulation::SimulationReport;

/// Claim strategy determines how the claim is executed and whether
/// the simulation gate is applied.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaimStrategy {
    /// Direct on-chain contract call — simulation gate applies.
    SimpleClaim,
    /// Multi-step claim (approve + claim) — simulation gate applies.
    MultiStep,
    /// Browser-based claim (dApp interaction) — has its own flow, no gate.
    BrowserClaim,
}

impl ClaimStrategy {
    /// Returns `true` when this strategy requires the simulation gate.
    pub fn requires_simulation_gate(&self) -> bool {
        matches!(self, ClaimStrategy::SimpleClaim | ClaimStrategy::MultiStep)
    }
}

/// Status of a claim execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaimStatus {
    Pending,
    Simulating,
    WaitingForGas,
    Executing,
    Confirmed { tx_hash: String },
    Failed { reason: String },
    /// Simulation gate rejected the transaction before broadcast.
    SimulationFailed { reason: String },
}

/// A claim operation in the execution pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimOperation {
    pub id: String,
    pub opportunity_id: String,
    pub wallet_address: String,
    pub chain: String,
    pub contract_address: Option<String>,
    pub calldata: Option<String>,
    pub status: ClaimStatus,
    pub gas_limit: Option<u64>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub harvest_score: u32,
    pub strategy: ClaimStrategy,
    /// Human-readable message from the last simulation run.
    pub simulation_message: Option<String>,
}

/// Result summary from processing a single claim through the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimResult {
    pub claim_id: String,
    pub opportunity_id: String,
    pub status: ClaimStatus,
    pub tx_hash: Option<String>,
    pub gas_used_gwei: Option<f64>,
    pub simulation_ok: bool,
    pub retries: u32,
}

/// Summary of a batch claim execution run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionBatchResult {
    pub total_processed: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped_gas: usize,
    pub results: Vec<ClaimResult>,
}

/// Apply the simulation safety gate using a full [`SimulationReport`].
///
/// * For `SimpleClaim` and `MultiStep` strategies the report's
///   `safe_to_proceed` flag determines whether the claim may advance
///   to signing/broadcast.
/// * For `BrowserClaim` the gate is always open (returns `true`).
///
/// On rejection the claim's status is set to [`ClaimStatus::SimulationFailed`]
/// and the human-readable message is stored in `simulation_message`.
pub fn apply_simulation_gate(
    claim: &mut ClaimOperation,
    report: &SimulationReport,
) -> bool {
    if !claim.strategy.requires_simulation_gate() {
        tracing::debug!(
            claim_id = %claim.id,
            strategy = ?claim.strategy,
            "simulation gate bypassed — strategy does not require it"
        );
        return true;
    }

    claim.simulation_message = Some(report.message.clone());

    if report.safe_to_proceed {
        tracing::info!(
            claim_id = %claim.id,
            strategy = ?claim.strategy,
            gas_savings_pct = report.gas_savings_pct,
            "simulation gate PASSED"
        );
        true
    } else {
        tracing::warn!(
            claim_id = %claim.id,
            strategy = ?claim.strategy,
            reason = %report.message,
            reverted = report.flags.reverted,
            "simulation gate REJECTED — will not broadcast"
        );
        claim.status = ClaimStatus::SimulationFailed {
            reason: report.message.clone(),
        };
        false
    }
}

/// Process a single claim through the pipeline stages.
///
/// Pipeline: VALIDATE → CHECK_GAS → SIMULATE (gated) → EXECUTE → RECORD
///
/// The simulation gate is only enforced for `SimpleClaim` and `MultiStep`
/// strategies.  `BrowserClaim` bypasses the gate entirely.
///
/// Returns the updated claim and result, without performing real RPC.
/// Real RPC calls are delegated to the ChainClient at the IPC layer.
pub fn process_claim_step(
    claim: &mut ClaimOperation,
    gas_ok: bool,
    simulation_ok: bool,
) -> ClaimResult {
    // Stage 1: Validate prerequisite data
    if claim.contract_address.is_none() && claim.calldata.is_none() {
        claim.status = ClaimStatus::Failed {
            reason: "no contract address or calldata".to_string(),
        };
        return make_result(claim, false);
    }

    // Stage 2: Gas check
    if !gas_ok {
        claim.status = ClaimStatus::WaitingForGas;
        return make_result(claim, false);
    }

    // Stage 3: Simulation gate
    match claim.strategy {
        // BrowserClaim has its own flow — bypass the simulation gate entirely
        ClaimStrategy::BrowserClaim => {
            tracing::debug!(
                claim_id = %claim.id,
                "BrowserClaim — bypassing simulation gate"
            );
        }
        // SimpleClaim / MultiStep — enforce the gate
        ClaimStrategy::SimpleClaim | ClaimStrategy::MultiStep => {
            claim.status = ClaimStatus::Simulating;
            if !simulation_ok {
                let reason = claim
                    .simulation_message
                    .clone()
                    .unwrap_or_else(|| "transaction simulation failed".to_string());
                tracing::warn!(
                    claim_id = %claim.id,
                    strategy = ?claim.strategy,
                    reason = %reason,
                    "simulation gate REJECTED — will not broadcast"
                );
                claim.status = ClaimStatus::SimulationFailed {
                    reason,
                };
                return make_result(claim, false);
            }
            tracing::info!(
                claim_id = %claim.id,
                strategy = ?claim.strategy,
                "simulation gate passed — proceeding to execution"
            );
        }
    }

    // Stage 4: Execute (mark as executing — real broadcast happens at IPC layer)
    claim.status = ClaimStatus::Executing;
    make_result(claim, true)
}

/// Mark a claim as confirmed with a tx hash.
pub fn confirm_claim(claim: &mut ClaimOperation, tx_hash: &str) {
    claim.status = ClaimStatus::Confirmed {
        tx_hash: tx_hash.to_string(),
    };
}

/// Mark a claim as failed.
pub fn fail_claim(claim: &mut ClaimOperation, reason: &str) {
    claim.status = ClaimStatus::Failed {
        reason: reason.to_string(),
    };
}

/// Check if a claim can be retried.
pub fn can_retry(claim: &ClaimOperation) -> bool {
    claim.retry_count < claim.max_retries
        && !matches!(
            claim.status,
            ClaimStatus::Confirmed { .. } | ClaimStatus::SimulationFailed { .. }
        )
}

/// Process a batch of claims from the queue.
pub fn process_batch(
    claims: &mut [ClaimOperation],
    gas_ok: bool,
    simulation_ok: bool,
) -> ExecutionBatchResult {
    let mut results = Vec::new();
    let mut succeeded = 0;
    let mut failed = 0;
    let mut skipped_gas = 0;

    for claim in claims.iter_mut() {
        let result = process_claim_step(claim, gas_ok, simulation_ok);
        match &claim.status {
            ClaimStatus::Executing | ClaimStatus::Confirmed { .. } => succeeded += 1,
            ClaimStatus::WaitingForGas => skipped_gas += 1,
            ClaimStatus::Failed { .. } | ClaimStatus::SimulationFailed { .. } => failed += 1,
            _ => {}
        }
        results.push(result);
    }

    ExecutionBatchResult {
        total_processed: results.len(),
        succeeded,
        failed,
        skipped_gas,
        results,
    }
}

fn make_result(claim: &ClaimOperation, simulation_ok: bool) -> ClaimResult {
    let tx_hash = match &claim.status {
        ClaimStatus::Confirmed { tx_hash } => Some(tx_hash.clone()),
        _ => None,
    };
    ClaimResult {
        claim_id: claim.id.clone(),
        opportunity_id: claim.opportunity_id.clone(),
        status: claim.status.clone(),
        tx_hash,
        gas_used_gwei: None,
        simulation_ok,
        retries: claim.retry_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::{SimulationReport, SuspiciousFlags};
    use transaction::SimulationResult;

    fn make_claim(id: &str, has_contract: bool) -> ClaimOperation {
        ClaimOperation {
            id: id.to_string(),
            opportunity_id: format!("opp-{}", id),
            wallet_address: "0xABC".to_string(),
            chain: "ethereum".to_string(),
            contract_address: if has_contract { Some("0xDef".to_string()) } else { None },
            calldata: if has_contract { Some("0xa9059cbb".to_string()) } else { None },
            status: ClaimStatus::Pending,
            gas_limit: Some(21000),
            retry_count: 0,
            max_retries: 3,
            harvest_score: 80,
            strategy: ClaimStrategy::SimpleClaim,
            simulation_message: None,
        }
    }

    fn make_claim_with_strategy(id: &str, strategy: ClaimStrategy) -> ClaimOperation {
        let mut claim = make_claim(id, true);
        claim.strategy = strategy;
        claim
    }

    fn passing_report() -> SimulationReport {
        SimulationReport {
            simulation: SimulationResult {
                success: true,
                return_data: "0x01".to_string(),
                revert_reason: None,
                estimated_gas: 50_000,
            },
            flags: SuspiciousFlags {
                reverted: false,
                high_gas: false,
                gas_divergence: false,
                unexpected_approval: false,
            },
            safe_to_proceed: true,
            gas_savings_pct: 5.0,
            message: "Simulation passed — safe to proceed".to_string(),
        }
    }

    fn failing_report(reason: &str) -> SimulationReport {
        SimulationReport {
            simulation: SimulationResult {
                success: false,
                return_data: "0x".to_string(),
                revert_reason: Some(reason.to_string()),
                estimated_gas: 21_000,
            },
            flags: SuspiciousFlags {
                reverted: true,
                high_gas: false,
                gas_divergence: false,
                unexpected_approval: false,
            },
            safe_to_proceed: false,
            gas_savings_pct: 0.0,
            message: format!("BLOCKED: Transaction would revert: {}", reason),
        }
    }

    // ── Serialization tests ─────────────────────────────────

    #[test]
    fn claim_status_serializable() {
        let statuses = vec![
            ClaimStatus::Pending,
            ClaimStatus::Simulating,
            ClaimStatus::WaitingForGas,
            ClaimStatus::Executing,
            ClaimStatus::Confirmed { tx_hash: "0xabc".to_string() },
            ClaimStatus::Failed { reason: "gas too high".to_string() },
            ClaimStatus::SimulationFailed { reason: "reverted".to_string() },
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let roundtrip: ClaimStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(serde_json::to_string(&roundtrip).unwrap(), json);
        }
    }

    #[test]
    fn claim_operation_serializable() {
        let op = make_claim("c1", true);
        let json = serde_json::to_string(&op).unwrap();
        let roundtrip: ClaimOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.id, "c1");
    }

    // ── Pipeline tests ───────────────────────────────────

    #[test]
    fn process_claim_no_contract_fails() {
        let mut claim = make_claim("1", false);
        claim.contract_address = None;
        claim.calldata = None;
        let result = process_claim_step(&mut claim, true, true);
        assert!(matches!(claim.status, ClaimStatus::Failed { .. }));
        assert!(!result.simulation_ok);
    }

    #[test]
    fn process_claim_gas_not_ok_waits() {
        let mut claim = make_claim("1", true);
        let result = process_claim_step(&mut claim, false, true);
        assert_eq!(claim.status, ClaimStatus::WaitingForGas);
        assert!(!result.simulation_ok);
    }

    #[test]
    fn process_claim_simulation_fails_sets_simulation_failed() {
        // SimpleClaim: simulation failure is a hard rejection (SimulationFailed)
        let mut claim = make_claim("1", true);
        let result = process_claim_step(&mut claim, true, false);
        assert!(matches!(claim.status, ClaimStatus::SimulationFailed { .. }));
        assert!(!result.simulation_ok);
    }

    #[test]
    fn process_claim_multistep_simulation_fails() {
        let mut claim = make_claim_with_strategy("1", ClaimStrategy::MultiStep);
        let result = process_claim_step(&mut claim, true, false);
        assert!(matches!(claim.status, ClaimStatus::SimulationFailed { .. }));
        assert!(!result.simulation_ok);
    }

    #[test]
    fn process_claim_success_path() {
        let mut claim = make_claim("1", true);
        let result = process_claim_step(&mut claim, true, true);
        assert_eq!(claim.status, ClaimStatus::Executing);
        assert!(result.simulation_ok);
    }

    #[test]
    fn confirm_claim_sets_hash() {
        let mut claim = make_claim("1", true);
        confirm_claim(&mut claim, "0xdeadbeef");
        assert!(matches!(claim.status, ClaimStatus::Confirmed { ref tx_hash } if tx_hash == "0xdeadbeef"));
    }

    #[test]
    fn fail_claim_sets_reason() {
        let mut claim = make_claim("1", true);
        fail_claim(&mut claim, "timeout");
        assert!(matches!(claim.status, ClaimStatus::Failed { ref reason } if reason == "timeout"));
    }

    #[test]
    fn can_retry_respects_max() {
        let mut claim = make_claim("1", true);
        assert!(can_retry(&claim));
        claim.retry_count = 3;
        assert!(!can_retry(&claim));
    }

    #[test]
    fn can_retry_false_when_confirmed() {
        let mut claim = make_claim("1", true);
        confirm_claim(&mut claim, "0x123");
        assert!(!can_retry(&claim));
    }

    #[test]
    fn can_retry_false_when_simulation_failed() {
        let mut claim = make_claim("1", true);
        claim.status = ClaimStatus::SimulationFailed {
            reason: "reverted".to_string(),
        };
        assert!(!can_retry(&claim));
    }

    // ── Batch processing ─────────────────────────────────

    #[test]
    fn process_batch_all_succeed() {
        let mut claims: Vec<ClaimOperation> = (0..3).map(|i| make_claim(&i.to_string(), true)).collect();
        let batch = process_batch(&mut claims, true, true);
        assert_eq!(batch.total_processed, 3);
        assert_eq!(batch.succeeded, 3);
        assert_eq!(batch.failed, 0);
        assert_eq!(batch.skipped_gas, 0);
    }

    #[test]
    fn process_batch_gas_wait() {
        let mut claims = vec![make_claim("1", true)];
        let batch = process_batch(&mut claims, false, true);
        assert_eq!(batch.skipped_gas, 1);
        assert_eq!(batch.succeeded, 0);
    }

    #[test]
    fn process_batch_simulation_failed_counted() {
        let mut claims = vec![make_claim("1", true)];
        let batch = process_batch(&mut claims, true, false);
        assert_eq!(batch.failed, 1);
        assert_eq!(batch.succeeded, 0);
    }

    #[test]
    fn batch_result_serializable() {
        let batch = ExecutionBatchResult {
            total_processed: 1,
            succeeded: 1,
            failed: 0,
            skipped_gas: 0,
            results: vec![],
        };
        let json = serde_json::to_string(&batch).unwrap();
        let rt: ExecutionBatchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.succeeded, 1);
    }

    // ── Simulation gate tests ───────────────────────────────

    #[test]
    fn gate_simple_claim_passes_on_safe_report() {
        let mut claim = make_claim_with_strategy("g1", ClaimStrategy::SimpleClaim);
        let report = passing_report();
        let ok = apply_simulation_gate(&mut claim, &report);
        assert!(ok);
        assert_eq!(claim.simulation_message.as_deref(), Some("Simulation passed — safe to proceed"));
        // Status should NOT have been changed to SimulationFailed
        assert_eq!(claim.status, ClaimStatus::Pending);
    }

    #[test]
    fn gate_simple_claim_rejects_on_unsafe_report() {
        let mut claim = make_claim_with_strategy("g2", ClaimStrategy::SimpleClaim);
        let report = failing_report("out of gas");
        let ok = apply_simulation_gate(&mut claim, &report);
        assert!(!ok);
        assert!(matches!(
            claim.status,
            ClaimStatus::SimulationFailed { ref reason } if reason.contains("revert")
        ));
        assert!(claim.simulation_message.is_some());
    }

    #[test]
    fn gate_multistep_rejects_on_unsafe_report() {
        let mut claim = make_claim_with_strategy("g3", ClaimStrategy::MultiStep);
        let report = failing_report("insufficient balance");
        let ok = apply_simulation_gate(&mut claim, &report);
        assert!(!ok);
        assert!(matches!(claim.status, ClaimStatus::SimulationFailed { .. }));
    }

    #[test]
    fn gate_multistep_passes_on_safe_report() {
        let mut claim = make_claim_with_strategy("g4", ClaimStrategy::MultiStep);
        let report = passing_report();
        let ok = apply_simulation_gate(&mut claim, &report);
        assert!(ok);
        assert_eq!(claim.status, ClaimStatus::Pending); // unchanged
    }

    #[test]
    fn gate_browser_claim_always_passes() {
        let mut claim = make_claim_with_strategy("g5", ClaimStrategy::BrowserClaim);
        let report = failing_report("would revert");
        let ok = apply_simulation_gate(&mut claim, &report);
        assert!(ok);
        // BrowserClaim should NOT store the message or change status
        assert!(claim.simulation_message.is_none());
        assert_eq!(claim.status, ClaimStatus::Pending);
    }

    #[test]
    fn gate_browser_claim_passes_even_with_flags() {
        let mut claim = make_claim_with_strategy("g6", ClaimStrategy::BrowserClaim);
        let mut report = failing_report("phishing");
        report.flags.unexpected_approval = true;
        let ok = apply_simulation_gate(&mut claim, &report);
        assert!(ok);
        assert_eq!(claim.status, ClaimStatus::Pending);
    }

    #[test]
    fn pipeline_browser_claim_bypasses_simulation_gate() {
        // BrowserClaim should reach Executing even when simulation_ok=false
        let mut claim = make_claim_with_strategy("b1", ClaimStrategy::BrowserClaim);
        let result = process_claim_step(&mut claim, true, false);
        assert_eq!(claim.status, ClaimStatus::Executing);
        assert!(result.simulation_ok);
    }

    #[test]
    fn pipeline_simple_claim_blocked_by_gate() {
        let mut claim = make_claim_with_strategy("s1", ClaimStrategy::SimpleClaim);
        let result = process_claim_step(&mut claim, true, false);
        assert!(matches!(claim.status, ClaimStatus::SimulationFailed { .. }));
        assert!(!result.simulation_ok);
    }

    #[test]
    fn pipeline_simple_claim_with_message_preserves_reason() {
        let mut claim = make_claim_with_strategy("s2", ClaimStrategy::SimpleClaim);
        claim.simulation_message = Some("BLOCKED: unlimited approval".to_string());
        let _result = process_claim_step(&mut claim, true, false);
        match &claim.status {
            ClaimStatus::SimulationFailed { reason } => {
                assert_eq!(reason, "BLOCKED: unlimited approval");
            }
            other => panic!("expected SimulationFailed, got {:?}", other),
        }
    }

    #[test]
    fn strategy_requires_simulation_gate() {
        assert!(ClaimStrategy::SimpleClaim.requires_simulation_gate());
        assert!(ClaimStrategy::MultiStep.requires_simulation_gate());
        assert!(!ClaimStrategy::BrowserClaim.requires_simulation_gate());
    }
}
