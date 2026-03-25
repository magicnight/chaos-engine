use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("chaos=info"));

    // Check if JSON logging is requested (for production/log aggregation)
    let json_logging = std::env::var("CHAOS_LOG_FORMAT").unwrap_or_default() == "json";

    if json_logging {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json().flatten_event(true))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .compact())
            .init();
    }
}
