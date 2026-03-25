use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const BASE: &str = "https://api.fiscaldata.treasury.gov/services/api/fiscal_service";

pub struct Treasury {
    client: HttpClient,
}

impl Treasury {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for Treasury {
    fn name(&self) -> &str {
        "US Treasury"
    }

    fn description(&self) -> &str {
        "US Treasury fiscal data — national debt and interest rates"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        let debt_url = format!(
            "{}/v2/accounting/od/debt_to_penny?fields=record_date,tot_pub_debt_out_amt,intragov_hold_amt,debt_held_public_amt&sort=-record_date&page[size]=10",
            BASE
        );
        let rates_url = format!(
            "{}/v2/accounting/od/avg_interest_rates?fields=record_date,security_desc,avg_interest_rate_amt&sort=-record_date&page[size]=50",
            BASE
        );

        let (debt_res, rates_res) = tokio::join!(
            self.client.fetch_json(&debt_url),
            self.client.fetch_json(&rates_url),
        );

        let mut signals = Vec::new();

        // Parse debt data
        let debt_data = debt_res
            .ok()
            .and_then(|v| v.get("data")?.as_array().cloned())
            .unwrap_or_default();

        let debt_entries: Vec<Value> = debt_data.iter().take(5).map(|d| {
            json!({
                "date": d.get("record_date").and_then(|v| v.as_str()).unwrap_or(""),
                "totalDebt": d.get("tot_pub_debt_out_amt").and_then(|v| v.as_str()).unwrap_or(""),
                "publicDebt": d.get("debt_held_public_amt").and_then(|v| v.as_str()).unwrap_or(""),
                "intragovDebt": d.get("intragov_hold_amt").and_then(|v| v.as_str()).unwrap_or(""),
            })
        }).collect();

        // Check for debt milestone signal
        if let Some(first) = debt_data.first() {
            if let Some(total_str) = first.get("tot_pub_debt_out_amt").and_then(|v| v.as_str()) {
                if let Ok(total) = total_str.parse::<f64>() {
                    if total > 36_000_000_000_000.0 {
                        signals.push(format!("National debt at ${:.2}T", total / 1e12));
                    }
                }
            }
        }

        // Compute daily change if we have at least 2 records
        if debt_data.len() >= 2 {
            let parse_debt = |entry: &Value| -> Option<f64> {
                entry.get("tot_pub_debt_out_amt")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
            };
            if let (Some(latest), Some(prev)) = (parse_debt(&debt_data[0]), parse_debt(&debt_data[1])) {
                let daily_change = latest - prev;
                if daily_change.abs() > 50_000_000_000.0 {
                    signals.push(format!(
                        "Large daily debt change: ${:.1}B",
                        daily_change / 1e9
                    ));
                }
            }
        }

        // Parse interest rates
        let rates_data = rates_res
            .ok()
            .and_then(|v| v.get("data")?.as_array().cloned())
            .unwrap_or_default();

        let interest_rates: Vec<Value> = rates_data.iter().take(20).map(|r| {
            json!({
                "date": r.get("record_date").and_then(|v| v.as_str()).unwrap_or(""),
                "security": r.get("security_desc").and_then(|v| v.as_str()).unwrap_or(""),
                "rate": r.get("avg_interest_rate_amt").and_then(|v| v.as_str()).unwrap_or(""),
            })
        }).collect();

        Ok(json!({
            "source": "US Treasury",
            "timestamp": Utc::now().to_rfc3339(),
            "debt": debt_entries,
            "interestRates": interest_rates,
            "signals": signals,
        }))
    }
}
