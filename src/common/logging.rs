use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub fn setup_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .init();

    tracing::info!("Logging initialized");
}

pub fn with_context<F, R>(
    tenant_id: Option<&str>,
    user_id: Option<&str>,
    request_id: &str,
    f: F,
) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::info_span!(
        "request",
        tenant_id = tenant_id.unwrap_or("unknown"),
        user_id = user_id.unwrap_or("unknown"),
        request_id = %request_id,
    );
    span.in_scope(f)
} 