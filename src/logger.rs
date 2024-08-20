use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Setup tracing subscriber logger
pub fn setup() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            crate::config::CONFIG.rust_log.clone(),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
