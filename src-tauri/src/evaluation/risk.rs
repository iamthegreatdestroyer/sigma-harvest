//! Risk assessment for discovered opportunities.
//! Performs heuristic-based risk analysis based on opportunity metadata.

use crate::discovery::RawOpportunity;
use serde::{Deserialize, Serialize};

/// Assess risk from a full RawOpportunity.
pub fn assess_opportunity(opp: &RawOpportunity) -> RiskAssessment {
    let flags = assess_risk(opp.contract_address.as_deref(), &opp.chain, opp);
    let level = determine_risk_level(&flags);
    RiskAssessment { flags, level }
}

/// Risk flags detected during evaluation.
pub fn assess_risk(
    contract_address: Option<&str>,
    _chain: &str,
    opp: &RawOpportunity,
) -> Vec<RiskFlag> {
    let mut flags = Vec::new();

    // 1. No contract address → harder to verify legitimacy
    if contract_address.is_none() {
        flags.push(RiskFlag::UnverifiedContract);
    }

    // 2. Contract address present but suspiciously short or not hex-like
    if let Some(addr) = contract_address {
        let trimmed = addr.strip_prefix("0x").unwrap_or(addr);
        if trimmed.len() < 40 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            flags.push(RiskFlag::UnverifiedContract);
        }
    }

    // 3. Suspiciously high estimated value (common scam pattern)
    if let Some(value) = opp.estimated_value_usd {
        if value > 5_000.0 {
            flags.push(RiskFlag::TooGoodToBeTrue);
        }
    }

    // 4. No URL → can't verify project exists
    if opp.url.is_none() {
        flags.push(RiskFlag::NoAudit);
    }

    // 5. Empty or very short description → low-effort listing (potential spam)
    if opp.description.len() < 10 && opp.title.len() < 10 {
        flags.push(RiskFlag::RecentDeployment);
    }

    flags
}

