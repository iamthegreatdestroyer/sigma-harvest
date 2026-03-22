//! Harvest Score algorithm v1.
//! Scores opportunities 0-100 based on value, risk, and urgency.

use crate::discovery::RawOpportunity;

/// Calculate the Harvest Score for a raw opportunity.
pub fn calculate(opportunity: &RawOpportunity) -> HarvestScoreResult {
    let _ = opportunity;
    // TODO: Implement scoring algorithm
    // Components:
    //   - Gas efficiency: (value / gas_cost) * 25  (max 25)
    //   - Contract verified: +20
    //   - Project signals: +15
    //   - Community size: +10
    //   - Time urgency: +15
    //   - Sybil risk penalty: -10 to -30
    HarvestScoreResult {
        score: 0,
        breakdown: ScoreBreakdown::default(),
    }
}

#[derive(Debug, Clone)]
pub struct HarvestScoreResult {
    pub score: u32,
    pub breakdown: ScoreBreakdown,
}

#[derive(Debug, Clone, Default)]
pub struct ScoreBreakdown {
    pub gas_efficiency: u32,
    pub contract_verified: u32,
    pub project_signals: u32,
    pub community_size: u32,
    pub time_urgency: u32,
    pub sybil_penalty: i32,
}
