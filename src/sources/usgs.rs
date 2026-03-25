use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const USGS_URL: &str = "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_day.geojson";

pub struct Usgs {
    client: HttpClient,
}

impl Usgs {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for Usgs {
    fn name(&self) -> &str {
        "USGS"
    }

    fn description(&self) -> &str {
        "USGS earthquake monitoring (M2.5+)"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let data = self.client.fetch_json(USGS_URL).await?;

        let features = data
            .get("features")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let total_quakes = features.len();
        let mut max_magnitude: f64 = 0.0;
        let mut tsunami_warnings: u32 = 0;
        let mut significant = Vec::new();

        for feature in &features {
            let props = match feature.get("properties") {
                Some(p) => p,
                None => continue,
            };
            let geom = match feature.get("geometry") {
                Some(g) => g,
                None => continue,
            };

            let mag = props.get("mag").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let place = props
                .get("place")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let tsunami = props.get("tsunami").and_then(|v| v.as_i64()).unwrap_or(0) != 0;

            let coords = geom.get("coordinates").and_then(|v| v.as_array());
            let lon = coords
                .and_then(|c| c.first())
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let lat = coords
                .and_then(|c| c.get(1))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            if mag > max_magnitude {
                max_magnitude = mag;
            }
            if tsunami {
                tsunami_warnings += 1;
            }

            if mag >= 5.0 && significant.len() < 20 {
                significant.push(json!({
                    "mag": mag,
                    "place": place,
                    "tsunami": tsunami,
                    "lat": lat,
                    "lon": lon,
                }));
            }
        }

        Ok(json!({
            "source": "USGS",
            "timestamp": Utc::now().to_rfc3339(),
            "totalQuakes": total_quakes,
            "maxMagnitude": max_magnitude,
            "tsunamiWarnings": tsunami_warnings,
            "quakes": significant,
        }))
    }
}
