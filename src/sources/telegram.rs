use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use crate::util::strip_html;
use super::IntelSource;

/// Per-channel timeout for Telegram web preview scraping.
/// Must be short enough that all batches complete within the global
/// SOURCE_TIMEOUT_SECS (default 30s). With 8 channels in batches of 3,
/// that is 3 rounds -- so ~8s per round leaves headroom.
///
/// If ALL channels time out, Telegram web preview is likely blocked from
/// this IP. Alternative: use a Telegram Bot API with TELEGRAM_BOT_TOKEN
/// for direct channel access, or use a proxy.
const CHANNEL_TIMEOUT: Duration = Duration::from_secs(8);

struct Channel {
    id: &'static str,
    label: &'static str,
    topic: &'static str,
}

const CHANNELS: &[Channel] = &[
    Channel { id: "intelslava", label: "Intel Slava Z", topic: "conflict" },
    Channel { id: "ryaborig", label: "Rybar Original", topic: "conflict" },
    Channel { id: "SputnikInt", label: "Sputnik International", topic: "geopolitics" },
    Channel { id: "nexaborig", label: "Nexta Original", topic: "conflict" },
    Channel { id: "UkraineNow", label: "Ukraine Now", topic: "conflict" },
    Channel { id: "OSINTdefender", label: "OSINT Defender", topic: "osint" },
    Channel { id: "MilitaryBBa", label: "Military Observer", topic: "conflict" },
    Channel { id: "liveuamap", label: "Liveuamap", topic: "conflict" },
];

const URGENT_KEYWORDS: &[&str] = &[
    "breaking", "urgent", "alert", "confirmed", "just in",
    "missile", "strike", "explosion", "airstrike", "drone", "bombardment",
    "shelling", "intercept", "nuclear", "chemical", "ceasefire",
    "escalation", "invasion", "offensive", "mobilization",
    "nato", "coup", "assassination", "sanctions",
    "casualties", "killed", "evacuation",
    "blackout", "sabotage", "cyberattack",
    "attack",
];

pub struct Telegram {
    client: HttpClient,
}

impl Telegram {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    async fn scrape_channel(&self, channel: &Channel) -> Value {
        let url = format!("https://t.me/s/{}", channel.id);

        // Use a per-channel timeout so one slow/blocked channel cannot stall
        // the entire sweep.
        let fut = self.client.raw_client()
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .header("Accept-Language", "en-US,en;q=0.9")
            .timeout(CHANNEL_TIMEOUT)
            .send();

        let html = match fut.await {
            Ok(resp) => match resp.text().await {
                Ok(t) => t,
                Err(_) => {
                    return json!({
                        "channel": channel.id,
                        "title": channel.label,
                        "error": "failed to read response body",
                        "postCount": 0,
                    });
                }
            },
            Err(e) => {
                let msg = if e.is_timeout() {
                    format!("timed out after {}s", CHANNEL_TIMEOUT.as_secs())
                } else {
                    e.to_string()
                };
                return json!({
                    "channel": channel.id,
                    "title": channel.label,
                    "error": msg,
                    "postCount": 0,
                });
            }
        };

        let posts = parse_web_preview(&html, channel.id);
        let post_count = posts.len();
        let urgent_count = posts.iter().filter(|p| {
            p.get("urgentFlags")
                .and_then(|v| v.as_array())
                .map(|a| !a.is_empty())
                .unwrap_or(false)
        }).count();

        json!({
            "channel": channel.id,
            "title": channel.label,
            "topic": channel.topic,
            "postCount": post_count,
            "urgentCount": urgent_count,
            "posts": posts,
        })
    }
}

