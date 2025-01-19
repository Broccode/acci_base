pub mod auth;
pub mod health;
pub mod metrics;
pub mod not_found;
pub mod tenant;

use axum::Router;

use crate::infrastructure::state::AppState;

#[allow(dead_code)]
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(health::health_routes())
        .merge(tenant::tenant_routes())
        .merge(metrics::metrics_routes())
}
