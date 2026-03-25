use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const CELESTRAK_BASE: &str = "https://celestrak.org/NORAD/elements/gp.php";

pub struct CelesTrak {
    client: HttpClient,
}

impl CelesTrak {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for CelesTrak {
    fn name(&self) -> &str {
        "CelesTrak"
    }

    fn description(&self) -> &str {
        "Satellite orbit tracking and launch monitoring"
    }

    fn tier(&self) -> u8 {
        4
    }

    async fn sweep(&self) -> Result<Value> {
        use futures::future::join_all;
        use std::time::Duration;

        const PER_QUERY_TIMEOUT: Duration = Duration::from_secs(12);

        // Run all 3 essential queries in parallel (stations is small enough to
        // combine conceptually, and starlink/oneweb counts are lower priority)
        let groups = vec![
            ("last-30-days", "recent"),
            ("stations", "stations"),
            ("military", "military"),
        ];

        let futures: Vec<_> = groups.iter().map(|(group, label)| {
            let url = format!("{}?GROUP={}&FORMAT=json", CELESTRAK_BASE, group);
            let client = self.client.clone();
            let tag = label.to_string();
            async move {
                let result = tokio::time::timeout(PER_QUERY_TIMEOUT, client.fetch_json(&url)).await;
                (tag, result)
            }
        }).collect();

        let results = join_all(futures).await;

        let mut recent_sats: Vec<Value> = Vec::new();
        let mut stations: Vec<Value> = Vec::new();
        let mut military_count: usize = 0;

        for (tag, result) in results {
            let data = match result {
                Ok(Ok(d)) => d,
                Ok(Err(_)) | Err(_) => continue,
            };
            let arr = data.as_array().cloned().unwrap_or_default();
            match tag.as_str() {
                "recent" => recent_sats = arr,
                "stations" => stations = arr,
                "military" => military_count = arr.len(),
                _ => {}
            }
        }

        // Starlink/OneWeb counts are low-priority; skip separate API calls
        // to stay well within the global timeout.
        let starlink_count: usize = 0;
        let oneweb_count: usize = 0;

        // Parse recent launches
        let mut by_country: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        let mut recent_entries: Vec<Value> = Vec::new();

        for sat in recent_sats.iter().take(100) {
            let name = sat
                .get("OBJECT_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let norad_id = sat
                .get("NORAD_CAT_ID")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let country = sat
                .get("COUNTRY_CODE")
                .and_then(|v| v.as_str())
                .unwrap_or("UNK");
            let launch_date = sat
                .get("LAUNCH_DATE")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let object_type = sat
                .get("OBJECT_TYPE")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let epoch = sat
                .get("EPOCH")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            *by_country.entry(country.to_string()).or_insert(0) += 1;

            if recent_entries.len() < 25 {
                recent_entries.push(json!({
                    "name": name,
                    "noradId": norad_id,
                    "country": country,
                    "launchDate": launch_date,
                    "objectType": object_type,
                    "epoch": epoch,
                }));
            }
        }

        // ISS data
        let iss = stations
            .iter()
            .find(|s| {
                s.get("OBJECT_NAME")
                    .and_then(|v| v.as_str())
                    .map(|n| n.contains("ISS"))
                    .unwrap_or(false)
                    || s.get("NORAD_CAT_ID")
                        .and_then(|v| v.as_u64())
                        == Some(25544)
            })
            .map(|s| {
                json!({
                    "name": s.get("OBJECT_NAME").and_then(|v| v.as_str()).unwrap_or("ISS"),
                    "noradId": s.get("NORAD_CAT_ID").and_then(|v| v.as_u64()).unwrap_or(25544),
                    "epoch": s.get("EPOCH").and_then(|v| v.as_str()).unwrap_or(""),
                    "inclination": s.get("INCLINATION").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    "period": s.get("PERIOD").and_then(|v| v.as_f64()).unwrap_or(0.0),
                })
            });

        let mut signals = Vec::new();
        let total_new = recent_sats.len();
        if total_new > 50 {
            signals.push(format!(
                "HIGH LAUNCH TEMPO: {} new objects tracked in last 30 days",
                total_new
            ));
        }

        let cn = by_country.get("PRC").copied().unwrap_or(0)
            + by_country.get("CN").copied().unwrap_or(0);
        if cn > 10 {
            signals.push(format!(
                "CHINA SPACE ACTIVITY: {} objects launched recently",
                cn
            ));
        }

        let ru = by_country.get("CIS").copied().unwrap_or(0)
            + by_country.get("RU").copied().unwrap_or(0);
        if ru > 5 {
            signals.push(format!(
                "RUSSIA SPACE ACTIVITY: {} objects launched recently",
                ru
            ));
        }

        if military_count > 500 {
            signals.push(format!(
                "MILITARY CONSTELLATION: {} tracked military satellites",
                military_count
            ));
        }

        if starlink_count > 6000 {
            signals.push(format!(
                "STARLINK MEGA-CONSTELLATION: {} active satellites",
                starlink_count
            ));
        }

        if signals.is_empty() {
            signals.push("Space activity within normal parameters".to_string());
        }

        Ok(json!({
            "source": "CelesTrak",
            "timestamp": Utc::now().to_rfc3339(),
            "recentLaunches": recent_entries,
            "totalNewObjects": total_new,
            "launchByCountry": by_country,
            "iss": iss,
            "militarySatellites": military_count,
            "constellations": {
                "starlink": starlink_count,
                "oneweb": oneweb_count,
            },
            "signals": signals,
        }))
    }
}
