use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const BULLETIN_URL: &str =
    "https://climate.copernicus.eu/sites/default/files/custom-uploads/C3S-monthly-climate-bulletin/latest.json";

pub struct Copernicus {
    client: HttpClient,
}

impl Copernicus {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for Copernicus {
    fn name(&self) -> &str {
        "Copernicus"
    }

    fn description(&self) -> &str {
        "Copernicus Climate Change Service monthly bulletin"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let data = match self.client.fetch_json(BULLETIN_URL).await {
            Ok(d) => d,
            Err(e) => {
                return Ok(json!({
                    "source": "Copernicus",
                    "timestamp": Utc::now().to_rfc3339(),
                    "error": format!("Failed to fetch Copernicus data: {}", e),
                    "hint": "The public bulletin endpoint may require registration. Visit https://climate.copernicus.eu for access.",
                }));
            }
        };

        // Extract whatever structure is available
        let title = data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let period = data
            .get("period")
            .or_else(|| data.get("date"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let summary = data
            .get("summary")
            .or_else(|| data.get("description"))
            .or_else(|| data.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Temperature anomaly if available
        let temp_anomaly = data
            .get("global_temperature_anomaly")
            .or_else(|| data.get("temperatureAnomaly"))
            .and_then(|v| v.as_f64());

        let mut signals = Vec::new();
        if let Some(anomaly) = temp_anomaly {
            if anomaly > 1.5 {
                signals.push(format!(
                    "CRITICAL: Global temperature anomaly at +{:.2}C — above 1.5C threshold",
                    anomaly
                ));
            } else if anomaly > 1.0 {
                signals.push(format!(
                    "ELEVATED: Global temperature anomaly at +{:.2}C",
                    anomaly
                ));
            }
        }

        if !summary.is_empty() && signals.is_empty() {
            let truncated = if summary.len() > 200 {
                format!(
                    "{}...",
                    summary.chars().take(200).collect::<String>().trim_end()
                )
            } else {
                summary.to_string()
            };
            signals.push(truncated);
        }

        if signals.is_empty() {
            signals.push("Climate bulletin data retrieved".to_string());
        }

        Ok(json!({
            "source": "Copernicus",
            "timestamp": Utc::now().to_rfc3339(),
            "title": title,
            "period": period,
            "summary": if summary.len() > 500 {
                format!("{}...", summary.chars().take(500).collect::<String>().trim_end())
            } else {
                summary.to_string()
            },
            "globalTemperatureAnomaly": temp_anomaly,
            "rawData": data,
            "signals": signals,
        }))
    }
}
