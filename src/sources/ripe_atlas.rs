use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const PROBES_URL: &str = "https://atlas.ripe.net/api/v2/probes/?status=1&page_size=1";
const MEASUREMENTS_URL: &str =
    "https://atlas.ripe.net/api/v2/measurements/?status=2&page_size=5&sort=-start_time";

pub struct RipeAtlas {
    client: HttpClient,
}

impl RipeAtlas {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for RipeAtlas {
    fn name(&self) -> &str {
        "RIPEAtlas"
    }

    fn description(&self) -> &str {
        "RIPE Atlas global internet measurement network"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        use futures::future::join_all;

        let client = self.client.clone();

        let probes_future = {
            let c = client.clone();
            async move { c.fetch_json(PROBES_URL).await }
        };
        let measurements_future = {
            let c = client;
            async move { c.fetch_json(MEASUREMENTS_URL).await }
        };

        let results = join_all(vec![
            Box::pin(probes_future)
                as std::pin::Pin<Box<dyn std::future::Future<Output = _> + Send>>,
            Box::pin(measurements_future),
        ])
        .await;

        let mut active_probes: u64 = 0;
        let mut recent_measurements = Vec::new();
        let mut signals = Vec::new();

        // Parse probe count
        if let Some(Ok(data)) = results.first() {
            active_probes = data
                .get("count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }

        // Parse recent measurements
        if let Some(Ok(data)) = results.get(1) {
            if let Some(arr) = data.get("results").and_then(|v| v.as_array()) {
                for m in arr.iter().take(5) {
                    let id = m.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let mtype = m
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let description = m
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let participant_count = m
                        .get("participant_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    recent_measurements.push(json!({
                        "id": id,
                        "type": mtype,
                        "description": description,
                        "participantCount": participant_count,
                    }));
                }
            }
        }

        if active_probes < 10_000 {
            signals.push(format!(
                "LOW PROBE COUNT: {} active probes — possible widespread outage",
                active_probes
            ));
        }
        if signals.is_empty() {
            signals.push(format!(
                "Internet measurement network healthy: {} active probes",
                active_probes
            ));
        }

        Ok(json!({
            "source": "RIPEAtlas",
            "timestamp": Utc::now().to_rfc3339(),
            "activeProbes": active_probes,
            "recentMeasurements": recent_measurements,
            "signals": signals,
        }))
    }
}
