use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

fn url_encode_symbol(symbol: &str) -> String {
    symbol.replace('^', "%5E").replace('=', "%3D")
}

const SYMBOLS: &[(&str, &str)] = &[
    ("SPY", "S&P 500"),
    ("QQQ", "Nasdaq 100"),
    ("DIA", "Dow Jones"),
    ("IWM", "Russell 2000"),
    ("GC=F", "Gold"),
    ("CL=F", "WTI Crude"),
    ("BZ=F", "Brent Crude"),
    ("NG=F", "Natural Gas"),
    ("BTC-USD", "Bitcoin"),
    ("ETH-USD", "Ethereum"),
    ("^VIX", "VIX"),
];

pub struct YFinance {
    client: HttpClient,
}

impl YFinance {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    async fn fetch_quote(&self, symbol: &str, name: &str) -> Option<Value> {
        let encoded = url_encode_symbol(symbol);
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?range=5d&interval=1d&includePrePost=false",
            encoded
        );

        let resp = self
            .client
            .raw_client()
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .send()
            .await
            .ok()?;

        let text = resp.text().await.ok()?;
        let data: Value = serde_json::from_str(&text).ok()?;

        let meta = data
            .pointer("/chart/result/0/meta")?;

        let price = meta.get("regularMarketPrice")?.as_f64()?;
        let prev_close = meta.get("chartPreviousClose")?.as_f64()?;
        let change = price - prev_close;
        let change_pct = if prev_close != 0.0 {
            (change / prev_close) * 100.0
        } else {
            0.0
        };

        Some(json!({
            "symbol": symbol,
            "name": name,
            "price": (price * 100.0).round() / 100.0,
            "change": (change * 100.0).round() / 100.0,
            "changePct": (change_pct * 100.0).round() / 100.0,
        }))
    }
}

#[async_trait]
impl IntelSource for YFinance {
    fn name(&self) -> &str {
        "YFinance"
    }

    fn description(&self) -> &str {
        "Yahoo Finance live market quotes"
    }

    fn tier(&self) -> u8 {
        5
    }

    async fn sweep(&self) -> Result<Value> {
        let futures: Vec<_> = SYMBOLS
            .iter()
            .map(|(sym, name)| self.fetch_quote(sym, name))
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut quotes = serde_json::Map::new();
        let mut ok_count = 0u32;
        let mut failed_count = 0u32;

        for (i, result) in results.into_iter().enumerate() {
            let (symbol, _) = SYMBOLS[i];
            match result {
                Some(quote) => {
                    ok_count += 1;
                    quotes.insert(symbol.to_string(), quote);
                }
                None => {
                    failed_count += 1;
                }
            }
        }

        Ok(json!({
            "source": "YFinance",
            "timestamp": Utc::now().to_rfc3339(),
            "summary": {
                "totalSymbols": SYMBOLS.len(),
                "ok": ok_count,
                "failed": failed_count,
            },
            "quotes": quotes,
        }))
    }
}
