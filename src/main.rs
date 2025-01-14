use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::common::middleware::LanguageLayer;
use crate::common::{config, i18n::I18nManager};

mod api;
mod common;
mod domain;
mod infrastructure;

#[tokio::main]
#[allow(clippy::disallowed_methods)]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    common::setup_logging().map_err(|e| anyhow::anyhow!("{}", e))?;

    // Initialize i18n
    let i18n_manager = Arc::new(
        I18nManager::new()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e.0))?,
    );

    // Initialize cache
    let _cache = Arc::new(
        infrastructure::cache::CacheConnection::new()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e.0))?,
    );

    // Build application
    let app = Router::new()
        .merge(api::health::health_routes())
        .layer(LanguageLayer::new(Arc::clone(&i18n_manager)))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive()); // TODO: Configure CORS properly for production

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], config::get_backend_port()));
    let server_msg = format!("Server running at http://{}:{}", addr.ip(), addr.port());
    tracing::info!("{}", server_msg);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
