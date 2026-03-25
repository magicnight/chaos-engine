use anyhow::Result;
use serde_json::{json, Value};
use std::time::Instant;

use super::{check_rate_limit, AlertRecord, AlertTier, BotCommand, BOT_COMMANDS};

const TELEGRAM_API: &str = "https://api.telegram.org";
/// Telegram sendMessage text limit (characters).
const TELEGRAM_MAX_TEXT: usize = 4096;

/// Full two-way Telegram bot: polling for commands, tiered alert sending,
/// rate limiting, mute support. Ported from `lib/alerts/telegram.mjs`.
pub struct TelegramBot {
    client: reqwest::Client,
    token: String,
    chat_id: String,
    last_update_id: i64,
    mute_until: Option<Instant>,
    alert_history: Vec<AlertRecord>,
    bot_username: Option<String>,
}

impl TelegramBot {
    pub fn new(token: &str, chat_id: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            token: token.to_string(),
            chat_id: chat_id.to_string(),
            last_update_id: 0,
            mute_until: None,
            alert_history: Vec::new(),
            bot_username: None,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.token.is_empty() && !self.chat_id.is_empty()
    }

    #[allow(dead_code)]
    pub fn chat_id(&self) -> &str {
        &self.chat_id
    }

    // -- Core Messaging -------------------------------------------------------

    /// Send a Markdown-formatted message. Splits at 4096 chars to avoid
    /// Telegram API limits, preferring newline breaks.
    pub async fn send_message(&self, text: &str) -> Result<()> {
        if !self.is_configured() {
            return Ok(());
        }
        let chunks = chunk_text(text, TELEGRAM_MAX_TEXT);
        for chunk in &chunks {
            let url = format!("{}/bot{}/sendMessage", TELEGRAM_API, self.token);
            let body = json!({
                "chat_id": &self.chat_id,
                "text": chunk,
                "parse_mode": "Markdown",
                "disable_web_page_preview": true,
            });
            let resp = self
                .client
                .post(&url)
                .json(&body)
                .timeout(std::time::Duration::from_secs(15))
                .send()
                .await;
            match resp {
                Ok(r) if !r.status().is_success() => {
                    let status = r.status();
                    let text = r.text().await.unwrap_or_default();
                    eprintln!(
                        "[Telegram] Send failed ({}): {}",
                        status,
                        &text[..text.len().min(200)]
                    );
                }
                Err(e) => {
                    eprintln!("[Telegram] Send error: {}", e);
                }
                _ => {}
            }
        }
        Ok(())
    }

    // -- Alert Sending --------------------------------------------------------

    /// Send a tiered alert. Checks rate limits and mute state.
    pub async fn send_alert(
        &mut self,
        tier: &AlertTier,
        headline: &str,
        reason: &str,
        delta: &Value,
    ) -> Result<()> {
        if self.is_muted() {
            eprintln!("[Telegram] Alerts muted, skipping");
            return Ok(());
        }
        if !check_rate_limit(tier, &self.alert_history) {
            eprintln!("[Telegram] Rate limited for tier {}", tier.label());
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

        let message = format!(
            "{emoji} *CHAOS {label}*\n\n\
             *{headline}*\n\n\
             {reason}\n\n\
             Direction: {direction}\n\n\
             _{timestamp} UTC_",
            emoji = tier.emoji(),
            label = tier.label(),
        );

        self.send_message(&message).await?;
        self.record_alert(tier);
        eprintln!("[Telegram] {} alert sent: {}", tier.label(), headline);
        Ok(())
    }

    // -- Polling for Commands -------------------------------------------------

    /// Poll Telegram for new messages/commands. Returns parsed commands
    /// that originated from the configured chat_id only.
    pub async fn poll_commands(&mut self) -> Vec<BotCommand> {
        if !self.is_configured() {
            return Vec::new();
        }

        let url = format!(
            "{}/bot{}/getUpdates?offset={}&timeout=0&limit=10",
            TELEGRAM_API,
            self.token,
            self.last_update_id + 1,
        );

        let resp = match self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                if !e.to_string().contains("aborted") {
                    eprintln!("[Telegram] Poll error: {}", e);
                }
                return Vec::new();
            }
        };

