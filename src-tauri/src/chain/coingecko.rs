//! Token price fetching via CoinGecko free API.
//! Provides USD prices for native tokens and ERC-20 tokens with caching.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// CoinGecko API base URL.
const COINGECKO_API: &str = "https://api.coingecko.com/api/v3";

/// Cache TTL — 5 minutes.
const CACHE_TTL: Duration = Duration::from_secs(300);

/// Map of our chain symbols to CoinGecko coin IDs.
fn chain_to_coingecko_id(chain: &str) -> Option<&'static str> {
    match chain.to_lowercase().as_str() {
        "ethereum" | "eth" => Some("ethereum"),
        "arbitrum" | "arb" => Some("ethereum"),   // Arbitrum uses ETH for gas
        "optimism" | "op" => Some("ethereum"),     // Optimism uses ETH for gas
        "base" => Some("ethereum"),                // Base uses ETH for gas
        "polygon" | "matic" => Some("matic-network"),
        "zksync" | "zk" => Some("ethereum"),       // zkSync uses ETH for gas
        _ => None,
    }
}

/// Price result for a single token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub id: String,
    pub symbol: String,
    pub usd: f64,
    pub usd_24h_change: Option<f64>,
    pub last_updated: String,
}

/// A collection of token prices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResponse {
    pub prices: Vec<TokenPrice>,
    pub cached: bool,
}

/// Cached price entry.
struct CachedPrice {
    prices: Vec<TokenPrice>,
    fetched_at: Instant,
}

/// CoinGecko price client with in-memory caching.
pub struct PriceClient {
    http: Client,
    cache: Mutex<Option<CachedPrice>>,
    api_key: Option<String>,
}

impl PriceClient {
    /// Create a new price client, optionally with a CoinGecko API key.
    pub fn new(api_key: Option<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");

        Self {
            http,
            cache: Mutex::new(None),
            api_key,
        }
    }

    /// Fetch prices for the native gas tokens of all supported chains.
    /// Uses CoinGecko's simple/price endpoint.
    /// Returns cached data if within TTL.
    pub async fn get_native_prices(&self) -> Result<PriceResponse, PriceError> {
        // Check cache
        {
            let cache = self.cache.lock().map_err(|e| PriceError::Cache(e.to_string()))?;
            if let Some(ref cached) = *cache {
                if cached.fetched_at.elapsed() < CACHE_TTL {
                    return Ok(PriceResponse {
                        prices: cached.prices.clone(),
                        cached: true,
                    });
                }
            }
        }

        // Fetch from CoinGecko
        let coin_ids = "ethereum,matic-network";
        let mut url = format!(
            "{}/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true&include_last_updated_at=true",
            COINGECKO_API, coin_ids
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&x_cg_demo_api_key={}", key));
        }

        let resp = self
            .http
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| PriceError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(PriceError::Api(format!("HTTP {}", resp.status())));
        }

        let data: HashMap<String, CoinGeckoPrice> = resp
            .json()
            .await
            .map_err(|e| PriceError::Parse(e.to_string()))?;

        let prices = parse_coingecko_response(data);

        // Update cache
        {
            let mut cache = self.cache.lock().map_err(|e| PriceError::Cache(e.to_string()))?;
            *cache = Some(CachedPrice {
                prices: prices.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(PriceResponse {
            prices,
            cached: false,
        })
    }

    /// Get the USD price for a specific chain's native token.
    pub async fn get_chain_price(&self, chain: &str) -> Result<f64, PriceError> {
        let coingecko_id = chain_to_coingecko_id(chain)
            .ok_or_else(|| PriceError::UnknownChain(chain.to_string()))?;

        let response = self.get_native_prices().await?;
        response
            .prices
            .iter()
            .find(|p| p.id == coingecko_id)
            .map(|p| p.usd)
            .ok_or_else(|| PriceError::PriceNotFound(chain.to_string()))
    }

    /// Convert a wei amount to USD given the chain.
    pub async fn wei_to_usd(&self, chain: &str, wei: u128) -> Result<f64, PriceError> {
        let price = self.get_chain_price(chain).await?;
        let eth_amount = wei as f64 / 1e18;
        Ok(eth_amount * price)
    }

    /// Clear the price cache.
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            *cache = None;
        }
    }
}

/// Raw CoinGecko simple/price response entry.
#[derive(Debug, Deserialize)]
struct CoinGeckoPrice {
    usd: Option<f64>,
    usd_24h_change: Option<f64>,
    last_updated_at: Option<i64>,
}

