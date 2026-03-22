pub mod harvest_score;
pub mod risk;

use serde::{Deserialize, Serialize};

/// An evaluated opportunity with harvest score and risk assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatedOpportunity {
    pub id: String,
    pub title: String,
    pub chain: String,
    pub harvest_score: u32,
    pub risk_level: RiskLevel,
    pub risk_flags: Vec<String>,
    pub estimated_value_usd: Option<f64>,
    pub gas_cost_estimate: Option<f64>,
    pub status: OpportunityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityStatus {
    Discovered,
    Evaluating,
    Qualified,
    Claiming,
    Claimed,
    Expired,
    Rejected,
    Failed,
}
