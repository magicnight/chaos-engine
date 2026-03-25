use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChaosError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[allow(dead_code)]
    #[error("Source '{name}' error: {message}")]
    Source { name: String, message: String },

    #[allow(dead_code)]
    #[error("Source '{name}' timed out after {timeout_secs}s")]
    Timeout { name: String, timeout_secs: u64 },

    #[error("Config error: {0}")]
    Config(String),

    #[error("Error: {0}")]
    Other(String),
}
