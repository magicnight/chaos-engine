use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const OFAC_SDN_URL: &str =
    "https://sanctionslistservice.ofac.treas.gov/api/PublicationPreview/exports/SDN.XML";
const OPENSANCTIONS_URL: &str = "https://api.opensanctions.org/search/default";

const BRIEFING_QUERIES: &[&str] = &[
    "Iran", "Russia", "North Korea", "Syria", "Venezuela", "Wagner",
];

pub struct Sanctions {
    client: HttpClient,
}

impl Sanctions {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

/// Extract text between XML tags using simple string search.
fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

/// Count occurrences of a case-insensitive pattern.
fn count_occurrences(text: &str, pattern: &str) -> usize {
    let lower = text.to_lowercase();
    let pat = pattern.to_lowercase();
    lower.matches(&pat).count()
}

#[async_trait]
impl IntelSource for Sanctions {
    fn name(&self) -> &str {
        "Sanctions"
    }

    fn description(&self) -> &str {
        "OFAC SDN + OpenSanctions monitoring"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        // --- OFAC SDN ---
        let ofac = match self.client.fetch_text(OFAC_SDN_URL).await {
            Ok(xml) => {
                let publish_date = extract_tag(&xml, "Publish_Date")
                    .or_else(|| extract_tag(&xml, "publish_date"));
                let entry_count = count_occurrences(&xml, "<sdnEntry>");
                let record_count = extract_tag(&xml, "Record_Count")
                    .or_else(|| extract_tag(&xml, "records_count"))
                    .and_then(|s| s.parse::<u64>().ok());

                json!({
                    "publishDate": publish_date,
                    "entryCount": entry_count,
                    "recordCount": record_count,
                    "dataAvailable": !xml.is_empty(),
                })
            }
            Err(e) => {
                json!({
                    "error": format!("OFAC fetch failed: {}", e),
                })
            }
        };

        // --- OpenSanctions ---
        let mut search_results = Vec::new();
        let mut total_sanctioned: u64 = 0;

        for query in BRIEFING_QUERIES {
            let url = format!(
                "{}?q={}&limit=10&topics=sanction",
                OPENSANCTIONS_URL, query
            );
            match self.client.fetch_json(&url).await {
                Ok(data) => {
                    let total = data
                        .get("total")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    total_sanctioned += total;

                    let entities: Vec<Value> = data
                        .get("results")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .take(5)
                                .map(|e| {
                                    json!({
                                        "id": e.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                                        "name": e.get("caption").and_then(|v| v.as_str()).unwrap_or(""),
                                        "schema": e.get("schema").and_then(|v| v.as_str()).unwrap_or(""),
                                        "datasets": e.get("datasets"),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    search_results.push(json!({
                        "query": query,
                        "totalResults": total,
                        "entities": entities,
                    }));
                }
                Err(_) => {
                    search_results.push(json!({
                        "query": query,
                        "error": "fetch failed",
                    }));
                }
            }
        }

        Ok(json!({
            "source": "Sanctions",
            "timestamp": Utc::now().to_rfc3339(),
            "ofac": ofac,
            "openSanctions": {
                "totalSanctionedEntities": total_sanctioned,
                "searches": search_results,
                "monitoringTargets": BRIEFING_QUERIES,
            },
        }))
    }
}
