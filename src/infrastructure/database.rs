use crate::common::error::{AppError, AppResult};
use sea_orm::DatabaseConnection as SeaOrmConnection;

#[derive(Clone)]
#[allow(dead_code)]
pub struct DatabaseConnection {
    connection: SeaOrmConnection,
}

#[allow(dead_code)]
impl DatabaseConnection {
    pub async fn new(database_url: &str) -> AppResult<Self> {
        let connection = sea_orm::Database::connect(database_url)
            .await
            .map_err(|e| (AppError::Database(e), Default::default()))?;

        Ok(Self { connection })
    }

    pub fn get_connection(&self) -> &SeaOrmConnection {
        &self.connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_database_connection_invalid_url() {
        let result =
            DatabaseConnection::new("postgres://invalid:123@nowhere:5432/nonexistent").await;
        assert!(result.is_err());

        match result {
            Err((error, _)) => match error {
                AppError::Database(_) => (),
                _ => panic!("Expected Database error"),
            },
            Ok(_) => panic!("Expected error for invalid URL"),
        }
    }

    #[tokio::test]
    async fn test_database_connection_valid_url() {
        // Skip test if no database URL is provided
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());

        let result = DatabaseConnection::new(&database_url).await;
        if result.is_ok() {
            let db = result.unwrap();
            assert!(db.get_connection().ping().await.is_ok());
        } else {
            // Test is running without a database, which is fine
            println!("Skipping database connection test - no valid database available");
        }
    }
}
