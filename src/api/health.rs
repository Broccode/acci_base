use crate::common::i18n::I18nManager;
use axum::extract::Extension;
use axum::response::Json;
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
    message: String,
    version: String,
    timestamp: u64,
}

/// Returns a router with all health-related routes
pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
}

/// Health check handler that returns the system status in the requested language
pub async fn health_check(
    Extension(i18n): Extension<I18nManager>,
    Extension(lang): Extension<String>,
) -> Json<HealthResponse> {
    let status_message = i18n.format_message(&lang, "health-status", None).await;
    let status = i18n
        .format_message(&lang, "system-status-healthy", None)
        .await;

    Json(HealthResponse {
        status,
        message: status_message,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

pub async fn readiness_check(
    Extension(i18n): Extension<I18nManager>,
    Extension(lang): Extension<String>,
) -> Json<HealthResponse> {
    let status = i18n
        .format_message(&lang, "system-status-ready", None)
        .await;
    let message = i18n
        .format_message(&lang, "system-ready-message", None)
        .await;

    Json(HealthResponse {
        status,
        message,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::i18n::I18nManager;
    use axum::{
        body::Body,
        extract::Extension,
        http::{header, Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn setup_test_app() -> Router {
        let i18n = I18nManager::new().await.expect("Failed to initialize i18n");
        Router::new()
            .route("/health", get(health_check))
            .route("/ready", get(readiness_check))
            .layer(Extension(i18n))
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = setup_test_app().await;
        let i18n = I18nManager::new().await.expect("Failed to initialize i18n");

        let mut request = Request::builder()
            .uri("/health")
            .header(header::ACCEPT_LANGUAGE, "en")
            .body(Body::empty())
            .unwrap();

        request.extensions_mut().insert(i18n);
        request.extensions_mut().insert("en".to_string());

        let response = app
            .oneshot(request)
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health_response: HealthResponse = serde_json::from_slice(&body).unwrap();

        let expected_status = "Healthy"; // This matches our i18n key system-status-healthy
        assert_eq!(health_response.status, expected_status);
        assert_eq!(health_response.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_readiness_check() {
        let app = setup_test_app().await;
        let i18n = I18nManager::new().await.expect("Failed to initialize i18n");

        let mut request = Request::builder()
            .uri("/ready")
            .header(header::ACCEPT_LANGUAGE, "en")
            .body(Body::empty())
            .unwrap();

        request.extensions_mut().insert(i18n);
        request.extensions_mut().insert("en".to_string());

        let response = app
            .oneshot(request)
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health_response: HealthResponse = serde_json::from_slice(&body).unwrap();

        let expected_status = "Ready"; // This matches our i18n key system-status-ready
        assert_eq!(health_response.status, expected_status);
        assert_eq!(health_response.version, env!("CARGO_PKG_VERSION"));
    }
}
