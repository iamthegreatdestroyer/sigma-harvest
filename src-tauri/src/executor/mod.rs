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
