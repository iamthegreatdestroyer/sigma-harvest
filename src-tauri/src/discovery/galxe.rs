//! Galxe GraphQL campaign discovery.
//! Fetches active quests/campaigns from the Galxe API.

use super::{DiscoveryError, DiscoverySource, OpportunityType, RawOpportunity};
use chrono::Utc;
use serde::Deserialize;

pub struct GalxeSource;

/// GraphQL query for fetching trending campaigns.
const CAMPAIGNS_QUERY: &str = r#"
query TrendingCampaigns {
    campaigns(input: { forAdmin: false, first: 20, status: Active, listType: Trending }) {
        list {
            id
            name
            description
            chain
            status
            type
            gamification {
                type
            }
            space {
                name
            }
        }
    }
}
"#;

#[derive(Debug, Deserialize)]
struct GalxeResponse {
    data: Option<GalxeData>,
}

#[derive(Debug, Deserialize)]
struct GalxeData {
    campaigns: Option<CampaignList>,
}

#[derive(Debug, Deserialize)]
struct CampaignList {
    list: Vec<GalxeCampaign>,
}

#[derive(Debug, Deserialize)]
struct GalxeCampaign {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    chain: String,
    #[serde(default)]
    space: Option<CampaignSpace>,
}

#[derive(Debug, Deserialize)]
struct CampaignSpace {
    #[serde(default)]
    name: String,
}

impl DiscoverySource for GalxeSource {
    fn name(&self) -> &str {
        "galxe"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        let client = reqwest::Client::builder()
            .user_agent("SigmaHarvest/1.0")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(DiscoveryError::Network)?;

        let body = serde_json::json!({ "query": CAMPAIGNS_QUERY });

        let response = client
            .post("https://graphigo.prd.galaxy.eco/query")
            .json(&body)
            .send()
            .await
            .map_err(DiscoveryError::Network)?;

        if !response.status().is_success() {
            return Err(DiscoveryError::Parse(format!(
                "Galxe API returned {}",
                response.status()
            )));
        }

        let data: GalxeResponse = response
            .json()
            .await
            .map_err(|e| DiscoveryError::Parse(format!("Galxe JSON: {e}")))?;

        let campaigns = data
            .data
            .and_then(|d| d.campaigns)
            .map(|c| c.list)
            .unwrap_or_default();

        let now = Utc::now().to_rfc3339();
        let opportunities = campaigns
            .into_iter()
            .map(|c| {
                let chain = normalize_galxe_chain(&c.chain);
                let project = c
                    .space
                    .as_ref()
                    .map(|s| s.name.clone())
                    .unwrap_or_default();

                RawOpportunity {
                    source: "galxe".to_string(),
                    chain,
                    opportunity_type: OpportunityType::Quest,
                    title: c.name,
                    description: if c.description.is_empty() {
                        format!("Galxe quest by {}", project)
                    } else {
                        truncate_desc(&c.description, 500)
                    },
                    url: Some(format!("https://galxe.com/quest/{}", c.id)),
                    contract_address: None,
                    estimated_value_usd: None,
                    gas_cost_estimate: None,
                    deadline: None,
                    discovered_at: now.clone(),
                }
            })
            .collect();

        Ok(opportunities)
    }
}

fn normalize_galxe_chain(raw: &str) -> String {
    match raw.to_lowercase().as_str() {
        "ethereum" | "eth" | "1" => "ethereum".to_string(),
        "arbitrum" | "arb" | "42161" => "arbitrum".to_string(),
        "optimism" | "op" | "10" => "optimism".to_string(),
        "base" | "8453" => "base".to_string(),
        "polygon" | "matic" | "137" => "polygon".to_string(),
        "zksync" | "324" => "zksync".to_string(),
        "bnb" | "bsc" | "56" => "bsc".to_string(),
        other => other.to_lowercase(),
    }
}

fn truncate_desc(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn galxe_source_name() {
        assert_eq!(GalxeSource.name(), "galxe");
    }

    #[test]
    fn normalize_galxe_chain_by_name() {
        assert_eq!(normalize_galxe_chain("Ethereum"), "ethereum");
        assert_eq!(normalize_galxe_chain("Arbitrum"), "arbitrum");
        assert_eq!(normalize_galxe_chain("Base"), "base");
    }

    #[test]
    fn normalize_galxe_chain_by_id() {
        assert_eq!(normalize_galxe_chain("1"), "ethereum");
        assert_eq!(normalize_galxe_chain("42161"), "arbitrum");
        assert_eq!(normalize_galxe_chain("137"), "polygon");
    }

    #[test]
    fn normalize_galxe_chain_unknown() {
        assert_eq!(normalize_galxe_chain("fantom"), "fantom");
    }

    #[test]
    fn deserialize_empty_response() {
        let json = r#"{"data": null}"#;
        let resp: GalxeResponse = serde_json::from_str(json).unwrap();
        assert!(resp.data.is_none());
    }

    #[test]
    fn deserialize_campaign() {
        let json = r#"{"id": "123", "name": "Test Quest", "description": "Do stuff", "chain": "ethereum", "space": {"name": "TestProject"}}"#;
        let campaign: GalxeCampaign = serde_json::from_str(json).unwrap();
        assert_eq!(campaign.name, "Test Quest");
        assert_eq!(campaign.space.unwrap().name, "TestProject");
    }

    #[test]
    fn truncate_desc_short() {
        assert_eq!(truncate_desc("short", 500), "short");
    }

    #[test]
    fn truncate_desc_long() {
        let long = "x".repeat(600);
        let result = truncate_desc(&long, 500);
        assert!(result.ends_with("..."));
        assert_eq!(result.len(), 503);
    }
}
