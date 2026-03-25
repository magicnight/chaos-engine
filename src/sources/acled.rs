use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const TOKEN_URL: &str = "https://acleddata.com/oauth/token";
const API_BASE: &str = "https://acleddata.com/api/acled/read";

pub struct Acled {
    client: HttpClient,
}

impl Acled {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for Acled {
    fn name(&self) -> &str {
        "ACLED"
    }

    fn description(&self) -> &str {
        "Armed Conflict Location & Event Data"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let email = std::env::var("ACLED_EMAIL").unwrap_or_default();
        let password = std::env::var("ACLED_PASSWORD").unwrap_or_default();

        if email.is_empty() || password.is_empty() {
            return Ok(json!({
                "source": "ACLED",
                "timestamp": Utc::now().to_rfc3339(),
                "error": "ACLED_EMAIL and ACLED_PASSWORD required. Register at https://acleddata.com/user/register",
            }));
        }

        // OAuth2 password grant to obtain access token
        let token_resp = self
            .client
            .raw_client()
            .post(TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "username={}&password={}&grant_type=password&client_id=acled",
                urlencod(&email),
                urlencod(&password),
            ))
            .send()
            .await?;

        let token_text = token_resp.text().await?;
        let token_data: Value = serde_json::from_str(&token_text)?;

        let access_token = match token_data.get("access_token").and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => {
                return Ok(json!({
                    "source": "ACLED",
                    "timestamp": Utc::now().to_rfc3339(),
                    "error": format!("ACLED auth failed: {}", token_text.chars().take(300).collect::<String>()),
                }));
            }
        };

        // Fetch last 7 days of events
        let url = format!(
            "{}?_format=json&limit=2000",
            API_BASE,
        );

        let resp = self
            .client
            .raw_client()
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "CHAOS/3.0")
            .send()
            .await?;

        let text = resp.text().await?;
        let data: Value = serde_json::from_str(&text)?;

        let events = data
            .get("data")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let total_events = events.len();
        let mut total_fatalities: u64 = 0;
        let mut by_type: serde_json::Map<String, Value> = serde_json::Map::new();
        let mut by_country: serde_json::Map<String, Value> = serde_json::Map::new();
        let mut deadliest = Vec::new();

        for event in &events {
            let event_type = event
                .get("event_type")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let country = event
                .get("country")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let fatalities_str = event
                .get("fatalities")
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            let fatalities: u64 = fatalities_str.parse().unwrap_or(0);

            total_fatalities += fatalities;

            // Count by type
            let type_entry = by_type
                .entry(event_type.to_string())
                .or_insert_with(|| json!({"count": 0, "fatalities": 0}));
            if let Some(obj) = type_entry.as_object_mut() {
                let c = obj.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                let f = obj.get("fatalities").and_then(|v| v.as_u64()).unwrap_or(0);
                obj.insert("count".to_string(), json!(c + 1));
                obj.insert("fatalities".to_string(), json!(f + fatalities));
            }

            // Count by country
            let country_entry = by_country
                .entry(country.to_string())
                .or_insert_with(|| json!({"count": 0, "fatalities": 0}));
            if let Some(obj) = country_entry.as_object_mut() {
                let c = obj.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                let f = obj.get("fatalities").and_then(|v| v.as_u64()).unwrap_or(0);
                obj.insert("count".to_string(), json!(c + 1));
                obj.insert("fatalities".to_string(), json!(f + fatalities));
            }

            // Collect deadliest events
            if fatalities > 0 && deadliest.len() < 15 {
                let lat = event
                    .get("latitude")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let lon = event
                    .get("longitude")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let location = event
                    .get("location")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let date = event
                    .get("event_date")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                deadliest.push(json!({
                    "date": date,
                    "type": event_type,
                    "country": country,
                    "location": location,
                    "fatalities": fatalities,
                    "lat": lat,
                    "lon": lon,
                }));
            }
        }

        // Sort deadliest by fatalities descending
        deadliest.sort_by(|a, b| {
            let fa = a.get("fatalities").and_then(|v| v.as_u64()).unwrap_or(0);
            let fb = b.get("fatalities").and_then(|v| v.as_u64()).unwrap_or(0);
            fb.cmp(&fa)
        });

        Ok(json!({
            "source": "ACLED",
            "timestamp": Utc::now().to_rfc3339(),
            "totalEvents": total_events,
            "totalFatalities": total_fatalities,
            "byType": by_type,
            "byCountry": by_country,
            "deadliestEvents": deadliest,
        }))
    }
}

/// Minimal percent-encoding for URL form data.
fn urlencod(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
    }
    out
}
