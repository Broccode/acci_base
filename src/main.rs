use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::common::error::AppError;
use crate::common::i18n::{FileResourceProvider, I18nManager, SupportedLanguage};
use crate::common::metrics;
use crate::infrastructure::database::connection::establish_connection;
use crate::infrastructure::services::tenant_service::TenantServiceImpl;
use crate::infrastructure::state::AppState;

mod api;
mod common;
mod domain;
mod infrastructure;
mod router;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize logging
    common::setup_logging()?;

    // Initialize i18n
    let i18n_manager =
        Arc::new(I18nManager::new(SupportedLanguage::En, Arc::new(FileResourceProvider)).await?);

    // Initialize database
    let db = Arc::new(establish_connection().await?);

    // Initialize tenant service
    let tenant_service = Arc::new(TenantServiceImpl::new(Arc::clone(&db)));

    // Initialize metrics
    let metrics_handle = metrics::init_metrics()?;

    // Create app state
    let state = AppState::new(tenant_service, i18n_manager, metrics_handle);

    // Build application
    let app = Router::new()
        .merge(api::health::health_routes())
        .merge(api::tenant::tenant_routes())
        .merge(api::metrics::metrics_routes())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive()); // TODO: Configure CORS properly for production

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3333));
    let server_msg = format!("Server running at http://{}:{}", addr.ip(), addr.port());
    tracing::info!("{}", server_msg);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::configuration(format!("Failed to bind to address: {}", e)))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::configuration(format!("Server error: {}", e)))
}
