// EU Sanctions — via EU Sanctions Map API
// https://www.sanctionsmap.eu/
// No auth required

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const REGIME_URL: &str = "https://www.sanctionsmap.eu/api/v1/regime";

pub struct EuSanctions {
    client: HttpClient,
}

impl EuSanctions {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for EuSanctions {
    fn name(&self) -> &str { "EUSanctions" }
    fn description(&self) -> &str { "EU sanctions regimes via Sanctions Map" }
    fn tier(&self) -> u8 { 3 }

    async fn sweep(&self) -> Result<Value> {
        let data = match self.client.fetch_json(REGIME_URL).await {
            Ok(d) => d,
            Err(e) => return Ok(json!({
                "source": self.name(),
                "timestamp": Utc::now().to_rfc3339(),
                "error": format!("EU Sanctions Map API unavailable: {}", e),
                "hint": "Alternative: check https://www.sanctionsmap.eu/ directly"
            })),
        };

        let regimes = data.as_array().map(|arr| {
            arr.iter().map(|r| {
                json!({
                    "name": r.get("programme_name").and_then(|v| v.as_str()).unwrap_or(""),
                    "country": r.get("country_description").and_then(|v| v.as_str()),
                    "adopted": r.get("adoption_date").and_then(|v| v.as_str()),
                    "url": r.get("url").and_then(|v| v.as_str()),
                })
            }).collect::<Vec<_>>()
        }).unwrap_or_default();

        Ok(json!({
            "source": self.name(),
            "timestamp": Utc::now().to_rfc3339(),
            "totalRegimes": regimes.len(),
            "regimes": regimes,
        }))
    }
}
