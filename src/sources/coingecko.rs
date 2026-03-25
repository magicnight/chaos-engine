use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const API_URL: &str =
    "https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page=20&sparkline=false";

pub struct CoinGecko {
    client: HttpClient,
}

impl CoinGecko {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for CoinGecko {
    fn name(&self) -> &str {
        "CoinGecko"
    }

    fn description(&self) -> &str {
        "Top 20 cryptocurrency market data"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        let data = self.client.fetch_json(API_URL).await?;

        let coins = data
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut total_market_cap: f64 = 0.0;
        let mut entries = Vec::new();
        let mut signals = Vec::new();

        for coin in coins.iter().take(20) {
            let name = coin.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let symbol = coin
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_uppercase();
            let price = coin
                .get("current_price")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let market_cap = coin
                .get("market_cap")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let change_24h = coin
                .get("price_change_percentage_24h")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let volume = coin
                .get("total_volume")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            total_market_cap += market_cap;

            if change_24h.abs() > 10.0 {
                signals.push(format!(
                    "{} ({}) moved {:.1}% in 24h",
                    name, symbol, change_24h
                ));
            }

            entries.push(json!({
                "name": name,
                "symbol": symbol,
                "price": price,
                "marketCap": market_cap,
                "change24h": change_24h,
                "volume24h": volume,
            }));
        }

        if signals.is_empty() {
            signals.push("Crypto markets within normal volatility range".to_string());
        }

        Ok(json!({
            "source": "CoinGecko",
            "timestamp": Utc::now().to_rfc3339(),
            "totalCoins": entries.len(),
            "totalMarketCap": total_market_cap,
            "coins": entries,
            "signals": signals,
        }))
    }
}
