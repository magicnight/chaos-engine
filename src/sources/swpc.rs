use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const NOAA_SCALES_URL: &str = "https://services.swpc.noaa.gov/products/noaa-scales.json";

pub struct Swpc {
    client: HttpClient,
}

impl Swpc {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

/// Extract scale value from a NOAA scales entry (e.g., "Scale" -> "G2" or "minor").
fn extract_scale(entry: &Value, field: &str) -> (String, String) {
    let obj = entry.get(field);
    let scale = obj
        .and_then(|v| v.get("Scale"))
        .and_then(|v| v.as_str())
        .unwrap_or("none")
        .to_string();
    let text = obj
        .and_then(|v| v.get("Text"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    (scale, text)
}

#[async_trait]
impl IntelSource for Swpc {
    fn name(&self) -> &str {
        "SWPC"
    }

    fn description(&self) -> &str {
        "NOAA space weather monitoring"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let data = self.client.fetch_json(NOAA_SCALES_URL).await?;

        // The response is an array: index 0 = current, index 1 = predicted 24h
        let entries = data.as_array().cloned().unwrap_or_default();

        let current = entries.first();
        let predicted = entries.get(1);

        let date_stamp = current
            .and_then(|e| e.get("DateStamp"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let time_stamp = current
            .and_then(|e| e.get("TimeStamp"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let (r_scale, r_text) = current
            .map(|e| extract_scale(e, "R"))
            .unwrap_or_default();
        let (s_scale, s_text) = current
            .map(|e| extract_scale(e, "S"))
            .unwrap_or_default();
        let (g_scale, g_text) = current
            .map(|e| extract_scale(e, "G"))
            .unwrap_or_default();

        let (r_pred, r_pred_text) = predicted
            .map(|e| extract_scale(e, "R"))
            .unwrap_or_default();
        let (s_pred, s_pred_text) = predicted
            .map(|e| extract_scale(e, "S"))
            .unwrap_or_default();
        let (g_pred, g_pred_text) = predicted
            .map(|e| extract_scale(e, "G"))
            .unwrap_or_default();

        // Generate signals for elevated conditions
        let mut signals = Vec::new();
        for (name, scale, text) in &[
            ("Radio Blackout", &r_scale, &r_text),
            ("Solar Radiation", &s_scale, &s_text),
            ("Geomagnetic Storm", &g_scale, &g_text),
        ] {
            if scale.as_str() != "none" && scale.as_str() != "0" && !scale.is_empty() {
                signals.push(format!("{}: {} - {}", name, scale, text));
            }
        }
        if signals.is_empty() {
            signals.push("Space weather conditions nominal".to_string());
        }

        Ok(json!({
            "source": "SWPC",
            "timestamp": Utc::now().to_rfc3339(),
            "observationTime": format!("{} {}", date_stamp, time_stamp),
            "current": {
                "R": {"scale": r_scale, "text": r_text},
                "S": {"scale": s_scale, "text": s_text},
                "G": {"scale": g_scale, "text": g_text},
            },
            "predicted24h": {
                "R": {"scale": r_pred, "text": r_pred_text},
                "S": {"scale": s_pred, "text": s_pred_text},
                "G": {"scale": g_pred, "text": g_pred_text},
            },
            "signals": signals,
        }))
    }
}
