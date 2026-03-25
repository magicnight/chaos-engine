use anyhow::Result;
use serde_json::json;

/// Notification dispatcher — webhook + desktop alerts.
pub struct Notifier {
    client: reqwest::Client,
    webhook_url: Option<String>,
}

impl Notifier {
    pub fn new(webhook_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            webhook_url,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.webhook_url.is_some()
    }

    /// Send message to configured webhook (Slack, Discord, Feishu compatible).
    pub async fn send_webhook(&self, message: &str) -> Result<()> {
        let url = match &self.webhook_url {
            Some(u) => u,
            None => return Ok(()),
        };

        // Body format works for Slack, Discord, and Feishu
        let body = json!({
            "content": message,
            "text": message,
        });

        let resp = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            eprintln!("[notify] Webhook failed {}: {}", status, &text[..text.len().min(200)]);
        }

        Ok(())
    }

    /// Send a desktop notification (best-effort, never fails).
    pub fn send_desktop(&self, title: &str, body: &str) {
        #[cfg(target_os = "windows")]
        {
            let script = format!(
                "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
                 $xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
                 $nodes = $xml.GetElementsByTagName('text'); \
                 $nodes.Item(0).AppendChild($xml.CreateTextNode('{}')) > $null; \
                 $nodes.Item(1).AppendChild($xml.CreateTextNode('{}')) > $null; \
                 $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); \
                 [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('CHAOS').Show($toast)",
                title.replace('\'', "''"),
                body.replace('\'', "''")
            );
            let _ = std::process::Command::new("powershell")
                .args(["-Command", &script])
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("osascript")
                .args([
                    "-e",
                    &format!(
                        "display notification \"{}\" with title \"{}\"",
                        body.replace('"', "\\\""),
                        title.replace('"', "\\\"")
                    ),
                ])
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("notify-send")
                .args([title, body])
                .spawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifier_not_configured() {
        let n = Notifier::new(None);
        assert!(!n.is_configured());
    }

    #[test]
    fn test_notifier_configured() {
        let n = Notifier::new(Some("https://hooks.example.com/test".to_string()));
        assert!(n.is_configured());
    }

    #[tokio::test]
    async fn test_send_webhook_noop_when_no_url() {
        let n = Notifier::new(None);
        // Should succeed silently when no webhook is configured
        let result = n.send_webhook("test").await;
        assert!(result.is_ok());
    }
}
