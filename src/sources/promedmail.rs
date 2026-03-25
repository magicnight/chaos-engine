use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const PROMED_RSS_URL: &str = "https://promedmail.org/feed/";

pub struct PromedMail {
    client: HttpClient,
}

impl PromedMail {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

/// Extract text content between XML tags using simple string search.
fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let start_pos = xml.find(&open)?;
    let after_open = &xml[start_pos + open.len()..];
    let gt_pos = after_open.find('>')?;
    let content_start = start_pos + open.len() + gt_pos + 1;

    let content_end = xml[content_start..].find(&close)?;
    let raw = &xml[content_start..content_start + content_end];

    let trimmed = raw.trim();
    if trimmed.starts_with("<![CDATA[") && trimmed.ends_with("]]>") {
        Some(trimmed[9..trimmed.len() - 3].to_string())
    } else {
        Some(trimmed.to_string())
    }
}

/// Extract all <item> blocks from RSS XML.
fn extract_items(xml: &str) -> Vec<String> {
    let mut items = Vec::new();
    let open_tag = "<item>";
    let close_tag = "</item>";
    let mut search_from = 0;

    while let Some(start) = xml[search_from..].find(open_tag) {
        let abs_start = search_from + start + open_tag.len();
        if let Some(end) = xml[abs_start..].find(close_tag) {
            items.push(xml[abs_start..abs_start + end].to_string());
        }
        search_from = abs_start;
    }

    items
}

/// Strip HTML/XML tags from text.
fn strip_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut inside = false;
    for ch in input.chars() {
        match ch {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => result.push(ch),
            _ => {}
        }
    }
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .trim()
        .to_string()
}

#[async_trait]
impl IntelSource for PromedMail {
    fn name(&self) -> &str {
        "ProMED"
    }

    fn description(&self) -> &str {
        "ProMED-mail disease outbreak early warning"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let xml = match self.client.fetch_text(PROMED_RSS_URL).await {
            Ok(text) => text,
            Err(e) => {
                return Ok(json!({
                    "source": "ProMED",
                    "timestamp": Utc::now().to_rfc3339(),
                    "error": format!("Failed to fetch ProMED RSS: {}", e),
                    "hint": "ProMED may block automated requests or be temporarily unavailable.",
                }));
            }
        };

        let item_blocks = extract_items(&xml);
        let mut alerts = Vec::new();
        let mut disease_mentions: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        // Known disease keywords to track
        let disease_keywords = [
            "avian influenza",
            "h5n1",
            "h7n9",
            "ebola",
            "marburg",
            "cholera",
            "dengue",
            "mpox",
            "monkeypox",
            "measles",
            "polio",
            "plague",
            "anthrax",
            "covid",
            "sars",
            "mers",
            "nipah",
            "hendra",
            "lassa",
            "yellow fever",
            "rabies",
            "meningitis",
            "tuberculosis",
        ];

        for item_xml in item_blocks.iter().take(25) {
            let title = extract_xml_tag(item_xml, "title").unwrap_or_default();
            let description = extract_xml_tag(item_xml, "description")
                .map(|s| strip_tags(&s))
                .unwrap_or_default();
            let link = extract_xml_tag(item_xml, "link").unwrap_or_default();
            let pub_date = extract_xml_tag(item_xml, "pubDate").unwrap_or_default();

            let title_lower = title.to_lowercase();
            for keyword in &disease_keywords {
                if title_lower.contains(keyword) {
                    *disease_mentions.entry(keyword.to_string()).or_insert(0) += 1;
                }
            }

            let truncated = if description.len() > 300 {
                format!(
                    "{}...",
                    description
                        .chars()
                        .take(300)
                        .collect::<String>()
                        .trim_end()
                )
            } else {
                description
            };

            alerts.push(json!({
                "title": title,
                "description": truncated,
                "link": link,
                "pubDate": pub_date,
            }));
        }

        let mut signals = Vec::new();
        for (disease, count) in &disease_mentions {
            if *count >= 3 {
                signals.push(format!(
                    "ELEVATED DISEASE ACTIVITY: {} mentioned in {} alerts",
                    disease, count
                ));
            } else if *count >= 1 {
                signals.push(format!(
                    "Disease mention: {} ({} alert(s))",
                    disease, count
                ));
            }
        }
        if signals.is_empty() {
            if alerts.is_empty() {
                signals.push("No ProMED alerts available".to_string());
            } else {
                signals.push(format!("{} disease alerts tracked", alerts.len()));
            }
        }

        Ok(json!({
            "source": "ProMED",
            "timestamp": Utc::now().to_rfc3339(),
            "totalAlerts": alerts.len(),
            "diseaseMentions": disease_mentions,
            "alerts": alerts,
            "signals": signals,
        }))
    }
}
