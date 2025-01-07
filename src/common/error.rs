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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_new() {
        let context = ErrorContext::new();
        assert!(context.error_id != Uuid::nil());
        assert!(context.tenant_id.is_none());
        assert!(context.user_id.is_none());
        assert!(context.request_id.is_none());
    }

    #[test]
    fn test_error_context_with_tenant() {
        let tenant_id = "test-tenant-123";
        let context = ErrorContext::new().with_tenant(tenant_id);
        assert_eq!(context.tenant_id.unwrap(), tenant_id);
    }

    #[test]
    fn test_error_context_with_user() {
        let user_id = "test-user-123";
        let context = ErrorContext::new().with_user(user_id);
        assert_eq!(context.user_id.unwrap(), user_id);
    }

    #[test]
    fn test_error_context_with_request() {
        let request_id = "test-request-123";
        let context = ErrorContext::new().with_request(request_id);
        assert_eq!(context.request_id.unwrap(), request_id);
    }

    #[test]
    fn test_error_context_chaining() {
        let tenant_id = "test-tenant-123";
        let user_id = "test-user-123";
        let request_id = "test-request-123";

        let context = ErrorContext::new()
            .with_tenant(tenant_id)
            .with_user(user_id)
            .with_request(request_id);

        assert_eq!(context.tenant_id.unwrap(), tenant_id);
        assert_eq!(context.user_id.unwrap(), user_id);
        assert_eq!(context.request_id.unwrap(), request_id);
    }

    #[test]
    fn test_app_error_display() {
        let auth_error = AppError::Authentication("Invalid credentials".into());
        assert_eq!(
            auth_error.to_string(),
            "Authentication error: Invalid credentials"
        );

        let tenant_error = AppError::Tenant("Tenant not found".into());
        assert_eq!(tenant_error.to_string(), "Tenant error: Tenant not found");
    }

    #[test]
    fn test_app_result_ok() {
        let result: AppResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_app_result_err() {
        let error = AppError::NotFound("Resource not found".into());
        let context = ErrorContext::new();
        let result: AppResult<i32> = Err((error, context));

        match result {
            Ok(_) => panic!("Expected error"),
            Err((error, context)) => {
                assert!(matches!(error, AppError::NotFound(_)));
                assert!(context.error_id != Uuid::nil());
            }
        }
    }
}