/// Parse CoinGecko response into our TokenPrice format.
fn parse_coingecko_response(data: HashMap<String, CoinGeckoPrice>) -> Vec<TokenPrice> {
    let symbol_map: HashMap<&str, &str> = HashMap::from([
        ("ethereum", "ETH"),
        ("matic-network", "MATIC"),
    ]);

    data.into_iter()
        .filter_map(|(id, price)| {
            let usd = price.usd?;
            let symbol = symbol_map.get(id.as_str()).unwrap_or(&"???");
            let timestamp = price
                .last_updated_at
                .map(|ts| {
                    chrono::DateTime::from_timestamp(ts, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            Some(TokenPrice {
                id: id.clone(),
                symbol: symbol.to_string(),
                usd,
                usd_24h_change: price.usd_24h_change,
                last_updated: timestamp,
            })
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum PriceError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("cache error: {0}")]
    Cache(String),
    #[error("unknown chain: {0}")]
    UnknownChain(String),
    #[error("price not found for {0}")]
    PriceNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── chain_to_coingecko_id ─────────────────────────────────

    #[test]
    fn chain_mapping_ethereum() {
        assert_eq!(chain_to_coingecko_id("ethereum"), Some("ethereum"));
        assert_eq!(chain_to_coingecko_id("ETH"), Some("ethereum"));
    }

    #[test]
    fn chain_mapping_polygon() {
        assert_eq!(chain_to_coingecko_id("polygon"), Some("matic-network"));
        assert_eq!(chain_to_coingecko_id("matic"), Some("matic-network"));
    }

    #[test]
    fn chain_mapping_l2s_to_eth() {
        assert_eq!(chain_to_coingecko_id("arbitrum"), Some("ethereum"));
        assert_eq!(chain_to_coingecko_id("optimism"), Some("ethereum"));
        assert_eq!(chain_to_coingecko_id("base"), Some("ethereum"));
        assert_eq!(chain_to_coingecko_id("zksync"), Some("ethereum"));
    }

    #[test]
    fn chain_mapping_unknown() {
        assert_eq!(chain_to_coingecko_id("solana"), None);
        assert_eq!(chain_to_coingecko_id("bitcoin"), None);
    }

    // ── parse_coingecko_response ──────────────────────────────

    #[test]
    fn parse_response_ethereum() {
        let mut data = HashMap::new();
        data.insert(
            "ethereum".to_string(),
            CoinGeckoPrice {
                usd: Some(3500.0),
                usd_24h_change: Some(2.5),
                last_updated_at: Some(1700000000),
            },
        );
        let prices = parse_coingecko_response(data);
        assert_eq!(prices.len(), 1);
        assert_eq!(prices[0].symbol, "ETH");
        assert_eq!(prices[0].usd, 3500.0);
    }

    #[test]
    fn parse_response_multiple() {
        let mut data = HashMap::new();
        data.insert(
            "ethereum".to_string(),
            CoinGeckoPrice {
                usd: Some(3500.0),
                usd_24h_change: Some(2.5),
                last_updated_at: Some(1700000000),
            },
        );
        data.insert(
            "matic-network".to_string(),
            CoinGeckoPrice {
                usd: Some(0.85),
                usd_24h_change: Some(-1.2),
                last_updated_at: Some(1700000000),
            },
        );
        let prices = parse_coingecko_response(data);
        assert_eq!(prices.len(), 2);
    }

    #[test]
    fn parse_response_skips_null_usd() {
        let mut data = HashMap::new();
        data.insert(
            "ethereum".to_string(),
            CoinGeckoPrice {
                usd: None,
                usd_24h_change: None,
                last_updated_at: None,
            },
        );
        let prices = parse_coingecko_response(data);
        assert_eq!(prices.len(), 0);
    }

    // ── TokenPrice serializable ───────────────────────────────

    #[test]
    fn token_price_serializable() {
        let price = TokenPrice {
            id: "ethereum".to_string(),
            symbol: "ETH".to_string(),
            usd: 3500.42,
            usd_24h_change: Some(2.5),
            last_updated: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&price).unwrap();
        let roundtrip: TokenPrice = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.usd, 3500.42);
        assert_eq!(roundtrip.symbol, "ETH");
    }

    // ── PriceResponse serializable ────────────────────────────

    #[test]
    fn price_response_serializable() {
        let resp = PriceResponse {
            prices: vec![TokenPrice {
                id: "ethereum".to_string(),
                symbol: "ETH".to_string(),
                usd: 3500.0,
                usd_24h_change: None,
                last_updated: "".to_string(),
            }],
            cached: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let roundtrip: PriceResponse = serde_json::from_str(&json).unwrap();
        assert!(roundtrip.cached);
        assert_eq!(roundtrip.prices.len(), 1);
    }

    // ── PriceClient ───────────────────────────────────────────

    #[test]
    fn price_client_new() {
        let client = PriceClient::new(None);
        client.clear_cache(); // should not panic
    }

    #[test]
    fn price_client_with_api_key() {
        let client = PriceClient::new(Some("test-key".to_string()));
        client.clear_cache();
    }

    // ── Error display ─────────────────────────────────────────

    #[test]
    fn error_http_display() {
        let err = PriceError::Http("timeout".to_string());
        assert!(format!("{}", err).contains("timeout"));
    }

    #[test]
    fn error_unknown_chain_display() {
        let err = PriceError::UnknownChain("solana".to_string());
        assert!(format!("{}", err).contains("solana"));
    }
}
