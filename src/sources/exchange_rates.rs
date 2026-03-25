use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const API_URL: &str = "https://open.er-api.com/v6/latest/USD";

// Key currencies to track for geopolitical/economic signals
const TRACKED: &[(&str, &str)] = &[
    ("CNY", "Chinese Yuan"),
    ("RUB", "Russian Ruble"),
    ("EUR", "Euro"),
    ("GBP", "British Pound"),
    ("JPY", "Japanese Yen"),
    ("KRW", "Korean Won"),
    ("TRY", "Turkish Lira"),
    ("ARS", "Argentine Peso"),
    ("BRL", "Brazilian Real"),
    ("INR", "Indian Rupee"),
    ("TWD", "Taiwan Dollar"),
    ("UAH", "Ukrainian Hryvnia"),
    ("ILS", "Israeli Shekel"),
    ("IRR", "Iranian Rial"),
    ("NGN", "Nigerian Naira"),
];

pub struct ExchangeRates {
    client: HttpClient,
}

impl ExchangeRates {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for ExchangeRates {
    fn name(&self) -> &str {
        "ExchangeRates"
    }

    fn description(&self) -> &str {
        "Global currency exchange rates (166 currencies vs USD)"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        let data = self.client.fetch_json(API_URL).await?;

        let rates = data.get("rates").cloned().unwrap_or(json!({}));
        let mut tracked_rates = Vec::new();
        let mut signals = Vec::new();

        for (code, label) in TRACKED {
            if let Some(rate) = rates.get(*code).and_then(|v| v.as_f64()) {
                tracked_rates.push(json!({
                    "currency": code,
                    "name": label,
                    "rate": rate,
                }));
            }
        }

        // Signal: major currency movements (compare to known baselines)
        if let Some(rub) = rates.get("RUB").and_then(|v| v.as_f64()) {
            if rub > 100.0 {
                signals.push(format!("RUB weakening: {} per USD", rub as u64));
            }
        }
        if let Some(try_) = rates.get("TRY").and_then(|v| v.as_f64()) {
            if try_ > 35.0 {
                signals.push(format!("TRY under pressure: {:.1} per USD", try_));
            }
        }

        if signals.is_empty() {
            signals.push(format!("{} currencies tracked", tracked_rates.len()));
        }

        Ok(json!({
            "source": "ExchangeRates",
            "timestamp": Utc::now().to_rfc3339(),
            "baseCurrency": "USD",
            "totalCurrencies": rates.as_object().map(|o| o.len()).unwrap_or(0),
            "tracked": tracked_rates,
            "signals": signals,
        }))
    }
}
