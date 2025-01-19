use axum::Router;

use crate::{
    api::{api_routes, not_found::not_found},
    infrastructure::state::AppState,
};

#[allow(dead_code)]
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(api_routes())
        .fallback(not_found)
        .with_state(state)
}
