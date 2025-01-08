use crate::common::error::{AppError, AppResult};
use sea_orm::{Database, DatabaseConnection};

#[allow(dead_code)]
pub fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/acci_base".to_string())
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct DbConnection {
    connection: DatabaseConnection,
}

impl DbConnection {
    #[allow(dead_code)]
    #[allow(clippy::disallowed_methods)]
    pub async fn new() -> AppResult<Self> {
        match Database::connect(get_database_url()).await {
            Ok(connection) => Ok(Self { connection }),
            Err(e) => {
                tracing::error!("Failed to connect to database: {}", e);
                Err((AppError::Database(e.to_string()), Default::default()))
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() -> AppResult<()> {
        let db = DbConnection::new().await?;
        assert!(db.get_connection().ping().await.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_database_connection_with_invalid_url() {
        std::env::set_var("DATABASE_URL", "invalid_url");
        let result = DbConnection::new().await;
        assert!(result.is_err());
        std::env::remove_var("DATABASE_URL");
    }
}
