use async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection};
use tracing::{debug, info, warn};

use crate::common::config;
use crate::common::error::AppResult;

#[async_trait]
pub trait DatabaseConnectionTrait: Send + Sync {
    #[allow(dead_code)]
    async fn connect(&self) -> AppResult<DatabaseConnection>;
    #[allow(dead_code)]
    fn clone_box(&self) -> Box<dyn DatabaseConnectionTrait>;
}

pub async fn establish_connection() -> anyhow::Result<DatabaseConnection> {
    let db_config = config::get_database_config();
    let mut connect_options = db_config.to_connect_options();

    // Enable statement logging in debug mode
    if cfg!(debug_assertions) {
        debug!("SQL statement logging enabled");
        let mut options = connect_options.clone();
        options.sqlx_logging(true);
        connect_options = options;
    }

    info!(
        "Initializing database connection pool with settings: host={}, port={}, database={}, max_connections={}, min_connections={}, connect_timeout={}s, acquire_timeout={}s, idle_timeout={}s, max_lifetime={}s",
        db_config.host,
        db_config.port,
        db_config.name,
        db_config.max_connections,
        db_config.min_connections,
        db_config.connect_timeout,
        db_config.acquire_timeout,
        db_config.idle_timeout,
        db_config.max_lifetime
    );

    let connection = match Database::connect(connect_options).await {
        Ok(conn) => {
            info!("Database connection pool established successfully");
            conn
        },
        Err(e) => {
            warn!("Failed to establish database connection pool: {}", e);
            return Err(e.into());
        },
    };

    // Test the connection
    if let Err(e) = connection.ping().await {
        warn!("Database ping test failed: {}", e);
        return Err(e.into());
    }
    debug!("Database ping test successful");

    Ok(connection)
}
