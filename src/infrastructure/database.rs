use crate::common::error::{AppError, AppResult};
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DbErr};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DatabaseConnector {
    async fn connect(&self, url: &str) -> Result<Box<dyn DatabaseConnectionTrait>, DbErr>;
}

// Trait für die Datenbankverbindung
#[async_trait]
pub trait DatabaseConnectionTrait: Send + Sync {
    #[allow(dead_code)]
    async fn ping(&self) -> Result<(), DbErr>;
    fn clone_box(&self) -> Box<dyn DatabaseConnectionTrait>;
}

// Implementierung für die echte DatabaseConnection
#[async_trait]
impl DatabaseConnectionTrait for DatabaseConnection {
    async fn ping(&self) -> Result<(), DbErr> {
        DatabaseConnection::ping(self).await
    }

    fn clone_box(&self) -> Box<dyn DatabaseConnectionTrait> {
        Box::new(self.clone())
    }
}

pub struct DefaultDatabaseConnector;

#[async_trait]
impl DatabaseConnector for DefaultDatabaseConnector {
    async fn connect(&self, url: &str) -> Result<Box<dyn DatabaseConnectionTrait>, DbErr> {
        let conn = sea_orm::Database::connect(url).await?;
        Ok(Box::new(conn))
    }
}

#[allow(dead_code)]
pub fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/acci_base".to_string())
}

pub struct DbConnection {
    connection: Box<dyn DatabaseConnectionTrait>,
}

impl Clone for DbConnection {
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone_box(),
        }
    }
}

impl DbConnection {
    #[allow(dead_code)]
    #[allow(clippy::disallowed_methods)]
    pub async fn new() -> AppResult<Self> {
        Self::new_with_connector(DefaultDatabaseConnector).await
    }

    #[allow(dead_code)]
    #[allow(clippy::disallowed_methods)]
    async fn new_with_connector<T: DatabaseConnector>(connector: T) -> AppResult<Self> {
        match connector.connect(&get_database_url()).await {
            Ok(connection) => Ok(Self { connection }),
            Err(e) => {
                tracing::error!("Failed to connect to database: {}", e);
                Err((AppError::from(e), Default::default()))
            },
        }
    }

    #[allow(dead_code)]
    pub fn get_connection(&self) -> &dyn DatabaseConnectionTrait {
        self.connection.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use std::env;

    // Mock für die Datenbankverbindung
    mock! {
        pub Connection {
            pub fn ping(&self) -> Result<(), DbErr>;
            pub fn clone_box(&self) -> Box<dyn DatabaseConnectionTrait>;
        }
    }

    #[async_trait]
    impl DatabaseConnectionTrait for MockConnection {
        async fn ping(&self) -> Result<(), DbErr> {
            self.ping()
        }

        fn clone_box(&self) -> Box<dyn DatabaseConnectionTrait> {
            self.clone_box()
        }
    }

    // Helper function to reset environment after tests
    fn cleanup_env() {
        env::remove_var("DATABASE_URL");
    }

    #[tokio::test]
    async fn test_default_database_url() {
        cleanup_env();
        assert_eq!(
            get_database_url(),
            "postgres://postgres:postgres@localhost:5432/acci_base"
        );
    }

    #[tokio::test]
    async fn test_successful_database_connection() {
        let mut mock_connector = MockDatabaseConnector::new();
        let mut mock_conn = MockConnection::new();

        mock_conn.expect_ping().returning(|| Ok(()));

        mock_conn
            .expect_clone_box()
            .returning(|| Box::new(MockConnection::new()));

        mock_connector
            .expect_connect()
            .return_once(move |_| Ok(Box::new(mock_conn)));

        let result = DbConnection::new_with_connector(mock_connector).await;
        assert!(result.is_ok());

        // Test ping
        let db = result.unwrap();
        assert!(db.get_connection().ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_failed_database_connection() {
        let mut mock_connector = MockDatabaseConnector::new();
        mock_connector.expect_connect().return_once(|_| {
            Err(DbErr::Conn(sea_orm::RuntimeErr::Internal(
                "Mock connection error".to_string(),
            )))
        });

        let result = DbConnection::new_with_connector(mock_connector).await;
        assert!(result.is_err());
        if let Err((error, _)) = result {
            match error {
                AppError::DatabaseError(e) => {
                    assert!(e.to_string().contains("Mock connection error"))
                },
                _ => panic!("Expected Database error"),
            }
        }
    }

    #[tokio::test]
    async fn test_connection_clone() {
        let mut mock_connector = MockDatabaseConnector::new();
        let mut mock_conn = MockConnection::new();

        mock_conn.expect_ping().returning(|| Ok(()));

        mock_conn.expect_clone_box().returning(|| {
            let mut cloned_mock = MockConnection::new();
            cloned_mock.expect_ping().returning(|| Ok(()));
            Box::new(cloned_mock)
        });

        mock_connector
            .expect_connect()
            .return_once(move |_| Ok(Box::new(mock_conn)));

        let db = DbConnection::new_with_connector(mock_connector)
            .await
            .unwrap();
        let cloned_db = db.clone();

        // Verify both connections work
        assert!(db.get_connection().ping().await.is_ok());
        assert!(cloned_db.get_connection().ping().await.is_ok());
    }
}