/// Parse messages from Telegram web preview HTML.
fn parse_web_preview(html: &str, channel_id: &str) -> Vec<Value> {
    let mut posts = Vec::new();
    let marker = "data-post=\"";

    let mut search_from = 0;
    while let Some(pos) = html[search_from..].find(marker) {
        let abs_pos = search_from + pos + marker.len();
        let post_id_end = match html[abs_pos..].find('"') {
            Some(e) => abs_pos + e,
            None => break,
        };
        let post_id = &html[abs_pos..post_id_end];

        // Find the block boundary (next data-post or end)
        let block_end = html[post_id_end..]
            .find(marker)
            .map(|p| post_id_end + p)
            .unwrap_or(html.len());
        let block = &html[post_id_end..block_end];

        // Extract message text
        let text_marker = "tgme_widget_message_text";
        let text = if let Some(tm_pos) = block.find(text_marker) {
            // Find the closing > of the div tag
            let after_marker = &block[tm_pos..];
            if let Some(gt_pos) = after_marker.find('>') {
                let content_start = tm_pos + gt_pos + 1;
                // Find closing </div>
                if let Some(div_end) = block[content_start..].find("</div>") {
                    let raw = &block[content_start..content_start + div_end];
                    let cleaned = raw.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
                    let stripped = strip_html(&cleaned);
                    let trimmed = stripped.trim();
                    if trimmed.len() > 300 {
                        trimmed.chars().take(300).collect::<String>()
                    } else {
                        trimmed.to_string()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Extract datetime
        let datetime = extract_attr(block, "datetime");

        // Extract views
        let views = if let Some(vp) = block.find("tgme_widget_message_views") {
            let after = &block[vp..];
            if let Some(gt) = after.find('>') {
                let content_start = vp + gt + 1;
                if let Some(end) = block[content_start..].find("</") {
                    let raw = block[content_start..content_start + end].trim().to_string();
                    parse_view_count(&raw)
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        // Detect urgency
        let urgent_flags = detect_urgency(&text);

        if !text.is_empty() && posts.len() < 20 {
            posts.push(json!({
                "postId": post_id,
                "text": text,
                "date": datetime,
                "views": views,
                "channel": channel_id,
                "urgentFlags": urgent_flags,
            }));
        }

        search_from = block_end;
    }

    posts
}

fn extract_attr(block: &str, attr: &str) -> String {
    let pattern = format!("{}=\"", attr);
    if let Some(pos) = block.find(&pattern) {
        let start = pos + pattern.len();
        if let Some(end) = block[start..].find('"') {
            return block[start..start + end].to_string();
        }
    }
    String::new()
}

fn parse_view_count(raw: &str) -> u64 {
    let s = raw.trim();
    if s.ends_with('K') {
        let num: f64 = s[..s.len() - 1].parse().unwrap_or(0.0);
        (num * 1000.0) as u64
    } else if s.ends_with('M') {
        let num: f64 = s[..s.len() - 1].parse().unwrap_or(0.0);
        (num * 1_000_000.0) as u64
    } else {
        s.parse().unwrap_or(0)
    }
}

fn detect_urgency(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    URGENT_KEYWORDS
        .iter()
        .filter(|kw| lower.contains(*kw))
        .map(|kw| kw.to_string())
        .collect()
}

#[async_trait]
impl IntelSource for Telegram {
    fn name(&self) -> &str {
        "Telegram"
    }

    fn description(&self) -> &str {
        "OSINT channel monitoring (public web preview)"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let mut channel_results = Vec::new();
        let mut total_posts = 0u32;
        let mut total_urgent = 0u32;
        let mut errors = Vec::new();

        // Fetch channels sequentially with small batches to avoid rate-limiting
        for chunk in CHANNELS.chunks(3) {
            let futures: Vec<_> = chunk.iter().map(|ch| self.scrape_channel(ch)).collect();
            let results = futures::future::join_all(futures).await;

            for r in results {
                let pc = r.get("postCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let uc = r.get("urgentCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                total_posts += pc;
                total_urgent += uc;

                if r.get("error").is_some() {
                    errors.push(json!({
                        "channel": r.get("channel"),
                        "error": r.get("error"),
                    }));
                }

                channel_results.push(r);
            }
        }

        let mut out = json!({
            "source": "Telegram",
            "timestamp": Utc::now().to_rfc3339(),
            "method": "Public channel web preview scraping (no auth required)",
            "channelsMonitored": CHANNELS.len(),
            "totalPosts": total_posts,
            "urgentPosts": total_urgent,
            "channels": channel_results,
        });

        if !errors.is_empty() {
            out.as_object_mut()
                .unwrap()
                .insert("errors".to_string(), json!(errors));

            // If ALL channels failed, add guidance about alternatives.
            if errors.len() == CHANNELS.len() {
                out.as_object_mut().unwrap().insert(
                    "note".to_string(),
                    json!("All Telegram channels failed. Web preview may be blocked from this IP. Alternative: use Telegram Bot API with TELEGRAM_BOT_TOKEN for direct channel access, or use a proxy."),
                );
            }
        }

        Ok(out)
    }
}
