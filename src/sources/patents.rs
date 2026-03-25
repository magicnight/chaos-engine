use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const PATENTS_API: &str = "https://search.patentsview.org/api/v1/patent/";

const STRATEGIC_DOMAINS: &[(&str, &str, &str)] = &[
    ("ai", "Artificial Intelligence", "artificial intelligence machine learning deep learning neural network"),
    ("quantum", "Quantum Computing", "quantum computing quantum processor qubit"),
    ("biotech", "Biotechnology", "synthetic biology gene editing CRISPR mRNA"),
    ("semiconductor", "Semiconductor", "semiconductor integrated circuit lithography chip fabrication"),
    ("energy", "Energy Technology", "nuclear fusion solar energy battery storage"),
    ("defense", "Defense Technology", "hypersonic directed energy weapon railgun"),
    ("space", "Space Technology", "satellite space launch orbital anti-satellite"),
];

pub struct Patents {
    client: HttpClient,
}

impl Patents {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn compact_patent(p: &Value) -> Value {
    json!({
        "id": p.get("patent_id").and_then(|v| v.as_str()).unwrap_or(""),
        "title": p.get("patent_title").and_then(|v| v.as_str()).unwrap_or(""),
        "date": p.get("patent_date").and_then(|v| v.as_str()).unwrap_or(""),
        "assignee": p.get("assignee_organization").and_then(|v| v.as_str()).unwrap_or("Unknown"),
    })
}

#[async_trait]
impl IntelSource for Patents {
    fn name(&self) -> &str {
        "USPTO-Patents"
    }

    fn description(&self) -> &str {
        "USPTO patent filings in strategic technology areas"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let since = (Utc::now() - chrono::Duration::days(90))
            .format("%Y-%m-%d")
            .to_string();

        let mut domain_results = serde_json::Map::new();
        let mut total_found = 0u32;
        let mut signals = Vec::new();

        for (key, label, terms) in STRATEGIC_DOMAINS {
            let q = json!({
                "_and": [
                    {"_gte": {"patent_date": since}},
                    {"_text_any": {"patent_abstract": terms}},
                ]
            });
            let f = json!([
                "patent_id", "patent_title", "patent_date",
                "patent_abstract", "assignee_organization", "patent_type"
            ]);
            let o = json!({"patent_date": "desc"});

            let resp = self
                .client
                .raw_client()
                .get(PATENTS_API)
                .query(&[
                    ("q", q.to_string()),
                    ("f", f.to_string()),
                    ("o", o.to_string()),
                    ("s", "10".to_string()),
                ])
                .send()
                .await;

            let result = match resp {
                Ok(r) => {
                    let text = r.text().await.unwrap_or_default();
                    serde_json::from_str::<Value>(&text).map_err(|e| anyhow::anyhow!("{}", e))
                }
                Err(e) => Err(anyhow::anyhow!("{}", e)),
            };

            let patents: Vec<Value> = match result {
                Ok(data) => {
                    let arr = data
                        .get("patents")
                        .or_else(|| data.get("results"))
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    arr.iter().map(compact_patent).collect()
                }
                Err(_) => Vec::new(),
            };

            total_found += patents.len() as u32;

            // Check for high-activity assignees
            let mut assignee_counts: std::collections::HashMap<String, u32> =
                std::collections::HashMap::new();
            for p in &patents {
                let assignee = p
                    .get("assignee")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                if assignee != "Unknown" {
                    *assignee_counts.entry(assignee.to_string()).or_insert(0) += 1;
                }
            }
            for (org, count) in &assignee_counts {
                if *count >= 3 {
                    signals.push(format!(
                        "HIGH ACTIVITY: {} filed {} {} patents in last 90 days",
                        org, count, label
                    ));
                }
            }

            domain_results.insert(key.to_string(), json!(patents));
        }

        if signals.is_empty() {
            signals.push(
                "No unusual patent filing patterns detected in strategic domains".to_string(),
            );
        }

        Ok(json!({
            "source": "USPTO-Patents",
            "timestamp": Utc::now().to_rfc3339(),
            "searchWindow": format!("{} to {}", since, Utc::now().format("%Y-%m-%d")),
            "totalFound": total_found,
            "domains": domain_results,
            "signals": signals,
        }))
    }
}
