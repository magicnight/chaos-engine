use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const INFOCON_URL: &str = "https://isc.sans.edu/api/infocon?json";

pub struct Isc {
    client: HttpClient,
}

impl Isc {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn threat_level_severity(level: &str) -> &'static str {
    match level.to_lowercase().as_str() {
        "red" => "critical",
        "orange" => "high",
        "yellow" => "elevated",
        "green" => "normal",
        _ => "unknown",
    }
}

#[async_trait]
impl IntelSource for Isc {
    fn name(&self) -> &str {
        "ISC-SANS"
    }

    fn description(&self) -> &str {
        "SANS Internet Storm Center threat level"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let data = self.client.fetch_json(INFOCON_URL).await?;

        let status = data
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let severity = threat_level_severity(status);

        let mut signals = Vec::new();
        match status.to_lowercase().as_str() {
            "red" => signals.push("ISC INFOCON RED: Critical internet threat level".to_string()),
            "orange" => {
                signals.push("ISC INFOCON ORANGE: High internet threat level".to_string())
            }
            "yellow" => {
                signals.push("ISC INFOCON YELLOW: Elevated internet threat level".to_string())
            }
            _ => signals.push(format!("ISC INFOCON {}: Normal conditions", status.to_uppercase())),
        }

        Ok(json!({
            "source": "ISC-SANS",
            "timestamp": Utc::now().to_rfc3339(),
            "infocon": {
                "status": status,
                "severity": severity,
            },
            "signals": signals,
        }))
    }
}
