use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const NOAA_URL: &str =
    "https://api.weather.gov/alerts/active?status=actual&severity=Extreme,Severe&limit=50";

pub struct Noaa {
    client: HttpClient,
}

impl Noaa {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn categorize_event(event: &str) -> &'static str {
    let lower = event.to_lowercase();
    if lower.contains("hurricane") || lower.contains("typhoon") || lower.contains("tropical") {
        "hurricanes"
    } else if lower.contains("tornado") {
        "tornadoes"
    } else if lower.contains("flood") {
        "floods"
    } else if lower.contains("blizzard") || lower.contains("ice storm") || lower.contains("winter")
    {
        "winterStorms"
    } else if lower.contains("fire") {
        "wildfires"
    } else {
        "other"
    }
}

fn extract_centroid(geometry: &Value) -> (f64, f64) {
    let geo_type = geometry
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match geo_type {
        "Point" => {
            let coords = geometry.get("coordinates").and_then(|v| v.as_array());
            let lon = coords
                .and_then(|c| c.first())
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let lat = coords
                .and_then(|c| c.get(1))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            (lat, lon)
        }
        "Polygon" => {
            if let Some(rings) = geometry
                .get("coordinates")
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_array())
            {
                let (mut sum_lat, mut sum_lon) = (0.0_f64, 0.0_f64);
                let mut count = 0u32;
                for point in rings {
                    if let Some(pair) = point.as_array() {
                        let lon = pair.first().and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let lat = pair.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        sum_lat += lat;
                        sum_lon += lon;
                        count += 1;
                    }
                }
                if count > 0 {
                    (sum_lat / count as f64, sum_lon / count as f64)
                } else {
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            }
        }
        _ => (0.0, 0.0),
    }
}

#[async_trait]
impl IntelSource for Noaa {
    fn name(&self) -> &str {
        "NOAA"
    }

    fn description(&self) -> &str {
        "NWS severe weather alerts"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let resp = self
            .client
            .raw_client()
            .get(NOAA_URL)
            .header("Accept", "application/geo+json")
            .header("User-Agent", "CHAOS/3.0 (intel-engine)")
            .send()
            .await?;

        let text = resp.text().await?;
        let data: Value = serde_json::from_str(&text)?;

        let features = data
            .get("features")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let total = features.len();
        let mut hurricanes = 0u32;
        let mut tornadoes = 0u32;
        let mut floods = 0u32;
        let mut winter_storms = 0u32;
        let mut wildfires = 0u32;
        let mut other = 0u32;
        let mut top_alerts = Vec::new();

        for feature in &features {
            let props = match feature.get("properties") {
                Some(p) => p,
                None => continue,
            };

            let event = props
                .get("event")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let severity = props
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let urgency = props
                .get("urgency")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let headline = props
                .get("headline")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let areas = props
                .get("areaDesc")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let category = categorize_event(event);
            match category {
                "hurricanes" => hurricanes += 1,
                "tornadoes" => tornadoes += 1,
                "floods" => floods += 1,
                "winterStorms" => winter_storms += 1,
                "wildfires" => wildfires += 1,
                _ => other += 1,
            }

            if top_alerts.len() < 20 {
                let (lat, lon) = feature
                    .get("geometry")
                    .filter(|g| !g.is_null())
                    .map(|g| extract_centroid(g))
                    .unwrap_or((0.0, 0.0));

                top_alerts.push(json!({
                    "event": event,
                    "severity": severity,
                    "urgency": urgency,
                    "headline": headline,
                    "areas": areas,
                    "lat": (lat * 10000.0).round() / 10000.0,
                    "lon": (lon * 10000.0).round() / 10000.0,
                }));
            }
        }

        Ok(json!({
            "source": "NOAA/NWS",
            "timestamp": Utc::now().to_rfc3339(),
            "totalSevereAlerts": total,
            "summary": {
                "hurricanes": hurricanes,
                "tornadoes": tornadoes,
                "floods": floods,
                "winterStorms": winter_storms,
                "wildfires": wildfires,
                "other": other,
            },
            "topAlerts": top_alerts,
        }))
    }
}
