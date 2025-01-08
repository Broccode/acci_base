use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Tenant error: {0}")]
    Tenant(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("I18n error: {0}")]
    I18n(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ErrorContext {
    pub error_id: Uuid,
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
        let tenant_id = "test-tenant";
        let context = ErrorContext::new().with_tenant(tenant_id);
        assert_eq!(context.tenant_id.unwrap(), tenant_id);
    }

    #[test]
    fn test_error_context_with_user() {
        let user_id = "test-user";
        let context = ErrorContext::new().with_user(user_id);
        assert_eq!(context.user_id.unwrap(), user_id);
    }

    #[test]
    fn test_error_context_with_request() {
        let request_id = "test-request";
        let context = ErrorContext::new().with_request(request_id);
        assert_eq!(context.request_id.unwrap(), request_id);
    }

    #[test]
    fn test_error_context_with_all_fields() {
        let tenant_id = "test-tenant";
        let user_id = "test-user";
        let request_id = "test-request";

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
        let db_error = AppError::Database("connection failed".into());
        assert_eq!(db_error.to_string(), "Database error: connection failed");

        let auth_error = AppError::Auth("invalid token".into());
        assert_eq!(
            auth_error.to_string(),
            "Authentication error: invalid token"
        );

        let validation_error = AppError::Validation("invalid input".into());
        assert_eq!(
            validation_error.to_string(),
            "Validation error: invalid input"
        );

        let i18n_error = AppError::I18n("missing translation".into());
        assert_eq!(i18n_error.to_string(), "I18n error: missing translation");

        let tenant_error = AppError::Tenant("invalid tenant".into());
        assert_eq!(tenant_error.to_string(), "Tenant error: invalid tenant");
    }

    #[test]
    fn test_app_result_err() {
        let error = AppError::Validation("Resource not found".into());
        let context = ErrorContext::new();
        let result: AppResult<i32> = Err((error, context));

        match result {
            Ok(_) => panic!("Expected error"),
            Err((error, context)) => {
                assert!(matches!(error, AppError::Validation(_)));
                assert!(context.error_id != Uuid::nil());
            }
        }
    }
}
