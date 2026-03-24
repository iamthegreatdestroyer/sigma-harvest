//! Transaction building, simulation, and gas estimation.
//! Private keys NEVER leave the Rust process.

use serde::{Deserialize, Serialize};

/// A prepared (unsigned) EIP-1559 transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedTransaction {
    pub chain: String,
    pub chain_id: u64,
    pub from: String,
    pub to: String,
    pub value_wei: String,
    pub data: String,
    pub nonce: u64,
    pub max_fee_per_gas_gwei: f64,
    pub max_priority_fee_gwei: f64,
    pub gas_limit: u64,
}

/// Result of a dry-run simulation via eth_call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub success: bool,
    pub return_data: String,
    pub revert_reason: Option<String>,
    pub estimated_gas: u64,
}

/// Build an unsigned transaction for a simple claim (contract call with no value).
pub fn build_claim_transaction(
    chain: &str,
    chain_id: u64,
    from: &str,
    to: &str,
    calldata: &str,
    nonce: u64,
    max_fee_gwei: f64,
    max_priority_gwei: f64,
    gas_limit: u64,
) -> Result<PreparedTransaction, TransactionError> {
    if to.is_empty() || !to.starts_with("0x") {
        return Err(TransactionError::InvalidAddress(to.to_string()));
    }
    if from.is_empty() || !from.starts_with("0x") {
        return Err(TransactionError::InvalidAddress(from.to_string()));
    }
    if gas_limit == 0 {
        return Err(TransactionError::InvalidGasLimit);
    }

    Ok(PreparedTransaction {
        chain: chain.to_string(),
        chain_id,
        from: from.to_string(),
        to: to.to_string(),
        value_wei: "0x0".to_string(),
        data: calldata.to_string(),
        nonce,
        max_fee_per_gas_gwei: max_fee_gwei,
        max_priority_fee_gwei: max_priority_gwei,
        gas_limit,
    })
}

/// Build the JSON-RPC params for an eth_call simulation.
pub fn build_simulation_params(tx: &PreparedTransaction) -> serde_json::Value {
    serde_json::json!([
        {
            "from": tx.from,
            "to": tx.to,
            "data": tx.data,
            "value": tx.value_wei,
        },
        "latest"
    ])
}

/// Build the JSON-RPC params for eth_estimateGas.
pub fn build_estimate_gas_params(tx: &PreparedTransaction) -> serde_json::Value {
    serde_json::json!([
        {
            "from": tx.from,
            "to": tx.to,
            "data": tx.data,
            "value": tx.value_wei,
        }
    ])
}

/// Parse an eth_call/estimateGas response into a SimulationResult.
pub fn parse_simulation_response(
    call_result: &serde_json::Value,
    gas_estimate: Option<u64>,
) -> SimulationResult {
    let return_data = call_result.as_str().unwrap_or("0x").to_string();

    // Check for common revert signatures (0x08c379a2 = Error(string))
    let revert_reason = if return_data.starts_with("0x08c379a2") && return_data.len() > 138 {
        // Decode ABI-encoded revert string
        let hex_str = &return_data[138..];
        let bytes: Vec<u8> = (0..hex_str.len())
            .step_by(2)
            .filter_map(|i| u8::from_str_radix(hex_str.get(i..i + 2)?, 16).ok())
            .collect();
        Some(String::from_utf8_lossy(&bytes).trim_end_matches('\0').to_string())
    } else {
        None
    };

    let success = revert_reason.is_none() && return_data != "0x";

    SimulationResult {
        success,
        return_data,
        revert_reason,
        estimated_gas: gas_estimate.unwrap_or(21000),
    }
}

