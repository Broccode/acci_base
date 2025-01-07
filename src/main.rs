use axum::Router;
use std::net::SocketAddr;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};

mod api;
mod common;
mod domain;
mod infrastructure;

use common::setup_logging;
use infrastructure::{DatabaseConnection, CacheConnection};

#[tokio::main]
async fn main() {
    // Initialize logging
    setup_logging();

    // Initialize infrastructure
    let _db = DatabaseConnection::new("postgres://localhost/acci_base")
        .await
        .expect("Failed to connect to database");
    
    let _cache = CacheConnection::new()
        .await
        .expect("Failed to connect to cache");

    // Create the application router
    let app = Router::new()
        .merge(api::health_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive()); // TODO: Configure CORS properly for production

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Starting server on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
