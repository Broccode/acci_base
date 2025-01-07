use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Tenant error: {0}")]
    Tenant(String),
}

#[derive(Debug)]
pub struct ErrorContext {
    pub error_id: Uuid,
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            error_id: Uuid::new_v4(),
            tenant_id: None,
            user_id: None,
            request_id: None,
        }
    }

    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_request(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

pub type AppResult<T> = Result<T, (AppError, ErrorContext)>;
