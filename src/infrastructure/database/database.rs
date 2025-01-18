use sea_orm::{Database, DatabaseConnection};
use tracing::info;

use crate::common::{
    config::DatabaseConfig,
    error::{AppError, AppResult},
};

use super::connection::DatabaseConnectionTrait;

#[derive(Clone)]
pub struct DatabaseConnection {
    config: DatabaseConfig,
}

impl DatabaseConnection {
    pub fn new(config: DatabaseConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl DatabaseConnectionTrait for DatabaseConnection {
    async fn connect(&self) -> AppResult<DatabaseConnection> {
        info!("Connecting to database at {}", self.config.url);
        Database::connect(&self.config.url)
            .await
            .map_err(|e| AppError::database(format!("Failed to connect to database: {}", e)))
    }

    fn clone_box(&self) -> Box<dyn DatabaseConnectionTrait> {
        Box::new(self.clone())
    }
}
