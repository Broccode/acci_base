use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::common::i18n::I18nManager;
use crate::common::middleware::setup_i18n;

mod api;
mod common;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    common::setup_logging();

    // Initialize i18n
    let i18n_manager = Arc::new(I18nManager::new().await?);

    // Build application
    let app = Router::new()
        .merge(api::health::health_routes())
        .layer(setup_i18n(Arc::clone(&i18n_manager)))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive()); // TODO: Configure CORS properly for production

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Starting server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
