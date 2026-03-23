pub mod gas_oracle;
pub mod queue;
pub mod transaction;

use serde::{Deserialize, Serialize};

/// Status of a claim execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub status: ClaimStatus,
    pub gas_limit: Option<u64>,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let op = ClaimOperation {
            id: "c1".to_string(),
            opportunity_id: "op1".to_string(),
            wallet_address: "0xABC".to_string(),
            chain: "ethereum".to_string(),
            status: ClaimStatus::Pending,
            gas_limit: Some(21000),
            retry_count: 0,
            max_retries: 3,
        };
        let json = serde_json::to_string(&op).unwrap();
        let roundtrip: ClaimOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.id, "c1");
        assert_eq!(roundtrip.chain, "ethereum");
    }

    #[test]
    fn claim_operation_optional_gas_limit() {
        let op = ClaimOperation {
            id: "c1".to_string(),
            opportunity_id: "op1".to_string(),
            wallet_address: "0xABC".to_string(),
            chain: "ethereum".to_string(),
            status: ClaimStatus::Pending,
            gas_limit: None,
            retry_count: 0,
            max_retries: 3,
        };
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("null"));
    }
}
