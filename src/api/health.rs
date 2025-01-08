use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    timestamp: u64,
}

pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
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
    use tokio::net::TcpListener;

    async fn spawn_app() -> String {
        let app = health_routes();
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
}
