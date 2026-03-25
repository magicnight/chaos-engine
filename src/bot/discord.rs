use anyhow::Result;
use serde_json::{json, Value};
use std::time::Instant;

use super::{check_rate_limit, AlertRecord, AlertTier};
use crate::config::Config;

/// Discord alerter using REST API (webhook + optional bot token).
/// No gateway connection -- uses webhook for alerts, bot token for
/// channel messages via REST. Ported from `lib/alerts/discord.mjs`.
#[allow(dead_code)]
pub struct DiscordBot {
    client: reqwest::Client,
    bot_token: Option<String>,
    channel_id: Option<String>,
    guild_id: Option<String>,
    webhook_url: Option<String>,
    mute_until: Option<Instant>,
    alert_history: Vec<AlertRecord>,
}

impl DiscordBot {
    pub fn from_config(config: &Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            bot_token: config.discord_bot_token.clone(),
            channel_id: config.discord_channel_id.clone(),
            guild_id: config.discord_guild_id.clone(),
            webhook_url: config.discord_webhook_url.clone(),
            mute_until: None,
            alert_history: Vec::new(),
        }
    }

    /// Configured if we have (bot_token + channel_id) OR a webhook_url.
    pub fn is_configured(&self) -> bool {
        (self.bot_token.is_some() && self.channel_id.is_some()) || self.webhook_url.is_some()
    }

    /// Human-readable mode description for status display.
    pub fn mode(&self) -> &str {
        if self.bot_token.is_some() && self.channel_id.is_some() {
            "bot mode"
        } else if self.webhook_url.is_some() {
            "webhook mode"
        } else {
            "disabled"
        }
    }

    // -- Sending Messages -----------------------------------------------------

    /// Send a message with an optional embed.
    /// Tries bot token first, falls back to webhook URL.
    pub async fn send_message(
        &self,
        content: Option<&str>,
        embed: Option<DiscordEmbed>,
    ) -> Result<()> {
        if !self.is_configured() {
            return Ok(());
        }

        // Try bot token + channel_id first
        if let (Some(token), Some(channel_id)) = (&self.bot_token, &self.channel_id) {
            let url = format!(
                "https://discord.com/api/v10/channels/{}/messages",
                channel_id
            );
            let mut body = json!({});
            if let Some(c) = content {
                body["content"] = json!(c);
            }
            if let Some(ref e) = embed {
                body["embeds"] = json!([e.to_json()]);
            }

            let resp = self
                .client
                .post(&url)
                .header("Authorization", format!("Bot {}", token))
                .header("Content-Type", "application/json")
                .json(&body)
                .timeout(std::time::Duration::from_secs(15))
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => return Ok(()),
                Ok(r) => {
                    let status = r.status();
                    let text = r.text().await.unwrap_or_default();
                    eprintln!(
                        "[Discord] Bot send failed ({}): {}",
                        status,
                        &text[..text.len().min(200)]
                    );
                }
                Err(e) => {
                    eprintln!("[Discord] Bot send error: {}", e);
                }
            }
        }

        // Fallback: webhook URL
        if let Some(ref webhook_url) = self.webhook_url {
            return self.send_webhook(webhook_url, content, embed.as_ref()).await;
        }

        eprintln!("[Discord] Cannot send -- bot not ready and no webhook URL configured");
        Ok(())
    }

    async fn send_webhook(
        &self,
        url: &str,
        content: Option<&str>,
        embed: Option<&DiscordEmbed>,
    ) -> Result<()> {
        let mut body = json!({});
        if let Some(c) = content {
            body["content"] = json!(c);
        }
        if let Some(e) = embed {
            body["embeds"] = json!([e.to_json()]);
        }

        let resp = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await;

        match resp {
            Ok(r) if !r.status().is_success() => {
                let status = r.status();
                let text = r.text().await.unwrap_or_default();
                eprintln!(
                    "[Discord] Webhook failed ({}): {}",
                    status,
                    &text[..text.len().min(200)]
                );
            }
            Err(e) => {
                eprintln!("[Discord] Webhook error: {}", e);
            }
            _ => {}
        }
        Ok(())
    }

    // -- Alert Sending --------------------------------------------------------

    /// Send a tiered alert with a rich embed. Checks rate limits and mute state.
    pub async fn send_alert(
        &mut self,
        tier: &AlertTier,
        headline: &str,
        reason: &str,
        delta: &Value,
    ) -> Result<()> {
        if self.is_muted() {
            eprintln!("[Discord] Alerts muted, skipping");
            return Ok(());
        }
        if !check_rate_limit(tier, &self.alert_history) {
            eprintln!("[Discord] Rate limited for tier {}", tier.label());
            return Ok(());
        }

        let direction = delta
            .get("summary")
            .and_then(|s| s.get("direction"))
            .and_then(|v| v.as_str())
            .unwrap_or("mixed")
            .to_uppercase();

        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let mut fields = vec![
            ("Direction".to_string(), direction, true),
            ("Tier".to_string(), tier.label().to_string(), true),
        ];

        // Add signal count if available
        if let Some(signals) = delta.get("signals") {
            let new_count = signals
                .get("new")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let esc_count = signals
                .get("escalated")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            if new_count + esc_count > 0 {
                fields.push((
                    "Signals".to_string(),
                    format!("{} new, {} escalated", new_count, esc_count),
                    true,
                ));
            }
        }

        let embed = DiscordEmbed {
            title: format!("{} CHAOS {}", tier.emoji(), tier.label()),
            description: format!("**{}**\n\n{}", headline, reason),
            color: tier.color(),
            fields,
            footer: Some(format!("CHAOS Engine | {} UTC", timestamp)),
        };

        self.send_message(None, Some(embed)).await?;
        self.record_alert(tier);
        eprintln!("[Discord] {} alert sent: {}", tier.label(), headline);
        Ok(())
    }

    // -- Mute / Rate Limit ----------------------------------------------------

    #[allow(dead_code)]
    pub fn mute(&mut self, hours: f64) {
        let dur = std::time::Duration::from_secs_f64(hours * 3600.0);
        self.mute_until = Some(Instant::now() + dur);
    }

    #[allow(dead_code)]
    pub fn unmute(&mut self) {
        self.mute_until = None;
    }

    fn is_muted(&self) -> bool {
        match self.mute_until {
            Some(until) => Instant::now() < until,
            None => false,
        }
    }

    fn record_alert(&mut self, tier: &AlertTier) {
        self.alert_history.push(AlertRecord {
            tier: tier.label().to_string(),
            timestamp: now_millis(),
        });
        if self.alert_history.len() > 50 {
            let start = self.alert_history.len() - 50;
            self.alert_history = self.alert_history[start..].to_vec();
        }
    }
}

