use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

// ReliefWeb now requires a registered appname for all API access.
// Register at: https://apidoc.reliefweb.int/register
// Set RELIEFWEB_APPNAME env var after registering.
// Alternative humanitarian data sources:
//   - UN OCHA HDX API: https://data.humdata.org/api/3/
//   - GDACS (already integrated) for disaster data
fn reliefweb_url() -> String {
    let appname = std::env::var("RELIEFWEB_APPNAME")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "chaos-engine".to_string());
    format!(
        "https://api.reliefweb.int/v1/reports?appname={}&limit=20&sort[]=date:desc&filter[field]=status&filter[value]=published",
        appname
    )
}

pub struct ReliefWeb {
    client: HttpClient,
}

impl ReliefWeb {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for ReliefWeb {
    fn name(&self) -> &str {
        "ReliefWeb"
    }

    fn description(&self) -> &str {
        "UN humanitarian crisis tracking"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let url = reliefweb_url();
        let data = match self.client.fetch_json(&url).await {
            Ok(d) => d,
            Err(e) => {
                let err_str = e.to_string();
                // ReliefWeb rejects unregistered appnames with 403.
                if err_str.contains("403") {
                    return Ok(json!({
                        "source": "ReliefWeb",
                        "timestamp": Utc::now().to_rfc3339(),
                        "error": "ReliefWeb API requires a registered appname (HTTP 403).",
                        "fix": "Register at https://apidoc.reliefweb.int/register and set RELIEFWEB_APPNAME env var.",
                        "alternatives": [
                            "UN OCHA HDX API: https://data.humdata.org/api/3/",
                            "GDACS (already integrated) for disaster data"
                        ],
                        "totalReports": 0,
                        "latestReports": [],
                    }));
                }
                return Err(e.into());
            }
        };

        let items = data
            .get("data")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut reports = Vec::new();

        for item in &items {
            let fields = match item.get("fields") {
                Some(f) => f,
                None => continue,
            };

            let title = fields
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let date = fields
                .pointer("/date/created")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let countries: Vec<&str> = fields
                .get("country")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
                        .collect()
                })
                .unwrap_or_default();

            let sources: Vec<&str> = fields
                .get("source")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
                        .collect()
                })
                .unwrap_or_default();

            let url_alias = fields
                .get("url_alias")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let url = if url_alias.is_empty() {
                String::new()
            } else {
                format!("https://reliefweb.int{}", url_alias)
            };

            reports.push(json!({
                "title": title,
                "date": date,
                "countries": countries,
                "source": sources,
                "url": url,
            }));
        }

        Ok(json!({
            "source": "ReliefWeb",
            "timestamp": Utc::now().to_rfc3339(),
            "totalReports": reports.len(),
            "latestReports": reports,
        }))
    }
}
