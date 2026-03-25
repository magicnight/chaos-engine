use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

struct Chokepoint {
    key: &'static str,
    label: &'static str,
    lat: f64,
    lon: f64,
    note: &'static str,
}

const CHOKEPOINTS: &[Chokepoint] = &[
    Chokepoint { key: "straitOfHormuz", label: "Strait of Hormuz", lat: 26.5, lon: 56.5, note: "20% of world oil" },
    Chokepoint { key: "suezCanal", label: "Suez Canal", lat: 30.5, lon: 32.3, note: "12% of world trade" },
    Chokepoint { key: "straitOfMalacca", label: "Strait of Malacca", lat: 2.5, lon: 101.5, note: "25% of world trade" },
    Chokepoint { key: "babElMandeb", label: "Bab el-Mandeb", lat: 12.6, lon: 43.3, note: "Red Sea gateway" },
    Chokepoint { key: "taiwanStrait", label: "Taiwan Strait", lat: 24.0, lon: 119.0, note: "88% of largest container ships" },
    Chokepoint { key: "bosporusStrait", label: "Bosphorus", lat: 41.1, lon: 29.1, note: "Black Sea access" },
    Chokepoint { key: "panamaCanal", label: "Panama Canal", lat: 9.1, lon: -79.7, note: "5% of world trade" },
    Chokepoint { key: "capeOfGoodHope", label: "Cape of Good Hope", lat: -34.4, lon: 18.5, note: "Suez alternative" },
];

pub struct Ships {
    #[allow(dead_code)] // will be used when AIS WebSocket listener is implemented
    client: HttpClient,
}

impl Ships {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for Ships {
    fn name(&self) -> &str {
        "Ships"
    }

    fn description(&self) -> &str {
        "Maritime AIS vessel tracking"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let has_key = std::env::var("AISSTREAM_API_KEY")
            .map(|k| !k.is_empty())
            .unwrap_or(false);

        if !has_key {
            return Ok(json!({
                "source": "Maritime/AIS",
                "timestamp": Utc::now().to_rfc3339(),
                "error": "AISSTREAM_API_KEY required for real-time vessel tracking. Free at https://aisstream.io",
                "status": "no_key",
                "chokepoints": CHOKEPOINTS.iter().map(|cp| json!({
                    "key": cp.key,
                    "label": cp.label,
                    "lat": cp.lat,
                    "lon": cp.lon,
                    "note": cp.note,
                })).collect::<Vec<_>>(),
                "monitoringCapabilities": [
                    "Dark ship detection (AIS transponder shutoffs)",
                    "Sanctions evasion (ship-to-ship transfers)",
                    "Naval deployment tracking",
                    "Port congestion (vessel dwell time)",
                    "Chokepoint traffic anomalies",
                ],
            }));
        }

        // With key present, report ready status.
        // AIS stream uses WebSocket for real-time data; the sweep reports
        // configuration and chokepoint metadata.
        let chokepoint_info: Vec<Value> = CHOKEPOINTS
            .iter()
            .map(|cp| {
                json!({
                    "key": cp.key,
                    "label": cp.label,
                    "lat": cp.lat,
                    "lon": cp.lon,
                    "note": cp.note,
                })
            })
            .collect();

        Ok(json!({
            "source": "Maritime/AIS",
            "timestamp": Utc::now().to_rfc3339(),
            "status": "ready",
            "message": "AIS stream key configured. Use WebSocket listener for real-time data.",
            "chokepoints": chokepoint_info,
        }))
    }
}
