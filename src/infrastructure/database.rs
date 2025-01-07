use sea_orm::DatabaseConnection as SeaOrmConnection;
use crate::common::error::{AppError, AppResult};

#[derive(Clone)]
pub struct DatabaseConnection {
    connection: SeaOrmConnection,
}

impl DatabaseConnection {
    pub async fn new(database_url: &str) -> AppResult<Self> {
        let connection = sea_orm::Database::connect(database_url)
            .await
            .map_err(|e| (
                AppError::Database(e),
                Default::default(),
            ))?;

        Ok(Self { connection })
    }

    pub fn get_connection(&self) -> &SeaOrmConnection {
        &self.connection
    }
} 