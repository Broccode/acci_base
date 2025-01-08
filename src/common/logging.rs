use crate::common::i18n::I18nManager;
use axum::http::HeaderMap;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use uuid::Uuid;

#[allow(clippy::disallowed_methods)]
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .try_init()?;

    tracing::info!("Logging initialized");
    Ok(())
}

#[allow(clippy::disallowed_methods)]
#[allow(dead_code)]
pub fn request_span(
    tenant_id: Option<String>,
    user_id: Option<String>,
    request_id: Uuid,
) -> tracing::Span {
    tracing::info_span!(
        "request",
        tenant_id = tenant_id.unwrap_or_else(|| "unknown".to_string()),
        user_id = user_id.unwrap_or_else(|| "unknown".to_string()),
        request_id = request_id.to_string()
    )
}

#[allow(clippy::disallowed_methods)]
#[allow(dead_code)]
pub fn request_span_from_headers(headers: &HeaderMap) -> tracing::Span {
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .map(String::from);
    let user_id = headers
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .map(String::from);
    let request_id = headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    tracing::info_span!(
        "request",
        tenant_id = tenant_id.unwrap_or_else(|| "unknown".to_string()),
        user_id = user_id.unwrap_or_else(|| "anonymous".to_string()),
        request_id = request_id.to_string()
    )
}

#[allow(clippy::disallowed_methods)]
#[allow(dead_code)]
pub fn with_context<F, R>(
    tenant_id: Option<String>,
    user_id: Option<String>,
    request_id: String,
    f: F,
) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::info_span!(
        "request",
        tenant_id = tenant_id.unwrap_or_else(|| "unknown".to_string()),
        user_id = user_id.unwrap_or_else(|| "unknown".to_string()),
        request_id = %request_id,
    );
    span.in_scope(f)
}

#[allow(clippy::disallowed_methods)]
#[allow(dead_code)]
pub async fn setup_request_span(
    tenant_id: Option<String>,
    user_id: Option<String>,
    request_id: String,
    i18n: &I18nManager,
) -> tracing::Span {
    let unknown = i18n.format_message("en", "log-unknown-tenant", None).await;
    let unknown_user = i18n.format_message("en", "log-unknown-user", None).await;

    tracing::info_span!(
        "request",
        tenant_id = tenant_id.unwrap_or(unknown),
        user_id = user_id.unwrap_or(unknown_user),
        request_id = request_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use tracing::{event, Level};
    use tracing_subscriber::fmt::TestWriter;

    static INIT: Once = Once::new();

    fn setup_test_logging() {
        INIT.call_once(|| {
            let env_filter = EnvFilter::new("debug");
            let _test_writer = TestWriter::new();
            let formatting_layer = fmt::layer()
                .with_test_writer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(formatting_layer)
                .init();
        });
    }

    #[test]
    fn test_with_context() {
        setup_test_logging();

        let tenant_id = Some("test-tenant".to_string());
        let user_id = Some("test-user".to_string());
        let request_id = "test-request-123".to_string();

        let result = with_context(tenant_id, user_id, request_id, || {
            event!(Level::INFO, "Test log message");
            42
        });

        assert_eq!(result, 42);
    }

    #[test]
    fn test_with_context_missing_ids() {
        setup_test_logging();

        let result = with_context(None, None, "test-request-123".to_string(), || {
            event!(Level::INFO, "Test log message with missing IDs");
            "test result"
        });

        assert_eq!(result, "test result");
    }

    #[test]
    fn test_nested_contexts() {
        setup_test_logging();

        let outer_result = with_context(
            Some("tenant1".to_string()),
            Some("user1".to_string()),
            "request1".to_string(),
            || {
                event!(Level::INFO, "Outer context");

                let inner_result = with_context(
                    Some("tenant2".to_string()),
                    Some("user2".to_string()),
                    "request2".to_string(),
                    || {
                        event!(Level::INFO, "Inner context");
                        "inner"
                    },
                );

                (inner_result, "outer")
            },
        );

        assert_eq!(outer_result, ("inner", "outer"));
    }
}
