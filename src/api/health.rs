use crate::common::i18n::I18nManager;
use axum::{
    debug_handler,
    extract::{Extension, Query},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug, Deserialize)]
pub struct LanguageQuery {
    lang: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
    message: String,
    version: String,
    timestamp: u64,
}

pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
}

#[debug_handler]
async fn health_check(
    Query(query): Query<LanguageQuery>,
    Extension(i18n): Extension<Arc<I18nManager>>,
) -> Json<HealthResponse> {
    let lang = query.lang.unwrap_or_else(|| "en".to_string());
    let status_message = i18n.format_message(&lang, "health-status", None).await;
    let status = i18n
        .format_message(&lang, "system-status-healthy", None)
        .await;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Json(HealthResponse {
        status,
        message: status_message,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
    })
}

#[debug_handler]
async fn readiness_check(
    Query(query): Query<LanguageQuery>,
    Extension(i18n): Extension<Arc<I18nManager>>,
) -> Json<HealthResponse> {
    let lang = query.lang.unwrap_or_else(|| "en".to_string());
    let status = i18n
        .format_message(&lang, "system-status-ready", None)
        .await;
    let message = i18n
        .format_message(&lang, "system-ready-message", None)
        .await;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Json(HealthResponse {
        status,
        message,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn setup_test_app() -> Router {
        let i18n = Arc::new(I18nManager::new().await.expect("Failed to initialize i18n"));
        Router::new()
            .route("/health", get(health_check))
            .route("/ready", get(readiness_check))
            .layer(Extension(i18n))
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = setup_test_app().await;

        let request = Request::builder()
            .uri("/health?lang=en")
            .header(header::ACCEPT_LANGUAGE, "en")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(request)
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health_response: HealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health_response.status, "Healthy");
        assert_eq!(health_response.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_readiness_check() {
        let app = setup_test_app().await;

        let request = Request::builder()
            .uri("/ready?lang=en")
            .header(header::ACCEPT_LANGUAGE, "en")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(request)
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health_response: HealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health_response.status, "Ready");
        assert_eq!(health_response.version, env!("CARGO_PKG_VERSION"));
    }
}
