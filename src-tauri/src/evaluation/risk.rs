//! Risk assessment for discovered opportunities.

/// Risk flags detected during evaluation.
pub fn assess_risk(contract_address: Option<&str>, _chain: &str) -> Vec<RiskFlag> {
    let _ = contract_address;
    // TODO: Implement risk checks
    // - Unverified contract
    // - Unlimited approve() calls
    // - Known scam database match
    // - Abnormally high estimated value
    vec![]
}

#[derive(Debug, Clone)]
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
