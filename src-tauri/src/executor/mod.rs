pub mod consolidation;
pub mod gas_oracle;
pub mod queue;
pub mod simulation;
pub mod transaction;

use serde::{Deserialize, Serialize};

/// Status of a claim execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaimStatus {
    Pending,
    Simulating,
    WaitingForGas,
    Executing,
    Confirmed { tx_hash: String },
    Failed { reason: String },
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

/// Process a single claim through the pipeline stages.
///
/// Pipeline: VALIDATE → CHECK_GAS → SIMULATE → EXECUTE → RECORD
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

    // Stage 3: Simulation
    claim.status = ClaimStatus::Simulating;
    if !simulation_ok {
        claim.retry_count += 1;
        if claim.retry_count >= claim.max_retries {
            claim.status = ClaimStatus::Failed {
                reason: format!("simulation failed after {} retries", claim.retry_count),
            };
        } else {
            claim.status = ClaimStatus::Pending; // retry later
        }
        return make_result(claim, false);
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
        && !matches!(claim.status, ClaimStatus::Confirmed { .. })
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
            ClaimStatus::Failed { .. } => failed += 1,
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
        }
    }

    #[test]
    fn claim_status_serializable() {
        let statuses = vec![
            ClaimStatus::Pending,
            ClaimStatus::Simulating,
            ClaimStatus::WaitingForGas,
            ClaimStatus::Executing,
            ClaimStatus::Confirmed { tx_hash: "0xabc".to_string() },
            ClaimStatus::Failed { reason: "gas too high".to_string() },
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
    fn process_claim_simulation_fails_retries() {
        let mut claim = make_claim("1", true);
        let result = process_claim_step(&mut claim, true, false);
        assert_eq!(claim.retry_count, 1);
        assert_eq!(claim.status, ClaimStatus::Pending);
        assert!(!result.simulation_ok);
    }

    #[test]
    fn process_claim_simulation_exhausts_retries() {
        let mut claim = make_claim("1", true);
        claim.retry_count = 2;
        claim.max_retries = 3;
        let _result = process_claim_step(&mut claim, true, false);
        assert!(matches!(claim.status, ClaimStatus::Failed { .. }));
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
}
