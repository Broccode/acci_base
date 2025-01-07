use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::time::SystemTime;

#[derive(Serialize)]
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