        if !resp.status().is_success() {
            return Vec::new();
        }

        let data: Value = match resp.json().await {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        if !data["ok"].as_bool().unwrap_or(false) {
            return Vec::new();
        }

        let mut commands = Vec::new();
        if let Some(results) = data["result"].as_array() {
            for update in results {
                if let Some(update_id) = update["update_id"].as_i64() {
                    if update_id > self.last_update_id {
                        self.last_update_id = update_id;
                    }
                }
                let msg = &update["message"];
                let text = match msg["text"].as_str() {
                    Some(t) => t.trim(),
                    None => continue,
                };
                // Security: only respond to the configured chat
                let chat_id = msg["chat"]["id"]
                    .as_i64()
                    .map(|id| id.to_string())
                    .unwrap_or_default();
                if chat_id != self.chat_id {
                    continue;
                }

                if let Some(cmd) = self.parse_command(text) {
                    commands.push(cmd);
                }
            }
        }
        commands
    }

    /// Handle a command and return the response text.
    pub fn handle_command(&mut self, cmd: &str, args: &str) -> String {
        match cmd {
            "help" => {
                let lines: Vec<String> = BOT_COMMANDS
                    .iter()
                    .map(|(c, d)| format!("/{} -- {}", c, d))
                    .collect();
                format!(
                    "*CHAOS BOT COMMANDS*\n\n{}\n\n_Tip: Commands are case-insensitive_",
                    lines.join("\n")
                )
            }
            "mute" => {
                let hours: f64 = args.parse().unwrap_or(1.0);
                let dur = std::time::Duration::from_secs_f64(hours * 3600.0);
                self.mute_until = Some(Instant::now() + dur);
                format!(
                    "Alerts muted for {}h.\nUse /unmute to resume.",
                    hours
                )
            }
            "unmute" => {
                self.mute_until = None;
                "Alerts resumed. You'll receive the next signal evaluation.".to_string()
            }
            "alerts" => {
                let recent: Vec<&AlertRecord> = self.alert_history.iter().rev().take(10).collect();
                if recent.is_empty() {
                    return "No recent alerts.".to_string();
                }
                let lines: Vec<String> = recent
                    .iter()
                    .map(|a| {
                        let emoji = match a.tier.as_str() {
                            "FLASH" => "\u{1F534}",
                            "PRIORITY" => "\u{1F7E1}",
                            "ROUTINE" => "\u{1F535}",
                            _ => "\u{26AA}",
                        };
                        format!("{} {} -- {}ms ago", emoji, a.tier, now_millis() - a.timestamp)
                    })
                    .collect();
                format!("*Recent Alerts (last {})*\n\n{}", recent.len(), lines.join("\n"))
            }
            _ => String::new(), // Unknown commands handled by caller
        }
    }

    /// Load bot identity via getMe (call once at startup).
    pub async fn load_bot_identity(&mut self) -> Result<()> {
        let url = format!("{}/bot{}/getMe", TELEGRAM_API, self.token);
        let resp = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
        let data: Value = resp.json().await?;
        if let Some(username) = data["result"]["username"].as_str() {
            self.bot_username = Some(username.to_lowercase());
            eprintln!("[Telegram] Bot identity: @{}", username);
        }
        Ok(())
    }

    // -- Internal helpers -----------------------------------------------------

