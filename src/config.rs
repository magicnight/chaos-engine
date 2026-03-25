use std::fs;
use std::path::Path;

use crate::error::ChaosError;

/// Reads a .env file and sets environment variables for any key not already set.
/// Skips comment lines (starting with #) and empty lines.
/// Empty values are not set.
pub fn load_dotenv(path: &Path) -> Result<(), ChaosError> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(ChaosError::Config(format!(
                "Failed to read {}: {}",
                path.display(),
                e
            )))
        }
    };

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split on first '='
        let (key, value) = match line.split_once('=') {
            Some(pair) => pair,
            None => continue,
        };

        let key = key.trim();
        let value = value.trim();

        // Skip empty keys or empty values
        if key.is_empty() || value.is_empty() {
            continue;
        }

        // Do NOT override existing env vars
        if std::env::var(key).is_err() {
            std::env::set_var(key, value);
        }
    }

    Ok(())
}

/// Parsed configuration for the CHAOS engine.
#[derive(Debug, Clone)]
pub struct Config {
    pub refresh_interval_minutes: u64,
    pub source_timeout_secs: u64,
    // LLM config
    pub llm_provider: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_base_url: Option<String>,
    pub llm_model: Option<String>,
    pub ollama_url: Option<String>,
    pub ollama_model: Option<String>,
    pub sweep_lang: String,

    // Fallback LLM
    pub fallback_provider: Option<String>,
    pub fallback_model: Option<String>,
    pub gemini_api_key: Option<String>,

    // Notification
    pub webhook_url: Option<String>,

    // Telegram bot
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub telegram_poll_interval: u64,

    // Discord bot
    pub discord_bot_token: Option<String>,
    pub discord_channel_id: Option<String>,
    pub discord_guild_id: Option<String>,
    pub discord_webhook_url: Option<String>,

    // Watchlist
    pub watch_regions: Vec<String>,
    pub alert_keywords: Vec<String>,
    pub watch_tickers: Vec<String>,
}

impl Config {
    /// Load config: first attempt to read .env from cwd, then read env vars.
    pub fn load() -> Result<Self, ChaosError> {
        // Try to load .env from current working directory (best-effort)
        let dotenv_path = Path::new(".env");
        load_dotenv(dotenv_path)?;

        Ok(Config {
            refresh_interval_minutes: env_u64("REFRESH_INTERVAL_MINUTES", 15)?,
            source_timeout_secs: env_u64("SOURCE_TIMEOUT_SECS", 30)?,
            // LLM
            llm_provider: env_optional("LLM_PROVIDER"),
            llm_api_key: env_optional("LLM_API_KEY").or_else(|| env_optional("API_KEY")),
            llm_base_url: env_optional("BASE_URL"),
            llm_model: env_optional("DEFAULT_MODEL"),
            ollama_url: env_optional("OLLAMA_URL"),
            ollama_model: env_optional("OLLAMA_MODEL"),
            sweep_lang: env_string("SWEEP_LANG", "en"),

            // Fallback LLM
            fallback_provider: env_optional("FALLBACK_PROVIDER"),
            fallback_model: env_optional("FALLBACK_MODEL"),
            gemini_api_key: env_optional("GEMINI_API_KEY"),

            // Notification
            webhook_url: env_optional("WEBHOOK_URL"),

            // Telegram bot
            telegram_bot_token: env_optional("TELEGRAM_BOT_TOKEN"),
            telegram_chat_id: env_optional("TELEGRAM_CHAT_ID"),
            telegram_poll_interval: env_u64("TELEGRAM_POLL_INTERVAL", 5000)?,

            // Discord bot
            discord_bot_token: env_optional("DISCORD_BOT_TOKEN"),
            discord_channel_id: env_optional("DISCORD_CHANNEL_ID"),
            discord_guild_id: env_optional("DISCORD_GUILD_ID"),
            discord_webhook_url: env_optional("DISCORD_WEBHOOK_URL"),

            // Watchlist
            watch_regions: env_csv("WATCH_REGIONS"),
            alert_keywords: env_csv("ALERT_KEYWORDS"),
            watch_tickers: env_csv("WATCH_TICKERS"),
        })
    }
}

