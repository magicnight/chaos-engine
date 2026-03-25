use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

// Statuspage-compatible APIs (all free, no key required)
const SERVICES: &[(&str, &str)] = &[
    ("GitHub", "https://www.githubstatus.com/api/v2/status.json"),
    ("Cloudflare", "https://www.cloudflarestatus.com/api/v2/status.json"),
    ("Discord", "https://discordstatus.com/api/v2/status.json"),
    ("OpenAI", "https://status.openai.com/api/v2/status.json"),
    ("Vercel", "https://www.vercel-status.com/api/v2/status.json"),
];

pub struct TechStatus {
    client: HttpClient,
}

impl TechStatus {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for TechStatus {
    fn name(&self) -> &str {
        "TechStatus"
    }

    fn description(&self) -> &str {
        "Major tech platform operational status"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let mut statuses = Vec::new();
        let mut incidents = 0u32;
        let mut signals = Vec::new();

        for (name, url) in SERVICES {
            match self.client.fetch_json(url).await {
                Ok(data) => {
                    let indicator = data
                        .get("status")
                        .and_then(|s| s.get("indicator"))
                        .and_then(|i| i.as_str())
                        .unwrap_or("unknown");
                    let description = data
                        .get("status")
                        .and_then(|s| s.get("description"))
                        .and_then(|d| d.as_str())
                        .unwrap_or("Unknown");

                    let is_degraded = indicator != "none";
                    if is_degraded {
                        incidents += 1;
                        signals.push(format!("{} degraded: {}", name, description));
                    }

                    statuses.push(json!({
                        "service": name,
                        "indicator": indicator,
                        "description": description,
                        "operational": !is_degraded,
                    }));
                }
                Err(_) => {
                    statuses.push(json!({
                        "service": name,
                        "indicator": "unknown",
                        "description": "Status check failed",
                        "operational": false,
                    }));
                }
            }
        }

        if signals.is_empty() {
            signals.push(format!("All {} monitored services operational", statuses.len()));
        }

        Ok(json!({
            "source": "TechStatus",
            "timestamp": Utc::now().to_rfc3339(),
            "totalServices": statuses.len(),
            "incidents": incidents,
            "statuses": statuses,
            "signals": signals,
        }))
    }
}
