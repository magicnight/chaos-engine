use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const BASE: &str = "https://api.stlouisfed.org/fred";

const KEY_SERIES: &[(&str, &str)] = &[
    ("DFF", "Fed Funds Rate"),
    ("DGS2", "2-Year Treasury Yield"),
    ("DGS10", "10-Year Treasury Yield"),
    ("DGS30", "30-Year Treasury Yield"),
    ("T10Y2Y", "10Y-2Y Spread (Yield Curve)"),
    ("T10Y3M", "10Y-3M Spread"),
    ("CPIAUCSL", "CPI All Items"),
    ("UNRATE", "Unemployment Rate"),
    ("PAYEMS", "Nonfarm Payrolls"),
    ("ICSA", "Initial Jobless Claims"),
    ("VIXCLS", "VIX (Fear Index)"),
    ("BAMLH0A0HYM2", "High Yield Spread (Credit Stress)"),
    ("DCOILWTICO", "WTI Crude Oil"),
    ("MORTGAGE30US", "30-Year Mortgage Rate"),
    ("DTWEXBGS", "USD Trade Weighted Index"),
];

pub struct Fred {
    client: HttpClient,
}

impl Fred {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    async fn fetch_series(&self, series_id: &str, api_key: &str) -> Option<Value> {
        let url = format!(
            "{}/series/observations?series_id={}&api_key={}&file_type=json&sort_order=desc&limit=5",
            BASE, series_id, api_key
        );
        let data = self.client.fetch_json(&url).await.ok()?;
        let obs = data.get("observations")?.as_array()?;

        // Find first observation with a non-"." value
        let latest = obs.iter().find(|o| {
            o.get("value")
                .and_then(|v| v.as_str())
                .map(|s| s != ".")
                .unwrap_or(false)
        })?;

        let value_str = latest.get("value")?.as_str()?;
        let value: f64 = value_str.parse().ok()?;
        let date = latest.get("date").and_then(|v| v.as_str()).unwrap_or("");

        Some(json!({
            "value": value,
            "date": date,
        }))
    }
}

#[async_trait]
impl IntelSource for Fred {
    fn name(&self) -> &str {
        "FRED"
    }

    fn description(&self) -> &str {
        "Federal Reserve Economic Data"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        let api_key = std::env::var("FRED_API_KEY").unwrap_or_default();

        if api_key.is_empty() {
            return Ok(json!({
                "source": "FRED",
                "timestamp": Utc::now().to_rfc3339(),
                "error": "No FRED API key. Get one free at https://fred.stlouisfed.org/docs/api/api_key.html",
                "hint": "Set FRED_API_KEY environment variable",
            }));
        }

        let futures: Vec<_> = KEY_SERIES
            .iter()
            .map(|(id, _)| self.fetch_series(id, &api_key))
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut indicators = Vec::new();
        let mut value_map = serde_json::Map::new();

        for (i, result) in results.into_iter().enumerate() {
            let (id, label) = KEY_SERIES[i];
            if let Some(data) = result {
                let value = data.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let date = data.get("date").and_then(|v| v.as_str()).unwrap_or("");
                value_map.insert(id.to_string(), json!(value));
                indicators.push(json!({
                    "id": id,
                    "label": label,
                    "value": value,
                    "date": date,
                }));
            }
        }

        // Generate signals
        let mut signals = Vec::new();

        let get_val = |key: &str| -> Option<f64> {
            value_map.get(key).and_then(|v| v.as_f64())
        };

        if let Some(spread) = get_val("T10Y2Y") {
            if spread < 0.0 {
                signals.push("YIELD CURVE INVERTED (10Y-2Y) — recession signal".to_string());
            }
        }
        if let Some(spread) = get_val("T10Y3M") {
            if spread < 0.0 {
                signals.push("YIELD CURVE INVERTED (10Y-3M) — stronger recession signal".to_string());
            }
        }
        if let Some(vix) = get_val("VIXCLS") {
            if vix > 40.0 {
                signals.push(format!("VIX EXTREME at {} — crisis-level fear", vix));
            } else if vix > 30.0 {
                signals.push(format!("VIX ELEVATED at {} — high fear/volatility", vix));
            }
        }
        if let Some(hy) = get_val("BAMLH0A0HYM2") {
            if hy > 5.0 {
                signals.push(format!("HIGH YIELD SPREAD WIDE at {}% — credit stress", hy));
            }
        }

        Ok(json!({
            "source": "FRED",
            "timestamp": Utc::now().to_rfc3339(),
            "indicators": indicators,
            "signals": signals,
        }))
    }
}
