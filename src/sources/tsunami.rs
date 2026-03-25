use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const TSUNAMI_URL: &str = "https://www.tsunami.gov/events/xml/PAAQAtom.xml";

pub struct Tsunami {
    client: HttpClient,
}

impl Tsunami {
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

/// Extract all <entry> blocks from Atom XML.
fn extract_entries(xml: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let open_tag = "<entry>";
    let close_tag = "</entry>";
    let mut search_from = 0;

    while let Some(start) = xml[search_from..].find(open_tag) {
        let abs_start = search_from + start + open_tag.len();
        if let Some(end) = xml[abs_start..].find(close_tag) {
            entries.push(xml[abs_start..abs_start + end].to_string());
        }
        search_from = abs_start;
    }

    entries
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
impl IntelSource for Tsunami {
    fn name(&self) -> &str {
        "Tsunami"
    }

    fn description(&self) -> &str {
        "Pacific Tsunami Warning Center alerts"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let xml = self.client.fetch_text(TSUNAMI_URL).await?;
        let entry_blocks = extract_entries(&xml);

        let mut alerts = Vec::new();
        let mut warning_count = 0u32;
        let mut watch_count = 0u32;
        let mut advisory_count = 0u32;

        for entry_xml in entry_blocks.iter().take(20) {
            let title = extract_xml_tag(entry_xml, "title").unwrap_or_default();
            let summary = extract_xml_tag(entry_xml, "summary")
                .map(|s| strip_tags(&s))
                .unwrap_or_default();
            let updated = extract_xml_tag(entry_xml, "updated").unwrap_or_default();
            let link = {
                // Atom <link href="..."/>
                let marker = "href=\"";
                entry_xml
                    .find("<link")
                    .and_then(|pos| {
                        let after = &entry_xml[pos..];
                        after.find(marker).map(|hp| {
                            let start = hp + marker.len();
                            let end = after[start..].find('"').unwrap_or(0);
                            after[start..start + end].to_string()
                        })
                    })
                    .unwrap_or_default()
            };

            let title_lower = title.to_lowercase();
            if title_lower.contains("warning") {
                warning_count += 1;
            } else if title_lower.contains("watch") {
                watch_count += 1;
            } else if title_lower.contains("advisory") {
                advisory_count += 1;
            }

            let truncated = if summary.len() > 300 {
                format!(
                    "{}...",
                    summary.chars().take(300).collect::<String>().trim_end()
                )
            } else {
                summary
            };

            alerts.push(json!({
                "title": title,
                "summary": truncated,
                "updated": updated,
                "link": link,
            }));
        }

        let mut signals = Vec::new();
        if warning_count > 0 {
            signals.push(format!(
                "TSUNAMI WARNING: {} active warning(s)",
                warning_count
            ));
        }
        if watch_count > 0 {
            signals.push(format!(
                "TSUNAMI WATCH: {} active watch(es)",
                watch_count
            ));
        }
        if advisory_count > 0 {
            signals.push(format!(
                "TSUNAMI ADVISORY: {} advisory(ies)",
                advisory_count
            ));
        }
        if signals.is_empty() {
            signals.push("No active tsunami warnings or watches".to_string());
        }

        Ok(json!({
            "source": "Tsunami",
            "timestamp": Utc::now().to_rfc3339(),
            "totalAlerts": alerts.len(),
            "warnings": warning_count,
            "watches": watch_count,
            "advisories": advisory_count,
            "alerts": alerts,
            "signals": signals,
        }))
    }
}
