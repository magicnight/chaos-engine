use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const BASE: &str = "https://comtradeapi.un.org/public/v1";

const STRATEGIC_COMMODITIES: &[(&str, &str)] = &[
    ("2709", "Crude Petroleum"),
    ("2711", "Natural Gas (LNG & Pipeline)"),
    ("7108", "Gold (unwrought/semi-manufactured)"),
    ("8542", "Semiconductors (Electronic Integrated Circuits)"),
    ("93", "Arms & Ammunition"),
    ("2844", "Radioactive Elements (Nuclear)"),
    ("8471", "Computers & Processing Units"),
    ("2701", "Coal"),
    ("7601", "Aluminium (unwrought)"),
    ("2612", "Uranium & Thorium Ores"),
];

const KEY_REPORTERS: &[(u32, &str)] = &[
    (842, "United States"),
    (156, "China"),
];

const KEY_COMMODITIES: &[&str] = &["2709", "2711", "7108", "8542", "93"];

pub struct Comtrade {
    client: HttpClient,
}

impl Comtrade {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    async fn fetch_trade(
        &self,
        reporter: u32,
        cmd_code: &str,
        period: i32,
    ) -> Option<Value> {
        let url = format!(
            "{}/preview/C/A/HS?reporterCode={}&period={}&cmdCode={}&flowCode=M",
            BASE, reporter, period, cmd_code
        );
        self.client.fetch_json(&url).await.ok()
    }
}

#[async_trait]
impl IntelSource for Comtrade {
    fn name(&self) -> &str {
        "UN Comtrade"
    }

    fn description(&self) -> &str {
        "UN strategic commodity trade flows"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        let current_year = Utc::now().format("%Y").to_string().parse::<i32>().unwrap_or(2025);
        let prev_year = current_year - 1;

        let mut trade_flows = Vec::new();
        let mut signals = Vec::new();

        for &(reporter, reporter_name) in KEY_REPORTERS {
            for &cmd_code in KEY_COMMODITIES {
                let commodity_label = STRATEGIC_COMMODITIES
                    .iter()
                    .find(|(c, _)| *c == cmd_code)
                    .map(|(_, l)| *l)
                    .unwrap_or(cmd_code);

                // Try current year, fall back to previous
                let data = match self.fetch_trade(reporter, cmd_code, current_year).await {
                    Some(d) => d,
                    None => match self.fetch_trade(reporter, cmd_code, prev_year).await {
                        Some(d) => d,
                        None => continue,
                    },
                };

                let records = data
                    .get("data")
                    .or_else(|| data.get("dataset"))
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                if records.is_empty() {
                    // If current year empty, try prev year
                    if let Some(fallback) = self.fetch_trade(reporter, cmd_code, prev_year).await {
                        let fb_records = fallback
                            .get("data")
                            .or_else(|| fallback.get("dataset"))
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();
                        if fb_records.is_empty() {
                            continue;
                        }
                        let compact = compact_records(&fb_records);
                        let anomalies = detect_anomalies(&compact);
                        signals.extend(anomalies);
                        trade_flows.push(json!({
                            "reporter": reporter_name,
                            "commodity": commodity_label,
                            "cmdCode": cmd_code,
                            "topPartners": compact,
                            "totalRecords": fb_records.len(),
                        }));
                        continue;
                    }
                    continue;
                }

                let compact = compact_records(&records);
                let anomalies = detect_anomalies(&compact);
                signals.extend(anomalies);

                trade_flows.push(json!({
                    "reporter": reporter_name,
                    "commodity": commodity_label,
                    "cmdCode": cmd_code,
                    "topPartners": compact,
                    "totalRecords": records.len(),
                }));
            }
        }

        if signals.is_empty() {
            signals.push("No significant trade anomalies detected in sampled commodities".to_string());
        }

        let commodities_map: Value = STRATEGIC_COMMODITIES
            .iter()
            .map(|(k, v)| (k.to_string(), json!(v)))
            .collect::<serde_json::Map<String, Value>>()
            .into();

        Ok(json!({
            "source": "UN Comtrade",
            "timestamp": Utc::now().to_rfc3339(),
            "tradeFlows": trade_flows,
            "signals": signals,
            "status": if trade_flows.is_empty() { "no_data" } else { "ok" },
            "note": "Comtrade data often lags 1-2 months. Recent periods may be incomplete.",
            "coveredCommodities": commodities_map,
        }))
    }
}

fn compact_records(records: &[Value]) -> Vec<Value> {
    records
        .iter()
        .take(10)
        .map(|rec| {
            json!({
                "reporter": rec.get("reporterDesc").or_else(|| rec.get("reporterCode")),
                "partner": rec.get("partnerDesc").or_else(|| rec.get("partnerCode")),
                "commodity": rec.get("cmdDesc").or_else(|| rec.get("cmdCode")),
                "flow": rec.get("flowDesc").or_else(|| rec.get("flowCode")),
                "value": rec.get("primaryValue")
                    .or_else(|| rec.get("cifvalue"))
                    .or_else(|| rec.get("fobvalue")),
                "quantity": rec.get("qty").or_else(|| rec.get("netWgt")),
                "period": rec.get("period"),
            })
        })
        .collect()
}

fn detect_anomalies(records: &[Value]) -> Vec<String> {
    let mut signals = Vec::new();
    let values: Vec<f64> = records
        .iter()
        .filter_map(|r| r.get("value").and_then(|v| v.as_f64()))
        .filter(|v| *v > 0.0)
        .collect();

    if values.len() <= 2 {
        return signals;
    }

    let avg: f64 = values.iter().sum::<f64>() / values.len() as f64;
    let variance: f64 = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();

    for rec in records {
        if let Some(val) = rec.get("value").and_then(|v| v.as_f64()) {
            if val > avg + 2.0 * std_dev {
                let partner = rec
                    .get("partner")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let commodity = rec
                    .get("commodity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                signals.push(format!(
                    "OUTLIER: {} trade with {} = ${:.2}B (mean: ${:.2}B)",
                    commodity,
                    partner,
                    val / 1e9,
                    avg / 1e9
                ));
            }
        }
    }

    signals
}
