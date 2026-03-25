use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

// NOTE: GDELT aggressively rate-limits (HTTP 429) and blocks non-browser
// User-Agents (HTTP 403).  We use a browser-like UA below.  If you still
// get 429s, consider adding a 1-2 second delay between sweeps or reducing
// maxrecords.  The GKG endpoint or the GDELT Events CSV export are more
// lenient alternatives.
const GDELT_URL: &str = "https://api.gdeltproject.org/api/v2/doc/doc?query=conflict%20OR%20military%20OR%20economy%20OR%20crisis%20OR%20war%20OR%20sanctions%20OR%20tariff&mode=ArtList&maxrecords=50&timespan=24h&format=json&sort=DateDesc";

// Browser-like User-Agent; GDELT blocks non-browser UAs with 403.
const BROWSER_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

const CONFLICT_KEYWORDS: &[&str] = &[
    "military", "conflict", "war", "strike", "missile", "attack", "bomb", "troops",
];
const ECONOMY_KEYWORDS: &[&str] = &[
    "economy", "recession", "inflation", "market", "sanctions", "tariff", "trade", "gdp",
];
const HEALTH_KEYWORDS: &[&str] = &[
    "pandemic", "outbreak", "epidemic", "disease", "virus", "health",
];
const CRISIS_KEYWORDS: &[&str] = &[
    "crisis", "disaster", "emergency", "refugee", "famine",
];

pub struct Gdelt {
    client: HttpClient,
}

impl Gdelt {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn categorize_title(title: &str) -> Vec<&'static str> {
    let lower = title.to_lowercase();
    let mut cats = Vec::new();
    if CONFLICT_KEYWORDS.iter().any(|k| lower.contains(k)) {
        cats.push("conflicts");
    }
    if ECONOMY_KEYWORDS.iter().any(|k| lower.contains(k)) {
        cats.push("economy");
    }
    if HEALTH_KEYWORDS.iter().any(|k| lower.contains(k)) {
        cats.push("health");
    }
    if CRISIS_KEYWORDS.iter().any(|k| lower.contains(k)) {
        cats.push("crisis");
    }
    cats
}

#[async_trait]
impl IntelSource for Gdelt {
    fn name(&self) -> &str {
        "GDELT"
    }

    fn description(&self) -> &str {
        "Global news events (100+ languages)"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        // GDELT blocks the default UA with 403; use a browser-like UA.
        // NOTE: GDELT may block some IPs entirely. Alternative: use the
        // GDELT GKG (Global Knowledge Graph) endpoint or the GDELT Events
        // CSV export at https://blog.gdeltproject.org/gdelt-2-0-our-global-world-in-realtime/
        let resp = self
            .client
            .raw_client()
            .get(GDELT_URL)
            .header("User-Agent", BROWSER_UA)
            .header("Accept", "application/json")
            .send()
            .await
            .context("GDELT request failed")?;

        let status = resp.status();
        let body = resp.text().await.context("GDELT: failed to read response body")?;

        if !status.is_success() || body.trim().is_empty() {
            return Ok(json!({
                "source": "GDELT",
                "timestamp": Utc::now().to_rfc3339(),
                "error": format!("GDELT API returned HTTP {} (body length {})", status.as_u16(), body.len()),
                "note": "GDELT may block some IPs. Alternative: use GDELT GKG endpoint or GDELT Events CSV export.",
                "totalArticles": 0,
            }));
        }

        let data: Value = serde_json::from_str(&body).unwrap_or_else(|_| {
            json!({ "articles": [] })
        });

        let articles = data
            .get("articles")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let total = articles.len();
        let mut all_articles = Vec::new();
        let mut conflicts = Vec::new();
        let mut economy = Vec::new();
        let mut health = Vec::new();
        let mut crisis = Vec::new();

        for article in &articles {
            let title = article
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let url = article
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let seendate = article
                .get("seendate")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let domain = article
                .get("domain")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let language = article
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let source_country = article
                .get("sourcecountry")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let entry = json!({
                "title": title,
                "url": url,
                "seendate": seendate,
                "domain": domain,
                "language": language,
                "sourcecountry": source_country,
            });

            all_articles.push(entry.clone());

            let cats = categorize_title(title);
            for cat in cats {
                match cat {
                    "conflicts" => conflicts.push(entry.clone()),
                    "economy" => economy.push(entry.clone()),
                    "health" => health.push(entry.clone()),
                    "crisis" => crisis.push(entry.clone()),
                    _ => {}
                }
            }
        }

        Ok(json!({
            "source": "GDELT",
            "timestamp": Utc::now().to_rfc3339(),
            "totalArticles": total,
            "allArticles": all_articles,
            "conflicts": conflicts,
            "economy": economy,
            "health": health,
            "crisis": crisis,
        }))
    }
}
