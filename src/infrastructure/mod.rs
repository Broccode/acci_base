// Infrastructure module for external service integrations
pub mod database;
pub mod cache;

// Re-exports
pub use database::DatabaseConnection;
pub use cache::CacheConnection; 