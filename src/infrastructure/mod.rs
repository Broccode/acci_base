// Infrastructure module for external service integrations
pub mod config;
pub mod database;
pub mod event_store;
pub mod message_broker;
pub mod redis;
pub mod services;
pub mod state;

// Re-exports
// pub use cache::CacheConnection;
// pub use database::DatabaseConnection;
