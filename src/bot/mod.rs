pub mod discord;
pub mod telegram;

use serde_json::Value;

/// Alert severity tiers mirroring the Node.js CHAOS alerter system.
#[derive(Debug, Clone)]
pub enum AlertTier {
    /// Immediate action required -- life-of-portfolio risk.
    Flash,
    /// Act within hours -- important signal cluster.
    Priority,
    /// Informational -- noteworthy but no urgency.
    Routine,
}

impl AlertTier {
    pub fn emoji(&self) -> &str {
        match self {
            Self::Flash => "\u{1F534}",
            Self::Priority => "\u{1F7E1}",
            Self::Routine => "\u{1F535}",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Flash => "FLASH",
            Self::Priority => "PRIORITY",
            Self::Routine => "ROUTINE",
        }
    }

    pub fn cooldown_secs(&self) -> u64 {
        match self {
            Self::Flash => 300,
            Self::Priority => 1800,
            Self::Routine => 3600,
        }
    }

    pub fn max_per_hour(&self) -> usize {
        match self {
            Self::Flash => 6,
            Self::Priority => 4,
            Self::Routine => 2,
        }
    }

    /// Discord embed colour.
    pub fn color(&self) -> u32 {
        match self {
            Self::Flash => 0xFF0000,
            Self::Priority => 0xFFAA00,
            Self::Routine => 0x3498DB,
        }
    }
}

/// Timestamped record kept for rate-limiting.
#[derive(Debug, Clone)]
pub struct AlertRecord {
    pub tier: String,
    pub timestamp: u64,
}

/// Parsed bot command from an incoming message.
#[derive(Debug, Clone)]
pub struct BotCommand {
    pub command: String,
    pub args: String,
}

/// Available bot commands with descriptions.
pub const BOT_COMMANDS: &[(&str, &str)] = &[
    ("status", "Get current system health, last sweep time, source status"),
    ("sweep", "Trigger a manual sweep cycle"),
    ("brief", "Get a compact text summary of the latest intelligence"),
    ("portfolio", "Show current positions and P&L (if Alpaca connected)"),
    ("alerts", "Show recent alert history"),
    ("mute", "Mute alerts for 1h (or /mute 2h, /mute 4h)"),
    ("unmute", "Resume alerts"),
    ("help", "Show available commands"),
];

