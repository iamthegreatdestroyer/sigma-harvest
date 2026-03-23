//! RSS/Atom feed scraping for airdrop announcements.
//! Uses feed-rs to parse standard RSS 2.0, Atom, and JSON Feed formats.
//! Filters entries for crypto/airdrop keywords and extracts opportunities.

use super::{DiscoveryError, DiscoverySource, OpportunityType, RawOpportunity};
use chrono::Utc;

pub struct RssSource {
    pub feed_urls: Vec<String>,
}

/// Keywords that signal an airdrop/crypto opportunity in feed content.
const AIRDROP_KEYWORDS: &[&str] = &[
    "airdrop", "free mint", "token claim", "retroactive",
    "quest", "faucet", "bridge incentive", "liquidity bonus",
    "reward", "distribution", "eligibility", "snapshot",
];

/// Chain names we recognize from feed text.
const CHAIN_KEYWORDS: &[(&str, &str)] = &[
    ("ethereum", "ethereum"),
    ("arbitrum", "arbitrum"),
    ("optimism", "optimism"),
    ("base chain", "base"),
    ("base", "base"),
    ("polygon", "polygon"),
    ("zksync", "zksync"),
    ("solana", "solana"),
    ("avalanche", "avalanche"),
];

impl DiscoverySource for RssSource {
    fn name(&self) -> &str {
        "rss"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        let client = reqwest::Client::builder()
            .user_agent("SigmaHarvest/1.0")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(DiscoveryError::Network)?;

        let mut opportunities = Vec::new();

        for url in &self.feed_urls {
            match fetch_and_parse(&client, url).await {
                Ok(mut opps) => opportunities.append(&mut opps),
                Err(e) => {
                    tracing::warn!("RSS feed {} failed: {}", url, e);
                    continue;
                }
            }
        }

        Ok(opportunities)
    }
}

async fn fetch_and_parse(
    client: &reqwest::Client,
    url: &str,
) -> Result<Vec<RawOpportunity>, DiscoveryError> {
    let body = client
        .get(url)
        .send()
        .await
        .map_err(DiscoveryError::Network)?
        .text()
        .await
        .map_err(DiscoveryError::Network)?;

    let feed = feed_rs::parser::parse(body.as_bytes())
        .map_err(|e| DiscoveryError::Parse(format!("Feed parse error: {e}")))?;

    let now = Utc::now().to_rfc3339();
    let mut results = Vec::new();

    for entry in &feed.entries {
        let title = entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default();
        let description = entry
            .summary
            .as_ref()
            .map(|s| s.content.clone())
            .or_else(|| {
                entry.content.as_ref().and_then(|c| {
                    c.body.as_ref().map(|b| {
                        // Strip HTML tags for plain text matching
                        b.chars()
                            .fold((String::new(), false), |(mut acc, in_tag), c| {
                                if c == '<' { (acc, true) }
                                else if c == '>' { (acc, false) }
                                else if !in_tag { acc.push(c); (acc, false) }
                                else { (acc, true) }
                            })
                            .0
                    })
                })
            })
            .unwrap_or_default();

        let combined = format!("{} {}", title, description).to_lowercase();

        // Only include entries with at least one airdrop keyword
        if !AIRDROP_KEYWORDS.iter().any(|kw| combined.contains(kw)) {
            continue;
        }

        let chain = detect_chain(&combined);
        let opportunity_type = detect_type(&combined);
        let link = entry.links.first().map(|l| l.href.clone());

        results.push(RawOpportunity {
            source: "rss".to_string(),
            chain,
            opportunity_type,
            title,
            description: truncate(&description, 500),
            url: link,
            contract_address: None,
            estimated_value_usd: None,
            gas_cost_estimate: None,
            deadline: entry.published.map(|d| d.to_rfc3339()),
            discovered_at: now.clone(),
        });
    }

    Ok(results)
}

fn detect_chain(text: &str) -> String {
    for (keyword, chain) in CHAIN_KEYWORDS {
        if text.contains(keyword) {
            return chain.to_string();
        }
    }
    "unknown".to_string()
}

fn detect_type(text: &str) -> OpportunityType {
    if text.contains("retroactive") {
        OpportunityType::Retroactive
    } else if text.contains("quest") {
        OpportunityType::Quest
    } else if text.contains("free mint") || text.contains("freemint") {
        OpportunityType::FreeMint
    } else if text.contains("faucet") {
        OpportunityType::Faucet
    } else if text.contains("bridge incentive") || text.contains("bridge reward") {
        OpportunityType::BridgeIncentive
    } else if text.contains("liquidity bonus") || text.contains("lp reward") {
        OpportunityType::LiquidityBonus
    } else {
        OpportunityType::Airdrop
    }
}

fn truncate(s: &str, max: usize) -> String {
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
    fn detect_chain_ethereum() {
        assert_eq!(detect_chain("a big ethereum airdrop"), "ethereum");
    }

    #[test]
    fn detect_chain_arbitrum() {
        assert_eq!(detect_chain("claim on arbitrum now"), "arbitrum");
    }

    #[test]
    fn detect_chain_unknown() {
        assert_eq!(detect_chain("free tokens for everyone"), "unknown");
    }

    #[test]
    fn detect_type_retroactive() {
        assert_eq!(
            std::mem::discriminant(&detect_type("retroactive rewards")),
            std::mem::discriminant(&OpportunityType::Retroactive)
        );
    }

    #[test]
    fn detect_type_quest() {
        assert_eq!(
            std::mem::discriminant(&detect_type("complete the quest")),
            std::mem::discriminant(&OpportunityType::Quest)
        );
    }

    #[test]
    fn detect_type_default_airdrop() {
        assert_eq!(
            std::mem::discriminant(&detect_type("claim your tokens")),
            std::mem::discriminant(&OpportunityType::Airdrop)
        );
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        let long = "a".repeat(600);
        let result = truncate(&long, 500);
        assert_eq!(result.len(), 503); // 500 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn rss_source_name() {
        let source = RssSource { feed_urls: vec![] };
        assert_eq!(source.name(), "rss");
    }

    #[tokio::test]
    async fn rss_discover_empty_feeds_returns_empty() {
        let source = RssSource { feed_urls: vec![] };
        let result = source.discover().await.unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn detect_chain_base() {
        assert_eq!(detect_chain("new base chain airdrop"), "base");
    }

    #[test]
    fn detect_type_free_mint() {
        assert_eq!(
            std::mem::discriminant(&detect_type("exclusive free mint")),
            std::mem::discriminant(&OpportunityType::FreeMint)
        );
    }

    #[test]
    fn detect_type_bridge() {
        assert_eq!(
            std::mem::discriminant(&detect_type("bridge incentive program")),
            std::mem::discriminant(&OpportunityType::BridgeIncentive)
        );
    }
}
