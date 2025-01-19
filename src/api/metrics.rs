use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use crate::infrastructure::state::AppState;

pub fn metrics_routes() -> Router<AppState> {
    Router::new().route("/metrics", get(metrics_handler))
}

async fn metrics_handler(State(state): State<AppState>) -> Response {
    let metrics = state.metrics_handle.render();
    match Response::builder()
        .header("Content-Type", "text/plain")
        .body(metrics)
    {
        Ok(response) => response.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error rendering metrics").into_response(),
    }
}