/// Evaluate delta signals into an alert tier using rule-based logic.
///
/// Returns `(tier, headline, reason)` or `None` if no alert is warranted.
/// Ported from `telegram.mjs` `_ruleBasedEvaluation()`.
pub fn evaluate_alert(delta: &Value) -> Option<(AlertTier, String, String)> {
    let signals = delta.get("signals")?;
    let new_signals = signals
        .get("new")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let escalated = signals.get("escalated").and_then(|v| v.as_array());
    let summary = delta.get("summary")?;
    let critical_changes = summary
        .get("criticalChanges")
        .or_else(|| summary.get("critical_changes"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let total_changes = summary
        .get("totalChanges")
        .or_else(|| summary.get("total_changes"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let direction = summary
        .get("direction")
        .and_then(|v| v.as_str())
        .unwrap_or("mixed");

    if total_changes == 0 {
        return None;
    }

    // Count severities from escalated signals
    let (mut criticals, mut highs) = (0usize, 0usize);
    if let Some(esc) = escalated {
        for s in esc {
            match s.get("severity").and_then(|v| v.as_str()) {
                Some("critical") => criticals += 1,
                Some("high") => highs += 1,
                _ => {}
            }
        }
    }

    if critical_changes >= 5 || criticals >= 2 {
        Some((
            AlertTier::Flash,
            format!("{} Critical Signals", criticals),
            format!(
                "{} critical signals across multiple domains. Direction: {}",
                criticals, direction
            ),
        ))
    } else if criticals >= 1 || highs >= 2 {
        Some((
            AlertTier::Priority,
            format!("{} Escalating Signals", criticals + highs),
            format!(
                "{} critical, {} high severity signals. Direction: {}",
                criticals, highs, direction
            ),
        ))
    } else if total_changes >= 3 || new_signals >= 2 {
        Some((
            AlertTier::Routine,
            format!("{} Changes Detected", total_changes),
            format!(
                "{} total changes, {} new signals. Direction: {}",
                total_changes, new_signals, direction
            ),
        ))
    } else {
        None
    }
}

/// Check rate limit for a given tier against alert history.
pub fn check_rate_limit(tier: &AlertTier, history: &[AlertRecord]) -> bool {
    let label = tier.label();
    let now_ms = now_millis();
    let one_hour_ago = now_ms.saturating_sub(3_600_000);

    // Check cooldown since last alert of same tier
    let last_same = history.iter().rev().find(|a| a.tier == label);
    if let Some(last) = last_same {
        if now_ms.saturating_sub(last.timestamp) < tier.cooldown_secs() * 1000 {
            return false;
        }
    }

    // Check hourly cap
    let recent_count = history
        .iter()
        .filter(|a| a.tier == label && a.timestamp > one_hour_ago)
        .count();
    if recent_count >= tier.max_per_hour() {
        return false;
    }

    true
}

/// Current time in milliseconds since UNIX epoch.
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_alert_tier_properties() {
        let flash = AlertTier::Flash;
        assert_eq!(flash.label(), "FLASH");
        assert_eq!(flash.cooldown_secs(), 300);
        assert_eq!(flash.max_per_hour(), 6);
        assert_eq!(flash.color(), 0xFF0000);

        let priority = AlertTier::Priority;
        assert_eq!(priority.label(), "PRIORITY");

        let routine = AlertTier::Routine;
        assert_eq!(routine.label(), "ROUTINE");
    }

    #[test]
    fn test_evaluate_alert_none_when_empty() {
        let delta = json!({
            "signals": { "new": [], "escalated": [] },
            "summary": { "totalChanges": 0, "criticalChanges": 0, "direction": "mixed" }
        });
        assert!(evaluate_alert(&delta).is_none());
    }

    #[test]
    fn test_evaluate_alert_routine() {
        let delta = json!({
            "signals": { "new": [{"key": "a"}, {"key": "b"}], "escalated": [] },
            "summary": { "totalChanges": 3, "criticalChanges": 0, "direction": "risk-on" }
        });
        let result = evaluate_alert(&delta);
        assert!(result.is_some());
        let (tier, _, _) = result.unwrap();
        assert_eq!(tier.label(), "ROUTINE");
    }

    #[test]
    fn test_evaluate_alert_priority() {
        let delta = json!({
            "signals": {
                "new": [],
                "escalated": [
                    {"severity": "high", "key": "a"},
                    {"severity": "high", "key": "b"},
                ]
            },
            "summary": { "totalChanges": 5, "criticalChanges": 0, "direction": "risk-off" }
        });
        let result = evaluate_alert(&delta);
        assert!(result.is_some());
        let (tier, _, _) = result.unwrap();
        assert_eq!(tier.label(), "PRIORITY");
    }

    #[test]
    fn test_evaluate_alert_flash() {
        let delta = json!({
            "signals": {
                "new": [],
                "escalated": [
                    {"severity": "critical", "key": "a"},
                    {"severity": "critical", "key": "b"},
                ]
            },
            "summary": { "totalChanges": 5, "criticalChanges": 5, "direction": "risk-off" }
        });
        let result = evaluate_alert(&delta);
        assert!(result.is_some());
        let (tier, _, _) = result.unwrap();
        assert_eq!(tier.label(), "FLASH");
    }

    #[test]
    fn test_check_rate_limit_allows_first() {
        let tier = AlertTier::Flash;
        assert!(check_rate_limit(&tier, &[]));
    }

    #[test]
    fn test_check_rate_limit_blocks_cooldown() {
        let tier = AlertTier::Routine;
        let now = now_millis();
        let history = vec![AlertRecord {
            tier: "ROUTINE".to_string(),
            timestamp: now - 60_000, // 1 minute ago, cooldown is 1h
        }];
        assert!(!check_rate_limit(&tier, &history));
    }

    #[test]
    fn test_evaluate_alert_supports_snake_case_keys() {
        let delta = json!({
            "signals": { "new": [{"key": "a"}, {"key": "b"}, {"key": "c"}], "escalated": [] },
            "summary": { "total_changes": 4, "critical_changes": 0, "direction": "mixed" }
        });
        let result = evaluate_alert(&delta);
        assert!(result.is_some());
    }
}
