use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::common::{
    config,
    error::{AppError, AppResult},
    i18n::{FileResourceProvider, I18nManager, SupportedLanguage},
};
use crate::infrastructure::database::connection::establish_connection;
use crate::infrastructure::services::tenant_service::TenantServiceImpl;
use crate::infrastructure::state::AppState;

mod api;
mod common;
mod domain;
mod infrastructure;

#[tokio::main]
#[allow(clippy::disallowed_methods)]
async fn main() -> AppResult<()> {
    // Initialize logging
    common::setup_logging().map_err(|e| AppError::configuration(e.to_string()))?;

    // Initialize i18n
    let i18n_manager =
        Arc::new(I18nManager::new(SupportedLanguage::En, Arc::new(FileResourceProvider)).await?);

    // Initialize database
    let db = Arc::new(establish_connection().await?);
    let tenant_service = Arc::new(TenantServiceImpl::new(Arc::clone(&db)));

    // Initialize cache
    let _cache = Arc::new(
        infrastructure::cache::CacheConnection::new()
            .await
            .map_err(|e| AppError::configuration(format!("Failed to initialize cache: {}", e)))?,
    );

    // Create app state
    let state = AppState::new(tenant_service, i18n_manager);

    // Build application
    let app = Router::new()
        .merge(api::health::health_routes())
        .merge(api::tenant::tenant_routes())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive()); // TODO: Configure CORS properly for production

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], config::get_backend_port()));
    let server_msg = format!("Server running at http://{}:{}", addr.ip(), addr.port());
    tracing::info!("{}", server_msg);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::configuration(format!("Failed to bind to address: {}", e)))?;

    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| AppError::internal(format!("Server error: {}", e)))?;

    Ok(())
}