fn env_u64(key: &str, default: u64) -> Result<u64, ChaosError> {
    match std::env::var(key) {
        Ok(val) => val.parse::<u64>().map_err(|_| {
            ChaosError::Config(format!(
                "Invalid value for {}: '{}' is not a valid integer",
                key, val
            ))
        }),
        Err(_) => Ok(default),
    }
}

fn env_optional(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

fn env_string(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

/// Parse a comma-separated env var into a Vec<String>, trimming whitespace.
fn env_csv(key: &str) -> Vec<String> {
    std::env::var(key)
        .ok()
        .filter(|v| !v.is_empty())
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_dotenv(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn test_load_dotenv_basic() {
        let file = make_dotenv("TEST_CHAOS_BASIC=hello123\n");
        // Ensure not set before
        std::env::remove_var("TEST_CHAOS_BASIC");
        load_dotenv(file.path()).unwrap();
        assert_eq!(std::env::var("TEST_CHAOS_BASIC").unwrap(), "hello123");
        std::env::remove_var("TEST_CHAOS_BASIC");
    }

    #[test]
    fn test_load_dotenv_skips_comments() {
        let file = make_dotenv("# this is a comment\nTEST_CHAOS_COMMENT_KEY=value\n");
        std::env::remove_var("TEST_CHAOS_COMMENT_KEY");
        load_dotenv(file.path()).unwrap();
        assert_eq!(std::env::var("TEST_CHAOS_COMMENT_KEY").unwrap(), "value");
        std::env::remove_var("TEST_CHAOS_COMMENT_KEY");
    }

    #[test]
    fn test_load_dotenv_skips_empty_lines() {
        let file = make_dotenv("\n\nTEST_CHAOS_EMPTY_LINE=set\n\n");
        std::env::remove_var("TEST_CHAOS_EMPTY_LINE");
        load_dotenv(file.path()).unwrap();
        assert_eq!(std::env::var("TEST_CHAOS_EMPTY_LINE").unwrap(), "set");
        std::env::remove_var("TEST_CHAOS_EMPTY_LINE");
    }

    #[test]
    fn test_load_dotenv_skips_empty_value() {
        let file = make_dotenv("TEST_CHAOS_EMPTY_VAL=\n");
        std::env::remove_var("TEST_CHAOS_EMPTY_VAL");
        load_dotenv(file.path()).unwrap();
        assert!(std::env::var("TEST_CHAOS_EMPTY_VAL").is_err());
    }

    #[test]
    fn test_load_dotenv_no_override() {
        let file = make_dotenv("TEST_CHAOS_NO_OVERRIDE=from_file\n");
        std::env::set_var("TEST_CHAOS_NO_OVERRIDE", "from_env");
        load_dotenv(file.path()).unwrap();
        assert_eq!(
            std::env::var("TEST_CHAOS_NO_OVERRIDE").unwrap(),
            "from_env"
        );
        std::env::remove_var("TEST_CHAOS_NO_OVERRIDE");
    }

    #[test]
    fn test_load_dotenv_missing_file_ok() {
        // Should succeed silently when file does not exist
        let result = load_dotenv(Path::new("/nonexistent/.env.chaos.test"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_defaults() {
        // Remove keys to force defaults
        std::env::remove_var("REFRESH_INTERVAL_MINUTES");
        std::env::remove_var("SOURCE_TIMEOUT_SECS");
        std::env::remove_var("FRED_API_KEY");
        std::env::remove_var("FIRMS_MAP_KEY");
        std::env::remove_var("EIA_API_KEY");

        let interval = env_u64("REFRESH_INTERVAL_MINUTES", 15).unwrap();
        let timeout = env_u64("SOURCE_TIMEOUT_SECS", 30).unwrap();
        assert_eq!(interval, 15);
        assert_eq!(timeout, 30);
    }

    #[test]
    fn test_env_optional_empty_is_none() {
        std::env::set_var("TEST_CHAOS_OPT_EMPTY", "");
        assert!(env_optional("TEST_CHAOS_OPT_EMPTY").is_none());
        std::env::remove_var("TEST_CHAOS_OPT_EMPTY");
    }

    #[test]
    fn test_env_optional_set_is_some() {
        std::env::set_var("TEST_CHAOS_OPT_SET", "mykey");
        assert_eq!(env_optional("TEST_CHAOS_OPT_SET"), Some("mykey".to_string()));
        std::env::remove_var("TEST_CHAOS_OPT_SET");
    }
}
