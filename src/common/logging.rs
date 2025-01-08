use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn setup_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

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

#[allow(dead_code)]
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

        let tenant_id = Some("test-tenant");
        let user_id = Some("test-user");
        let request_id = "test-request-123";

        let result = with_context(tenant_id, user_id, request_id, || {
            event!(Level::INFO, "Test log message");
            42
        });

        assert_eq!(result, 42);
    }

    #[test]
    fn test_with_context_missing_ids() {
        setup_test_logging();

        let result = with_context(None, None, "test-request-123", || {
            event!(Level::INFO, "Test log message with missing IDs");
            "test result"
        });

        assert_eq!(result, "test result");
    }

    #[test]
    fn test_nested_contexts() {
        setup_test_logging();

        let outer_result = with_context(Some("tenant1"), Some("user1"), "request1", || {
            event!(Level::INFO, "Outer context");

            let inner_result = with_context(Some("tenant2"), Some("user2"), "request2", || {
                event!(Level::INFO, "Inner context");
                "inner"
            });

            (inner_result, "outer")
        });

        assert_eq!(outer_result, ("inner", "outer"));
    }
}
