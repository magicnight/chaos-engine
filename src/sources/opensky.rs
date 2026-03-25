use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const BASE_URL: &str = "https://opensky-network.org/api/states/all";

// NOTE: OpenSky rate-limits aggressively — anonymous users get ~100 API
// credits/day (each bounding-box query costs 4 credits).  Authenticated
// users get ~4000/day.  We keep the hotspot list small (4 regions) to
// stay within anonymous limits on a single sweep.  If you have OpenSky
// credentials, set OPENSKY_USER / OPENSKY_PASS and consider adding more
// regions back.

struct Hotspot {
    key: &'static str,
    label: &'static str,
    lamin: f64,
    lomin: f64,
    lamax: f64,
    lomax: f64,
}

const HOTSPOTS: &[Hotspot] = &[
    Hotspot { key: "middleEast", label: "Middle East", lamin: 12.0, lomin: 30.0, lamax: 42.0, lomax: 65.0 },
    Hotspot { key: "taiwan", label: "Taiwan Strait", lamin: 20.0, lomin: 115.0, lamax: 28.0, lomax: 125.0 },
    Hotspot { key: "ukraine", label: "Ukraine Region", lamin: 44.0, lomin: 22.0, lamax: 53.0, lomax: 41.0 },
    Hotspot { key: "southChinaSea", label: "South China Sea", lamin: 5.0, lomin: 105.0, lamax: 23.0, lomax: 122.0 },
];

pub struct OpenSky {
    client: HttpClient,
}

impl OpenSky {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    async fn fetch_region(&self, hotspot: &Hotspot) -> Value {
        let url = format!(
            "{}?lamin={}&lomin={}&lamax={}&lomax={}",
            BASE_URL, hotspot.lamin, hotspot.lomin, hotspot.lamax, hotspot.lomax,
        );

        match self.client.fetch_json(&url).await {
            Ok(data) => {
                let states = data
                    .get("states")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                let total = states.len();
                let mut by_country: serde_json::Map<String, Value> = serde_json::Map::new();
                let mut no_callsign = 0u32;
                let mut high_altitude = 0u32;

                for state in &states {
                    let arr = match state.as_array() {
                        Some(a) => a,
                        None => continue,
                    };
                    // index 2 = origin_country
                    let country = arr
                        .get(2)
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let count = by_country
                        .entry(country.to_string())
                        .or_insert_with(|| json!(0));
                    if let Some(n) = count.as_u64() {
                        *count = json!(n + 1);
                    }

                    // index 1 = callsign
                    let callsign = arr
                        .get(1)
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if callsign.trim().is_empty() {
                        no_callsign += 1;
                    }

                    // index 7 = baro_altitude
                    let alt = arr
                        .get(7)
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    if alt > 12000.0 {
                        high_altitude += 1;
                    }
                }

                json!({
                    "region": hotspot.label,
                    "key": hotspot.key,
                    "totalAircraft": total,
                    "byCountry": by_country,
                    "noCallsign": no_callsign,
                    "highAltitude": high_altitude,
                })
            }
            Err(e) => {
                json!({
                    "region": hotspot.label,
                    "key": hotspot.key,
                    "totalAircraft": 0,
                    "error": e.to_string(),
                })
            }
        }
    }
}

#[async_trait]
impl IntelSource for OpenSky {
    fn name(&self) -> &str {
        "OpenSky"
    }

    fn description(&self) -> &str {
        "Real-time flight tracking via ADS-B"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let futures: Vec<_> = HOTSPOTS
            .iter()
            .map(|h| self.fetch_region(h))
            .collect();

        let results = futures::future::join_all(futures).await;

        let errors: Vec<&Value> = results
            .iter()
            .filter(|r| r.get("error").is_some())
            .collect();

        let error_info = if errors.len() == results.len() {
            Some(format!(
                "OpenSky unavailable across all hotspots: {}",
                errors
                    .first()
                    .and_then(|e| e.get("error"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
            ))
        } else if !errors.is_empty() {
            Some(format!(
                "OpenSky unavailable for {}/{} hotspots",
                errors.len(),
                results.len()
            ))
        } else {
            None
        };

        let mut out = json!({
            "source": "OpenSky",
            "timestamp": Utc::now().to_rfc3339(),
            "hotspots": results,
        });

        if let Some(err) = error_info {
            out.as_object_mut()
                .unwrap()
                .insert("error".to_string(), json!(err));
        }

        Ok(out)
    }
}
