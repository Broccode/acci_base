use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::{
    api,
    common::middleware::{auth_middleware, AuthState},
};

pub fn create_router(auth_state: AuthState) -> Router {
    Router::new()
        .route("/health", get(api::health_check))
        .route("/auth/login", get(api::login))
        .route("/auth/callback", get(api::oauth_callback))
        .route("/auth/logout", get(api::logout))
        .route(
            "/api/v1/*path",
            get(api::not_found)
                .post(api::not_found)
                .with_state(auth_state.clone()),
        )
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(auth_state)
}