    fn parse_command(&self, text: &str) -> Option<BotCommand> {
        let parts: Vec<&str> = text.splitn(2, char::is_whitespace).collect();
        let raw = parts[0].to_lowercase();
        if !raw.starts_with('/') {
            return None;
        }
        // Strip @botname suffix (e.g. /status@chaos_bot)
        let command_part = if let Some(at_idx) = raw.find('@') {
            let mentioned = &raw[at_idx + 1..];
            if let Some(ref my_name) = self.bot_username {
                if mentioned != my_name.as_str() {
                    return None; // Not addressed to us
                }
            }
            &raw[1..at_idx]
        } else {
            &raw[1..]
        };

        let args = if parts.len() > 1 {
            parts[1].trim().to_string()
        } else {
            String::new()
        };

        Some(BotCommand {
            command: command_part.to_string(),
            args,
        })
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
        // Keep only last 50
        if self.alert_history.len() > 50 {
            let start = self.alert_history.len() - 50;
            self.alert_history = self.alert_history[start..].to_vec();
        }
    }
}

/// Split text into chunks of at most `max_len`, preferring newline breaks.
fn chunk_text(text: &str, max_len: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    if text.len() <= max_len {
        return vec![text.to_string()];
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    while start < bytes.len() {
        let mut end = (start + max_len).min(bytes.len());
        if end < bytes.len() {
            // Try to break at a newline
            if let Some(pos) = text[start..end].rfind('\n') {
                end = start + pos + 1;
            }
        }
        chunks.push(text[start..end].to_string());
        start = end;
    }
    chunks
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

    #[test]
    fn test_not_configured_by_default() {
        let bot = TelegramBot::new("", "");
        assert!(!bot.is_configured());
    }

    #[test]
    fn test_configured_with_token_and_chat() {
        let bot = TelegramBot::new("123:ABC", "987654");
        assert!(bot.is_configured());
    }

    #[test]
    fn test_chunk_text_short() {
        let chunks = chunk_text("hello", 4096);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "hello");
    }

    #[test]
    fn test_chunk_text_long() {
        let text = "a\n".repeat(3000);
        let chunks = chunk_text(&text, 4096);
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.len() <= 4096);
        }
    }

    #[test]
    fn test_chunk_text_empty() {
        let chunks = chunk_text("", 4096);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_parse_command_basic() {
        let bot = TelegramBot::new("tok", "123");
        let cmd = bot.parse_command("/status").unwrap();
        assert_eq!(cmd.command, "status");
        assert_eq!(cmd.args, "");
    }

    #[test]
    fn test_parse_command_with_args() {
        let bot = TelegramBot::new("tok", "123");
        let cmd = bot.parse_command("/mute 2h").unwrap();
        assert_eq!(cmd.command, "mute");
        assert_eq!(cmd.args, "2h");
    }

    #[test]
    fn test_parse_command_with_bot_mention() {
        let mut bot = TelegramBot::new("tok", "123");
        bot.bot_username = Some("chaos_bot".to_string());
        let cmd = bot.parse_command("/status@chaos_bot").unwrap();
        assert_eq!(cmd.command, "status");

        // Different bot name should be rejected
        assert!(bot.parse_command("/status@other_bot").is_none());
    }

    #[test]
    fn test_parse_non_command() {
        let bot = TelegramBot::new("tok", "123");
        assert!(bot.parse_command("hello world").is_none());
    }

    #[test]
    fn test_handle_help() {
        let mut bot = TelegramBot::new("tok", "123");
        let resp = bot.handle_command("help", "");
        assert!(resp.contains("CHAOS BOT COMMANDS"));
        assert!(resp.contains("/status"));
    }

    #[test]
    fn test_handle_mute_unmute() {
        let mut bot = TelegramBot::new("tok", "123");
        assert!(!bot.is_muted());

        let resp = bot.handle_command("mute", "2");
        assert!(resp.contains("muted"));
        assert!(bot.is_muted());

        let resp = bot.handle_command("unmute", "");
        assert!(resp.contains("resumed"));
        assert!(!bot.is_muted());
    }

    #[test]
    fn test_handle_alerts_empty() {
        let mut bot = TelegramBot::new("tok", "123");
        let resp = bot.handle_command("alerts", "");
        assert!(resp.contains("No recent alerts"));
    }
}
