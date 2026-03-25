use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const FIRMS_BASE: &str = "https://firms.modaps.eosdis.nasa.gov/api/area/csv";

struct HotspotRegion {
    key: &'static str,
    label: &'static str,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
}

const HOTSPOTS: &[HotspotRegion] = &[
    HotspotRegion { key: "middleEast", label: "Middle East", west: 30.0, south: 12.0, east: 65.0, north: 42.0 },
    HotspotRegion { key: "ukraine", label: "Ukraine", west: 22.0, south: 44.0, east: 41.0, north: 53.0 },
    HotspotRegion { key: "iran", label: "Iran", west: 44.0, south: 25.0, east: 63.0, north: 40.0 },
    HotspotRegion { key: "sudanHorn", label: "Sudan / Horn of Africa", west: 21.0, south: 2.0, east: 52.0, north: 23.0 },
    HotspotRegion { key: "myanmar", label: "Myanmar", west: 92.0, south: 9.0, east: 102.0, north: 29.0 },
];

pub struct Firms {
    client: HttpClient,
}

impl Firms {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    async fn fetch_region(&self, key: &str, region: &HotspotRegion) -> Value {
        let api_key = match std::env::var("FIRMS_MAP_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => return json!({"region": region.label, "error": "no_key"}),
        };

        let url = format!(
            "{}/{}/VIIRS_SNPP_NRT/{},{},{},{}/2",
            FIRMS_BASE, api_key, region.west, region.south, region.east, region.north,
        );

        match self.client.fetch_text(&url).await {
            Ok(csv_text) => {
                let fires = parse_csv(&csv_text);
                analyze_fires(&fires, key, region.label)
            }
            Err(e) => {
                json!({
                    "region": region.label,
                    "key": key,
                    "error": e.to_string(),
                })
            }
        }
    }
}

/// Parse a CSV string into a list of key-value rows.
fn parse_csv(text: &str) -> Vec<Vec<(String, String)>> {
    let mut lines = text.lines();
    let headers: Vec<String> = match lines.next() {
        Some(h) => h.split(',').map(|s| s.trim().to_string()).collect(),
        None => return Vec::new(),
    };

    lines
        .map(|line| {
            let vals: Vec<&str> = line.split(',').collect();
            headers
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    (
                        h.clone(),
                        vals.get(i).unwrap_or(&"").trim().to_string(),
                    )
                })
                .collect()
        })
        .collect()
}

fn get_field<'a>(row: &'a [(String, String)], name: &str) -> &'a str {
    row.iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
        .unwrap_or("")
}

fn analyze_fires(fires: &[Vec<(String, String)>], key: &str, label: &str) -> Value {
    let total = fires.len();
    let mut high_conf = 0u32;
    let mut night_detections = 0u32;
    let mut high_intensity = Vec::new();

    for fire in fires {
        let confidence = get_field(fire, "confidence");
        if confidence == "h" || confidence == "high" {
            high_conf += 1;
        }

        let daynight = get_field(fire, "daynight");
        if daynight == "N" {
            night_detections += 1;
        }

        let frp: f64 = get_field(fire, "frp").parse().unwrap_or(0.0);
        if frp > 10.0 && high_intensity.len() < 15 {
            let lat: f64 = get_field(fire, "latitude").parse().unwrap_or(0.0);
            let lon: f64 = get_field(fire, "longitude").parse().unwrap_or(0.0);
            let brightness: f64 = get_field(fire, "bright_ti4").parse().unwrap_or(0.0);
            let date = get_field(fire, "acq_date");
            let time = get_field(fire, "acq_time");

            high_intensity.push(json!({
                "lat": lat,
                "lon": lon,
                "brightness": brightness,
                "frp": frp,
                "date": date,
                "time": time,
                "confidence": confidence,
                "daynight": daynight,
            }));
        }
    }

    // Sort high intensity by FRP descending
    high_intensity.sort_by(|a, b| {
        let fa = a.get("frp").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let fb = b.get("frp").and_then(|v| v.as_f64()).unwrap_or(0.0);
        fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
    });

    json!({
        "region": label,
        "key": key,
        "totalDetections": total,
        "highConfidence": high_conf,
        "nightDetections": night_detections,
        "highIntensity": high_intensity,
    })
}

#[async_trait]
impl IntelSource for Firms {
    fn name(&self) -> &str {
        "FIRMS"
    }

    fn description(&self) -> &str {
        "NASA satellite fire/strike detection"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let key = std::env::var("FIRMS_MAP_KEY").unwrap_or_default();
        if key.is_empty() {
            return Ok(json!({
                "source": "NASA FIRMS",
                "timestamp": Utc::now().to_rfc3339(),
                "error": "FIRMS_MAP_KEY required. Free at https://firms.modaps.eosdis.nasa.gov/api/area/",
            }));
        }

        let futures: Vec<_> = HOTSPOTS
            .iter()
            .map(|h| self.fetch_region(h.key, h))
            .collect();

        let hotspots = futures::future::join_all(futures).await;

        // Generate signals
        let mut signals = Vec::new();
        for h in &hotspots {
            let region = h.get("region").and_then(|v| v.as_str()).unwrap_or("");
            let hi_count = h
                .get("highIntensity")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let night = h
                .get("nightDetections")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if hi_count > 5 {
                signals.push(format!(
                    "HIGH INTENSITY FIRES in {}: {} detections >10MW FRP",
                    region, hi_count
                ));
            }
            if night > 20 {
                signals.push(format!(
                    "ELEVATED NIGHT ACTIVITY in {}: {} night detections",
                    region, night
                ));
            }
        }

        Ok(json!({
            "source": "NASA FIRMS",
            "timestamp": Utc::now().to_rfc3339(),
            "hotspots": hotspots,
            "signals": signals,
        }))
    }
}