/// Determine overall risk level from flags.
fn determine_risk_level(flags: &[RiskFlag]) -> RiskLevel {
    let has_critical = flags.iter().any(|f| {
        matches!(f, RiskFlag::KnownScamMatch | RiskFlag::TooGoodToBeTrue)
    });

    if has_critical {
        return RiskLevel::Critical;
    }

    match flags.len() {
        0 => RiskLevel::Low,
        1 => RiskLevel::Medium,
        _ => RiskLevel::High,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub flags: Vec<RiskFlag>,
    pub level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFlag {
    UnverifiedContract,
    UnlimitedApproval,
    KnownScamMatch,
    TooGoodToBeTrue,
    NoAudit,
    RecentDeployment,
}

impl std::fmt::Display for RiskFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnverifiedContract => write!(f, "Unverified contract"),
            Self::UnlimitedApproval => write!(f, "Unlimited token approval"),
            Self::KnownScamMatch => write!(f, "Known scam match"),
            Self::TooGoodToBeTrue => write!(f, "Suspiciously high value"),
            Self::NoAudit => write!(f, "No audit found"),
            Self::RecentDeployment => write!(f, "Recently deployed contract"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{OpportunityType, RawOpportunity};

    fn test_opp() -> RawOpportunity {
        RawOpportunity {
            source: "rss".to_string(),
            chain: "ethereum".to_string(),
            opportunity_type: OpportunityType::Airdrop,
            title: "Test Airdrop Campaign".to_string(),
            description: "Free tokens for early users who participate in the program".to_string(),
            url: Some("https://example.com".to_string()),
            contract_address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            estimated_value_usd: Some(50.0),
            gas_cost_estimate: Some(2.0),
            deadline: Some("2026-12-31".to_string()),
            discovered_at: "2026-03-22".to_string(),
        }
    }

    #[test]
    fn assess_clean_opportunity_no_flags() {
        let opp = test_opp();
        let flags = assess_risk(opp.contract_address.as_deref(), &opp.chain, &opp);
        assert!(flags.is_empty(), "Expected no flags for clean opp, got {:?}", flags);
    }

    #[test]
    fn assess_risk_no_contract_flags_unverified() {
        let mut opp = test_opp();
        opp.contract_address = None;
        let flags = assess_risk(None, "ethereum", &opp);
        assert!(flags.iter().any(|f| matches!(f, RiskFlag::UnverifiedContract)));
    }

    #[test]
    fn assess_risk_short_contract_flags_unverified() {
        let mut opp = test_opp();
        opp.contract_address = Some("0x1234".to_string());
        let flags = assess_risk(opp.contract_address.as_deref(), "ethereum", &opp);
        assert!(flags.iter().any(|f| matches!(f, RiskFlag::UnverifiedContract)));
    }

    #[test]
    fn assess_risk_high_value_flags_suspicious() {
        let mut opp = test_opp();
        opp.estimated_value_usd = Some(10_000.0);
        let flags = assess_risk(opp.contract_address.as_deref(), "ethereum", &opp);
        assert!(flags.iter().any(|f| matches!(f, RiskFlag::TooGoodToBeTrue)));
    }

    #[test]
    fn assess_risk_no_url_flags_no_audit() {
        let mut opp = test_opp();
        opp.url = None;
        let flags = assess_risk(opp.contract_address.as_deref(), "ethereum", &opp);
        assert!(flags.iter().any(|f| matches!(f, RiskFlag::NoAudit)));
    }

    #[test]
    fn assess_risk_sparse_content_flags_recent() {
        let mut opp = test_opp();
        opp.description = "x".to_string();
        opp.title = "y".to_string();
        let flags = assess_risk(opp.contract_address.as_deref(), "ethereum", &opp);
        assert!(flags.iter().any(|f| matches!(f, RiskFlag::RecentDeployment)));
    }

    #[test]
    fn assess_opportunity_returns_assessment() {
        let opp = test_opp();
        let assessment = assess_opportunity(&opp);
        assert!(assessment.flags.is_empty());
        assert!(matches!(assessment.level, RiskLevel::Low));
    }

    #[test]
    fn assess_opportunity_critical_for_scam() {
        let mut opp = test_opp();
        opp.estimated_value_usd = Some(50_000.0);
        let assessment = assess_opportunity(&opp);
        assert!(matches!(assessment.level, RiskLevel::Critical));
    }

    #[test]
    fn determine_risk_level_no_flags_low() {
        assert!(matches!(determine_risk_level(&[]), RiskLevel::Low));
    }

    #[test]
    fn determine_risk_level_one_flag_medium() {
        assert!(matches!(
            determine_risk_level(&[RiskFlag::NoAudit]),
            RiskLevel::Medium
        ));
    }

    #[test]
    fn determine_risk_level_two_flags_high() {
        assert!(matches!(
            determine_risk_level(&[RiskFlag::NoAudit, RiskFlag::UnverifiedContract]),
            RiskLevel::High
        ));
    }

    #[test]
    fn determine_risk_level_scam_critical() {
        assert!(matches!(
            determine_risk_level(&[RiskFlag::TooGoodToBeTrue]),
            RiskLevel::Critical
        ));
    }

    #[test]
    fn risk_flag_display_all_variants() {
        let variants = vec![
            (RiskFlag::UnverifiedContract, "Unverified contract"),
            (RiskFlag::UnlimitedApproval, "Unlimited token approval"),
            (RiskFlag::KnownScamMatch, "Known scam match"),
            (RiskFlag::TooGoodToBeTrue, "Suspiciously high value"),
            (RiskFlag::NoAudit, "No audit found"),
            (RiskFlag::RecentDeployment, "Recently deployed contract"),
        ];
        for (flag, expected) in variants {
            assert_eq!(format!("{}", flag), expected);
        }
    }

    #[test]
    fn risk_flag_is_debug() {
        let flag = RiskFlag::UnverifiedContract;
        let debug = format!("{:?}", flag);
        assert!(debug.contains("UnverifiedContract"));
    }

    #[test]
    fn risk_flag_is_clone() {
        let flag = RiskFlag::KnownScamMatch;
        let clone = flag.clone();
        assert_eq!(format!("{}", clone), "Known scam match");
    }

    #[test]
    fn risk_assessment_serializable() {
        let assessment = assess_opportunity(&test_opp());
        let json = serde_json::to_string(&assessment).unwrap();
        let deserialized: RiskAssessment = serde_json::from_str(&json).unwrap();
        assert!(deserialized.flags.is_empty());
    }
}
