use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const V2_BASE: &str = "https://api.bls.gov/publicAPI/v2/timeseries/data/";

const SERIES: &[(&str, &str)] = &[
    ("CUSR0000SA0", "CPI-U All Items"),
    ("LNS14000000", "Unemployment Rate"),
    ("CES0000000001", "Nonfarm Payrolls (thousands)"),
    ("CUUR0000SA0L1E", "CPI-U Core (ex Food & Energy)"),
    ("WPUFD49104", "PPI Final Demand"),
];

pub struct Bls {
    client: HttpClient,
}

impl Bls {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for Bls {
    fn name(&self) -> &str {
        "BLS"
    }

    fn description(&self) -> &str {
        "Bureau of Labor Statistics — CPI, unemployment, payrolls"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        let api_key = std::env::var("BLS_API_KEY").unwrap_or_default();

        let now = chrono::Utc::now();
        let end_year = now.format("%Y").to_string();
        let start_year = format!("{}", now.format("%Y").to_string().parse::<i32>().unwrap_or(2025) - 1);

        let series_ids: Vec<&str> = SERIES.iter().map(|(id, _)| *id).collect();

        let mut payload = json!({
            "seriesid": series_ids,
            "startyear": start_year,
            "endyear": end_year,
        });

        if !api_key.is_empty() {
            payload.as_object_mut().unwrap().insert(
                "registrationkey".to_string(),
                json!(api_key),
            );
        }

        let resp = self
            .client
            .raw_client()
            .post(V2_BASE)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let text = resp.text().await?;
        let data: Value = serde_json::from_str(&text)?;

        let status = data.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status != "REQUEST_SUCCEEDED" {
            let msg = data.get("message")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .unwrap_or("BLS API returned no data");
            return Ok(json!({
                "source": "BLS",
                "timestamp": Utc::now().to_rfc3339(),
                "error": msg,
                "rawStatus": status,
            }));
        }

        let series_arr = data
            .pointer("/Results/series")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut indicators = Vec::new();
        let mut signals = Vec::new();

        for s in &series_arr {
            let id = s.get("seriesID").and_then(|v| v.as_str()).unwrap_or("");
            let label = SERIES.iter().find(|(sid, _)| *sid == id).map(|(_, l)| *l).unwrap_or(id);

            let obs = s.get("data").and_then(|v| v.as_array());
            let obs = match obs {
                Some(a) => a,
                None => continue,
            };

            // Filter and sort observations (newest first)
            let mut valid: Vec<&Value> = obs.iter().filter(|o| {
                let v = o.get("value").and_then(|v| v.as_str()).unwrap_or(".");
                v != "." && v != "-"
            }).collect();

            valid.sort_by(|a, b| {
                let ya = a.get("year").and_then(|v| v.as_str()).unwrap_or("0");
                let yb = b.get("year").and_then(|v| v.as_str()).unwrap_or("0");
                let pa = a.get("period").and_then(|v| v.as_str()).unwrap_or("");
                let pb = b.get("period").and_then(|v| v.as_str()).unwrap_or("");
                yb.cmp(ya).then(pb.cmp(pa))
            });

            if valid.is_empty() {
                continue;
            }

            let latest = valid[0];
            let value_str = latest.get("value").and_then(|v| v.as_str()).unwrap_or("0");
            let value: f64 = value_str.parse().unwrap_or(0.0);
            let year = latest.get("year").and_then(|v| v.as_str()).unwrap_or("");
            let period = latest.get("period").and_then(|v| v.as_str()).unwrap_or("");

            let mut indicator = json!({
                "id": id,
                "label": label,
                "value": value,
                "period": format!("{}-{}", year, period),
            });

            // Month-over-month change
            if valid.len() >= 2 {
                let prev_str = valid[1].get("value").and_then(|v| v.as_str()).unwrap_or("0");
                let prev: f64 = prev_str.parse().unwrap_or(0.0);
                if prev != 0.0 {
                    let change = value - prev;
                    let change_pct = (change / prev) * 100.0;
                    indicator.as_object_mut().unwrap().insert("momChange".to_string(), json!(change));
                    indicator.as_object_mut().unwrap().insert("momChangePct".to_string(), json!(change_pct));

                    // Generate signals
                    if id == "LNS14000000" && value > 5.0 {
                        signals.push(format!("Unemployment elevated at {}%", value));
                    }
                    if id == "CUSR0000SA0" && change_pct > 0.4 {
                        signals.push(format!("CPI-U MoM jump: {:.2}%", change_pct));
                    }
                    if id == "CES0000000001" && change < -50.0 {
                        signals.push(format!("Nonfarm payrolls dropped by {}K", change.abs()));
                    }
                }
            }

            indicators.push(indicator);
        }

        Ok(json!({
            "source": "BLS",
            "timestamp": Utc::now().to_rfc3339(),
            "indicators": indicators,
            "signals": signals,
        }))
    }
}
