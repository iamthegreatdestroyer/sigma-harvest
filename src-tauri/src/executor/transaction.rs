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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_claim_transaction_returns_not_implemented() {
        let result = build_claim_transaction("0xContract", "ethereum", "0xFrom");
        assert!(result.is_err());
    }

    #[test]
    fn transaction_error_not_implemented_display() {
        let err = TransactionError::NotImplemented;
        assert!(format!("{}", err).contains("not yet implemented"));
    }

    #[test]
    fn transaction_error_simulation_display() {
        let err = TransactionError::SimulationFailed("revert".to_string());
        assert!(format!("{}", err).contains("revert"));
    }

    #[test]
    fn transaction_error_gas_ceiling_display() {
        let err = TransactionError::GasExceedsCeiling {
            actual: 50000,
            ceiling: 21000,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("50000"));
        assert!(msg.contains("21000"));
    }

    #[test]
    fn transaction_error_is_debug() {
        let err = TransactionError::NotImplemented;
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotImplemented"));
    }
}
