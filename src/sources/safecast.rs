use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

struct NuclearSite {
    key: &'static str,
    label: &'static str,
    lat: f64,
    lon: f64,
    radius: u32,
}

const NUCLEAR_SITES: &[NuclearSite] = &[
    NuclearSite { key: "fukushima", label: "Fukushima Daiichi", lat: 37.42, lon: 141.03, radius: 50 },
    NuclearSite { key: "chernobyl", label: "Chernobyl Exclusion Zone", lat: 51.39, lon: 30.1, radius: 50 },
    NuclearSite { key: "zaporizhzhia", label: "Zaporizhzhia NPP (Ukraine)", lat: 47.51, lon: 34.58, radius: 100 },
    NuclearSite { key: "sellafield", label: "Sellafield (UK)", lat: 54.42, lon: -3.49, radius: 50 },
    NuclearSite { key: "hanford", label: "Hanford (USA)", lat: 46.55, lon: -119.53, radius: 50 },
    NuclearSite { key: "laHague", label: "La Hague (France)", lat: 49.68, lon: -1.88, radius: 50 },
];

pub struct Safecast {
    client: HttpClient,
}

impl Safecast {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    async fn fetch_site(&self, site: &NuclearSite) -> Value {
        let url = format!(
            "https://api.safecast.org/measurements.json?distance={}&latitude={}&longitude={}&order=created_at+desc&per_page=5",
            site.radius * 1000,
            site.lat,
            site.lon,
        );

        match self.client.fetch_json(&url).await {
            Ok(data) => {
                let measurements = data.as_array().cloned().unwrap_or_default();
                let values: Vec<f64> = measurements
                    .iter()
                    .filter_map(|m| m.get("value").and_then(|v| v.as_f64()))
                    .collect();

                let avg_cpm = if values.is_empty() {
                    None
                } else {
                    Some(values.iter().sum::<f64>() / values.len() as f64)
                };

                let max_cpm = values.iter().cloned().fold(f64::NAN, f64::max);
                let max_cpm = if max_cpm.is_nan() { None } else { Some(max_cpm) };

                let last_reading = measurements
                    .first()
                    .and_then(|m| m.get("captured_at"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Normal background: 10-80 CPM. >100 CPM warrants attention.
                let anomaly = avg_cpm.map(|a| a > 100.0).unwrap_or(false);

                json!({
                    "site": site.label,
                    "key": site.key,
                    "recentReadings": values.len(),
                    "avgCPM": avg_cpm,
                    "maxCPM": max_cpm,
                    "anomaly": anomaly,
                    "lastReading": last_reading,
                })
            }
            Err(e) => {
                json!({
                    "site": site.label,
                    "key": site.key,
                    "recentReadings": 0,
                    "error": e.to_string(),
                })
            }
        }
    }
}

#[async_trait]
impl IntelSource for Safecast {
    fn name(&self) -> &str {
        "Safecast"
    }

    fn description(&self) -> &str {
        "Nuclear radiation monitoring network"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let futures: Vec<_> = NUCLEAR_SITES
            .iter()
            .map(|s| self.fetch_site(s))
            .collect();

        let results = futures::future::join_all(futures).await;

        let anomalies: Vec<String> = results
            .iter()
            .filter(|r| r.get("anomaly").and_then(|v| v.as_bool()).unwrap_or(false))
            .map(|r| {
                let site = r.get("site").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let avg = r.get("avgCPM").and_then(|v| v.as_f64()).unwrap_or(0.0);
                format!("ELEVATED RADIATION at {}: {:.1} CPM (normal: 10-80)", site, avg)
            })
            .collect();

        let signals = if anomalies.is_empty() {
            vec!["All monitored nuclear sites within normal radiation levels".to_string()]
        } else {
            anomalies
        };

        Ok(json!({
            "source": "Safecast",
            "timestamp": Utc::now().to_rfc3339(),
            "sites": results,
            "signals": signals,
        }))
    }
}
