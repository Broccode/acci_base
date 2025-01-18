use async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection};
use tracing::info;

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
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_config.user, db_config.password, db_config.host, db_config.port, db_config.name
    );

    info!(
        "Connecting to database at {}:{}",
        db_config.host, db_config.port
    );
    let connection = Database::connect(&database_url).await?;
    info!("Database connection established");

    Ok(connection)
}
