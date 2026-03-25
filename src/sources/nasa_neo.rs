use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

pub struct NasaNeo {
    client: HttpClient,
}

impl NasaNeo {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for NasaNeo {
    fn name(&self) -> &str {
        "NASA-NEO"
    }

    fn description(&self) -> &str {
        "NASA Near Earth Object close approach tracking"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let api_key = std::env::var("NASA_API_KEY").unwrap_or_else(|_| "DEMO_KEY".to_string());

        let today = Utc::now().format("%Y-%m-%d").to_string();
        let url = format!(
            "https://api.nasa.gov/neo/rest/v1/feed?api_key={}&start_date={}",
            api_key, today
        );

        let data = self.client.fetch_json(&url).await?;

        let element_count = data
            .get("element_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let neo_objects = data.get("near_earth_objects").and_then(|v| v.as_object());

        let mut objects = Vec::new();
        let mut hazardous_count = 0u32;
        let mut signals = Vec::new();

        if let Some(days) = neo_objects {
            for (_date, day_arr) in days {
                let arr = match day_arr.as_array() {
                    Some(a) => a,
                    None => continue,
                };
                for neo in arr {
                    let name = neo
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let is_hazardous = neo
                        .get("is_potentially_hazardous_asteroid")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if is_hazardous {
                        hazardous_count += 1;
                    }

                    // Estimated diameter (km)
                    let diameter_min = neo
                        .get("estimated_diameter")
                        .and_then(|d| d.get("kilometers"))
                        .and_then(|k| k.get("estimated_diameter_min"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let diameter_max = neo
                        .get("estimated_diameter")
                        .and_then(|d| d.get("kilometers"))
                        .and_then(|k| k.get("estimated_diameter_max"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);

                    // Close approach data
                    let approach = neo
                        .get("close_approach_data")
                        .and_then(|v| v.as_array())
                        .and_then(|a| a.first());

                    let miss_distance_km = approach
                        .and_then(|a| a.get("miss_distance"))
                        .and_then(|m| m.get("kilometers"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);

                    let velocity_kmh = approach
                        .and_then(|a| a.get("relative_velocity"))
                        .and_then(|r| r.get("kilometers_per_hour"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);

                    let close_approach_date = approach
                        .and_then(|a| a.get("close_approach_date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if objects.len() < 30 {
                        objects.push(json!({
                            "name": name,
                            "hazardous": is_hazardous,
                            "diameterMinKm": diameter_min,
                            "diameterMaxKm": diameter_max,
                            "missDistanceKm": miss_distance_km,
                            "velocityKmh": velocity_kmh,
                            "closeApproachDate": close_approach_date,
                        }));
                    }
                }
            }
        }

        if hazardous_count > 0 {
            signals.push(format!(
                "{} potentially hazardous asteroid(s) approaching",
                hazardous_count
            ));
        }
        // Check for very close approaches (< 1 lunar distance = ~384400 km)
        for obj in &objects {
            let dist = obj
                .get("missDistanceKm")
                .and_then(|v| v.as_f64())
                .unwrap_or(f64::MAX);
            if dist < 384_400.0 {
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                signals.push(format!(
                    "CLOSE APPROACH: {} passing within 1 lunar distance ({:.0} km)",
                    name, dist
                ));
            }
        }
        if signals.is_empty() {
            signals.push("No significant near-Earth object threats".to_string());
        }

        Ok(json!({
            "source": "NASA-NEO",
            "timestamp": Utc::now().to_rfc3339(),
            "elementCount": element_count,
            "hazardousCount": hazardous_count,
            "objects": objects,
            "signals": signals,
        }))
    }
}
