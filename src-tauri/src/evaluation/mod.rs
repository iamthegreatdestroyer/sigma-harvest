pub mod harvest_score;
pub mod risk;

use serde::{Deserialize, Serialize};

// Re-export risk types for convenience
pub use risk::{RiskAssessment, RiskFlag, RiskLevel};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluated_opportunity_serializable() {
        let opp = EvaluatedOpportunity {
            id: "opp-1".to_string(),
            title: "Test Airdrop".to_string(),
            chain: "ethereum".to_string(),
            harvest_score: 75,
            risk_level: RiskLevel::Low,
            risk_flags: vec!["NoAudit".to_string()],
            estimated_value_usd: Some(100.0),
            gas_cost_estimate: Some(2.5),
            status: OpportunityStatus::Qualified,
        };
        let json = serde_json::to_string(&opp).unwrap();
        let deserialized: EvaluatedOpportunity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.harvest_score, 75);
        assert_eq!(deserialized.id, "opp-1");
    }

    #[test]
    fn risk_level_all_variants_serializable() {
        for level in [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High, RiskLevel::Critical] {
            let json = serde_json::to_string(&level).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn opportunity_status_all_variants_serializable() {
        let statuses = vec![
            OpportunityStatus::Discovered,
            OpportunityStatus::Evaluating,
            OpportunityStatus::Qualified,
            OpportunityStatus::Claiming,
            OpportunityStatus::Claimed,
            OpportunityStatus::Expired,
            OpportunityStatus::Rejected,
            OpportunityStatus::Failed,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let roundtrip: OpportunityStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(serde_json::to_string(&roundtrip).unwrap(), json);
        }
    }

    #[test]
    fn evaluated_opportunity_optional_fields() {
        let opp = EvaluatedOpportunity {
            id: "id".to_string(),
            title: "t".to_string(),
            chain: "ethereum".to_string(),
            harvest_score: 0,
            risk_level: RiskLevel::Medium,
            risk_flags: vec![],
            estimated_value_usd: None,
            gas_cost_estimate: None,
            status: OpportunityStatus::Discovered,
        };
        let json = serde_json::to_string(&opp).unwrap();
        assert!(json.contains("null"));
    }
}
