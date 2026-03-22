//! Transaction building and signing.
//! Private keys NEVER leave the Rust process.

/// Build a raw transaction for a claim operation.
pub fn build_claim_transaction(
    _contract: &str,
    _chain: &str,
    _from: &str,
) -> Result<(), TransactionError> {
    // TODO: Build EIP-1559 transaction
    Err(TransactionError::NotImplemented)
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("transaction building not yet implemented")]
    NotImplemented,
    #[error("simulation failed: {0}")]
    SimulationFailed(String),
    #[error("gas exceeds ceiling: {actual} > {ceiling}")]
    GasExceedsCeiling { actual: u64, ceiling: u64 },
}
