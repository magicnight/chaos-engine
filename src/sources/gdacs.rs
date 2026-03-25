use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const GDACS_RSS_URL: &str = "https://www.gdacs.org/xml/rss.xml";

pub struct Gdacs {
    client: HttpClient,
}

impl Gdacs {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

/// Extract text content between XML tags using simple string search.
/// Handles CDATA sections: <![CDATA[...]]>
fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let start_pos = xml.find(&open)?;
    // Find the closing > of the opening tag (skip attributes)
    let after_open = &xml[start_pos + open.len()..];
    let gt_pos = after_open.find('>')?;
    let content_start = start_pos + open.len() + gt_pos + 1;

    let content_end = xml[content_start..].find(&close)?;
    let raw = &xml[content_start..content_start + content_end];

    // Handle CDATA
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

/// Parse an RSS item block into structured data.
fn parse_item(item_xml: &str) -> Value {
    let title = extract_xml_tag(item_xml, "title").unwrap_or_default();
    let description = extract_xml_tag(item_xml, "description").unwrap_or_default();
    let link = extract_xml_tag(item_xml, "link").unwrap_or_default();
    let pub_date = extract_xml_tag(item_xml, "pubDate").unwrap_or_default();

    // GDACS-specific fields (use gdacs: namespace prefix)
    let alert_level = extract_xml_tag(item_xml, "gdacs:alertlevel")
        .or_else(|| extract_xml_tag(item_xml, "gdacs:alertLevel"))
        .unwrap_or_default();
    let severity = extract_xml_tag(item_xml, "gdacs:severity")
        .unwrap_or_default();
    let event_type = extract_xml_tag(item_xml, "gdacs:eventtype")
        .or_else(|| extract_xml_tag(item_xml, "gdacs:eventType"))
        .unwrap_or_default();
    let country = extract_xml_tag(item_xml, "gdacs:country")
        .unwrap_or_default();
    let population = extract_xml_tag(item_xml, "gdacs:population")
        .unwrap_or_default();

    // Extract coordinates from geo:lat / geo:long or georss:point
    let lat = extract_xml_tag(item_xml, "geo:lat")
        .and_then(|s| s.parse::<f64>().ok());
    let lon = extract_xml_tag(item_xml, "geo:long")
        .and_then(|s| s.parse::<f64>().ok());

    // Fallback: georss:point is "lat lon"
    let (lat, lon) = match (lat, lon) {
        (Some(la), Some(lo)) => (Some(la), Some(lo)),
        _ => {
            if let Some(point) = extract_xml_tag(item_xml, "georss:point") {
                let parts: Vec<&str> = point.split_whitespace().collect();
                let la = parts.first().and_then(|s| s.parse::<f64>().ok());
                let lo = parts.get(1).and_then(|s| s.parse::<f64>().ok());
                (la, lo)
            } else {
                (None, None)
            }
        }
    };

    // Strip HTML from description
    let clean_desc = strip_tags(&description);
    let truncated = if clean_desc.len() > 300 {
        format!("{}...", clean_desc.chars().take(300).collect::<String>().trim_end())
    } else {
        clean_desc
    };

    json!({
        "title": title,
        "description": truncated,
        "link": link,
        "pubDate": pub_date,
        "alertLevel": alert_level,
        "severity": severity,
        "eventType": event_type,
        "country": country,
        "population": population,
        "lat": lat,
        "lon": lon,
    })
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
impl IntelSource for Gdacs {
    fn name(&self) -> &str {
        "GDACS"
    }

    fn description(&self) -> &str {
        "Global Disaster Alert and Coordination System"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let xml = self.client.fetch_text(GDACS_RSS_URL).await?;
        let item_blocks = extract_items(&xml);
        let alerts: Vec<Value> = item_blocks.iter().take(30).map(|b| parse_item(b)).collect();

        // Count by alert level
        let mut red = 0u32;
        let mut orange = 0u32;
        let mut green = 0u32;

        for alert in &alerts {
            let level = alert
                .get("alertLevel")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match level.to_lowercase().as_str() {
                "red" => red += 1,
                "orange" => orange += 1,
                "green" => green += 1,
                _ => {}
            }
        }

        let mut signals = Vec::new();
        if red > 0 {
            signals.push(format!("RED ALERT: {} high-severity disaster events", red));
        }
        if orange > 0 {
            signals.push(format!("ORANGE ALERT: {} moderate-severity events", orange));
        }
        if signals.is_empty() {
            signals.push("No high-severity disaster alerts active".to_string());
        }

        Ok(json!({
            "source": "GDACS",
            "timestamp": Utc::now().to_rfc3339(),
            "totalAlerts": alerts.len(),
            "summary": {
                "red": red,
                "orange": orange,
                "green": green,
            },
            "alerts": alerts,
            "signals": signals,
        }))
    }
}
