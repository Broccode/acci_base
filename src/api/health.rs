use crate::common::middleware::LanguageExt;
use axum::{http::Request, response::Json};
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
#[must_use]
pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
}

/// Health check handler that returns the system status in the requested language
pub async fn health_check<B>(request: Request<B>) -> Json<HealthResponse> {
    let lang = request.language();
    let i18n = request.i18n_manager();

    let status_message = i18n.format_message(&lang, "health-status", None).await;

    Json(HealthResponse {
        status: "healthy".to_string(),
        message: status_message,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

async fn readiness_check() -> Json<HealthResponse> {
    // TODO: Add checks for database, cache, and other dependencies
    Json(HealthResponse {
        status: "ready".to_string(),
        message: "System is ready".to_string(),
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
    use crate::common::middleware::setup_i18n;
    use axum::body::Body;
    use axum::http::header;
    use std::sync::Arc;
    use tokio::net::TcpListener;

    async fn spawn_app() -> String {
        let i18n_manager = Arc::new(I18nManager::new().await.unwrap());
        let app = Router::new()
            .merge(health_routes())
            .layer(setup_i18n(Arc::clone(&i18n_manager)));

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind random port");
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("Failed to start server");
        });

        format!("http://{}", addr)
    }

    #[tokio::test]
    async fn test_health_check() {
        let address = spawn_app().await;
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/health", address))
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(response.status().as_u16(), 200);

        let health_response = response.json::<HealthResponse>().await.unwrap();
        assert_eq!(health_response.status, "healthy");
        assert_eq!(health_response.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_readiness_check() {
        let address = spawn_app().await;
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/ready", address))
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(response.status().as_u16(), 200);

        let health_response = response.json::<HealthResponse>().await.unwrap();
        assert_eq!(health_response.status, "ready");
        assert_eq!(health_response.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_health_check_with_language() {
        // Initialize i18n manager
        let i18n_manager = Arc::new(I18nManager::new().await.unwrap());

        // Create request with German language preference
        let mut request = Request::builder()
            .header(header::ACCEPT_LANGUAGE, "de")
            .body(Body::empty())
            .unwrap();

        // Add i18n manager to request extensions
        request.extensions_mut().insert(Arc::clone(&i18n_manager));
        request.extensions_mut().insert("de".to_string());

        // Call health check
        let response = health_check(request).await;

        // Verify response
        assert_eq!(response.0.status, "healthy");
        assert_eq!(response.0.version, env!("CARGO_PKG_VERSION"));
        assert!(response.0.timestamp > 0);
    }
}
