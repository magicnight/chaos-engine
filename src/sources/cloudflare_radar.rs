use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const RADAR_BASE: &str = "https://api.cloudflare.com/client/v4/radar";

pub struct CloudflareRadar {
    client: HttpClient,
}

impl CloudflareRadar {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for CloudflareRadar {
    fn name(&self) -> &str {
        "Cloudflare-Radar"
    }

    fn description(&self) -> &str {
        "Cloudflare Radar internet traffic anomalies"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let api_token = std::env::var("CLOUDFLARE_API_TOKEN").unwrap_or_default();

        if api_token.is_empty() {
            return Ok(json!({
                "source": "Cloudflare-Radar",
                "timestamp": Utc::now().to_rfc3339(),
                "status": "no_credentials",
                "message": "Set CLOUDFLARE_API_TOKEN in .env. Get a free token at https://dash.cloudflare.com/profile/api-tokens",
            }));
        }

        // Fetch outage annotations (30 days)
        let outages_url = format!(
            "{}/annotations/outages?dateRange=30d&format=json",
            RADAR_BASE
        );
        let outages_resp = self
            .client
            .raw_client()
            .get(&outages_url)
            .header("Authorization", format!("Bearer {}", api_token))
            .send()
            .await;

        let outages_data = match outages_resp {
            Ok(r) => {
                let text = r.text().await.unwrap_or_default();
                serde_json::from_str::<Value>(&text).unwrap_or(json!({}))
            }
            Err(_) => json!({}),
        };

        let annotations = outages_data
            .get("result")
            .and_then(|r| r.get("annotations"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let outage_events: Vec<Value> = annotations
            .iter()
            .take(20)
            .map(|a| {
                json!({
                    "description": a.get("description").and_then(|v| v.as_str()).unwrap_or("").chars().take(500).collect::<String>(),
                    "startDate": a.get("startDate").and_then(|v| v.as_str()).unwrap_or(""),
                    "endDate": a.get("endDate").and_then(|v| v.as_str()).unwrap_or(""),
                    "locations": a.get("locations").unwrap_or(&json!([])),
                    "eventType": a.get("eventType").and_then(|v| v.as_str()).unwrap_or("outage"),
                })
            })
            .collect();

        // Fetch attack summary (7 days)
        let attacks_url = format!(
            "{}/attacks/layer3/summary/protocol?dateRange=7d&format=json",
            RADAR_BASE
        );
        let attacks_resp = self
            .client
            .raw_client()
            .get(&attacks_url)
            .header("Authorization", format!("Bearer {}", api_token))
            .send()
            .await;

        let attacks_data = match attacks_resp {
            Ok(r) => {
                let text = r.text().await.unwrap_or_default();
                serde_json::from_str::<Value>(&text).unwrap_or(json!({}))
            }
            Err(_) => json!({}),
        };

        let attack_summary = attacks_data
            .get("result")
            .and_then(|r| r.get("summary_0"))
            .cloned()
            .or_else(|| attacks_data.get("result").cloned())
            .unwrap_or(json!({}));

        // Fetch traffic anomalies (7 days)
        let anomalies_url = format!(
            "{}/traffic_anomalies?dateRange=7d&format=json&limit=50",
            RADAR_BASE
        );
        let anomalies_resp = self
            .client
            .raw_client()
            .get(&anomalies_url)
            .header("Authorization", format!("Bearer {}", api_token))
            .send()
            .await;

        let anomalies_data = match anomalies_resp {
            Ok(r) => {
                let text = r.text().await.unwrap_or_default();
                serde_json::from_str::<Value>(&text).unwrap_or(json!({}))
            }
            Err(_) => json!({}),
        };

        let anomalies = anomalies_data
            .get("result")
            .and_then(|r| r.get("trafficAnomalies"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut signals = Vec::new();
        if !outage_events.is_empty() {
            signals.push(format!(
                "{} internet outage events recorded in last 30 days",
                annotations.len()
            ));
        }
        if anomalies.len() > 10 {
            signals.push(format!(
                "{} traffic anomalies detected in last 7 days -- elevated internet instability",
                anomalies.len()
            ));
        }
        if signals.is_empty() {
            signals.push("Internet traffic patterns within normal range".to_string());
        }

        Ok(json!({
            "source": "Cloudflare-Radar",
            "timestamp": Utc::now().to_rfc3339(),
            "outages": {
                "total": annotations.len(),
                "events": outage_events,
            },
            "anomalies": {
                "total": anomalies.len(),
                "events": anomalies.iter().take(20).cloned().collect::<Vec<_>>(),
            },
            "attacks": attack_summary,
            "signals": signals,
        }))
    }
}