/// Check gas limit against a ceiling.
pub fn check_gas_ceiling(tx: &PreparedTransaction, ceiling_gwei: f64) -> Result<(), TransactionError> {
    if tx.max_fee_per_gas_gwei > ceiling_gwei {
        return Err(TransactionError::GasExceedsCeiling {
            actual: tx.max_fee_per_gas_gwei as u64,
            ceiling: ceiling_gwei as u64,
        });
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("simulation failed: {0}")]
    SimulationFailed(String),
    #[error("gas exceeds ceiling: {actual} > {ceiling}")]
    GasExceedsCeiling { actual: u64, ceiling: u64 },
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    #[error("gas limit must be > 0")]
    InvalidGasLimit,
    #[error("signing error: {0}")]
    SigningError(String),
    #[error("broadcast error: {0}")]
    BroadcastError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_claim_transaction_success() {
        let tx = build_claim_transaction(
            "ethereum", 1,
            "0xAbC123", "0xDef456",
            "0xa9059cbb", 0, 30.0, 2.0, 100_000,
        ).unwrap();
        assert_eq!(tx.chain, "ethereum");
        assert_eq!(tx.chain_id, 1);
        assert_eq!(tx.from, "0xAbC123");
        assert_eq!(tx.to, "0xDef456");
        assert_eq!(tx.gas_limit, 100_000);
        assert_eq!(tx.nonce, 0);
    }

    #[test]
    fn build_claim_transaction_invalid_to() {
        let result = build_claim_transaction(
            "ethereum", 1, "0xAbC", "bad_addr", "0x", 0, 30.0, 2.0, 100_000,
        );
        assert!(matches!(result, Err(TransactionError::InvalidAddress(_))));
    }

    #[test]
    fn build_claim_transaction_invalid_from() {
        let result = build_claim_transaction(
            "ethereum", 1, "bad", "0xDef456", "0x", 0, 30.0, 2.0, 100_000,
        );
        assert!(matches!(result, Err(TransactionError::InvalidAddress(_))));
    }

    #[test]
    fn build_claim_transaction_zero_gas() {
        let result = build_claim_transaction(
            "ethereum", 1, "0xA", "0xB", "0x", 0, 30.0, 2.0, 0,
        );
        assert!(matches!(result, Err(TransactionError::InvalidGasLimit)));
    }

    #[test]
    fn build_simulation_params_format() {
        let tx = build_claim_transaction(
            "ethereum", 1, "0xA", "0xB", "0xdata", 5, 30.0, 2.0, 50000,
        ).unwrap();
        let params = build_simulation_params(&tx);
        let arr = params.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["from"], "0xA");
        assert_eq!(arr[0]["to"], "0xB");
        assert_eq!(arr[1], "latest");
    }

    #[test]
    fn build_estimate_gas_params_format() {
        let tx = build_claim_transaction(
            "ethereum", 1, "0xA", "0xB", "0x", 0, 30.0, 2.0, 50000,
        ).unwrap();
        let params = build_estimate_gas_params(&tx);
        let arr = params.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["from"], "0xA");
    }

    #[test]
    fn parse_simulation_success() {
        let result = serde_json::json!("0x0000000000000000000000000000000000000001");
        let sim = parse_simulation_response(&result, Some(45000));
        assert!(sim.success);
        assert_eq!(sim.estimated_gas, 45000);
        assert!(sim.revert_reason.is_none());
    }

    #[test]
    fn parse_simulation_empty_return() {
        let result = serde_json::json!("0x");
        let sim = parse_simulation_response(&result, None);
        assert!(!sim.success);
        assert_eq!(sim.estimated_gas, 21000);
    }

    #[test]
    fn check_gas_ceiling_ok() {
        let tx = build_claim_transaction(
            "ethereum", 1, "0xA", "0xB", "0x", 0, 20.0, 2.0, 50000,
        ).unwrap();
        assert!(check_gas_ceiling(&tx, 30.0).is_ok());
    }

    #[test]
    fn check_gas_ceiling_exceeded() {
        let tx = build_claim_transaction(
            "ethereum", 1, "0xA", "0xB", "0x", 0, 50.0, 2.0, 50000,
        ).unwrap();
        assert!(matches!(
            check_gas_ceiling(&tx, 30.0),
            Err(TransactionError::GasExceedsCeiling { .. })
        ));
    }

    #[test]
    fn transaction_error_display() {
        let err = TransactionError::SimulationFailed("revert".to_string());
        assert!(format!("{}", err).contains("revert"));
        let err = TransactionError::GasExceedsCeiling { actual: 50, ceiling: 30 };
        assert!(format!("{}", err).contains("50"));
    }

    #[test]
    fn prepared_transaction_serializable() {
        let tx = build_claim_transaction(
            "ethereum", 1, "0xA", "0xB", "0x", 0, 20.0, 2.0, 50000,
        ).unwrap();
        let json = serde_json::to_string(&tx).unwrap();
        let rt: PreparedTransaction = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.chain, "ethereum");
    }

    #[test]
    fn simulation_result_serializable() {
        let s = SimulationResult {
            success: true,
            return_data: "0x01".to_string(),
            revert_reason: None,
            estimated_gas: 21000,
        };
        let json = serde_json::to_string(&s).unwrap();
        let rt: SimulationResult = serde_json::from_str(&json).unwrap();
        assert!(rt.success);
    }
}