/// A Discord embed for rich message formatting.
pub struct DiscordEmbed {
    pub title: String,
    pub description: String,
    /// Colour as 24-bit integer (e.g. 0xFF0000 for red).
    pub color: u32,
    /// (name, value, inline) tuples.
    pub fields: Vec<(String, String, bool)>,
    pub footer: Option<String>,
}

impl DiscordEmbed {
    /// Serialize to the Discord API JSON embed format.
    pub fn to_json(&self) -> Value {
        let fields: Vec<Value> = self
            .fields
            .iter()
            .map(|(name, value, inline)| {
                json!({
                    "name": name,
                    "value": value,
                    "inline": inline,
                })
            })
            .collect();

        let mut embed = json!({
            "title": self.title,
            "description": self.description,
            "color": self.color,
            "fields": fields,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Some(ref footer) = self.footer {
            embed["footer"] = json!({ "text": footer });
        }

        embed
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(
        bot_token: Option<&str>,
        channel_id: Option<&str>,
        webhook_url: Option<&str>,
    ) -> Config {
        let mut cfg = Config {
            refresh_interval_minutes: 15,
            source_timeout_secs: 30,
            llm_provider: None,
            llm_api_key: None,
            llm_base_url: None,
            llm_model: None,
            ollama_url: None,
            ollama_model: None,
            sweep_lang: "en".to_string(),
            webhook_url: None,
            watch_regions: Vec::new(),
            alert_keywords: Vec::new(),
            watch_tickers: Vec::new(),
            telegram_bot_token: None,
            telegram_chat_id: None,
            telegram_poll_interval: 5000,
            fallback_provider: None,
            fallback_model: None,
            gemini_api_key: None,
            discord_bot_token: None,
            discord_channel_id: None,
            discord_guild_id: None,
            discord_webhook_url: None,
        };
        cfg.discord_bot_token = bot_token.map(String::from);
        cfg.discord_channel_id = channel_id.map(String::from);
        cfg.discord_webhook_url = webhook_url.map(String::from);
        cfg
    }

    #[test]
    fn test_not_configured_empty() {
        let cfg = make_config(None, None, None);
        let bot = DiscordBot::from_config(&cfg);
        assert!(!bot.is_configured());
        assert_eq!(bot.mode(), "disabled");
    }

    #[test]
    fn test_configured_bot_mode() {
        let cfg = make_config(Some("token"), Some("chan123"), None);
        let bot = DiscordBot::from_config(&cfg);
        assert!(bot.is_configured());
        assert_eq!(bot.mode(), "bot mode");
    }

    #[test]
    fn test_configured_webhook_mode() {
        let cfg = make_config(None, None, Some("https://hooks.example.com/test"));
        let bot = DiscordBot::from_config(&cfg);
        assert!(bot.is_configured());
        assert_eq!(bot.mode(), "webhook mode");
    }

    #[test]
    fn test_embed_to_json() {
        let embed = DiscordEmbed {
            title: "Test".to_string(),
            description: "Hello".to_string(),
            color: 0xFF0000,
            fields: vec![("Field1".to_string(), "Value1".to_string(), true)],
            footer: Some("Footer text".to_string()),
        };
        let json = embed.to_json();
        assert_eq!(json["title"], "Test");
        assert_eq!(json["color"], 0xFF0000);
        assert!(json["fields"].is_array());
        assert_eq!(json["footer"]["text"], "Footer text");
    }

    #[test]
    fn test_mute_unmute() {
        let cfg = make_config(None, None, Some("https://hook.test"));
        let mut bot = DiscordBot::from_config(&cfg);
        assert!(!bot.is_muted());

        bot.mute(1.0);
        assert!(bot.is_muted());

        bot.unmute();
        assert!(!bot.is_muted());
    }
}
