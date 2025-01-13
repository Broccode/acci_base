use crate::common::error::{AppError, AppResult, ErrorContext};
use crate::common::i18n::I18nManager;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

lazy_static! {
    // Domain validation regex (basic validation, can be enhanced)
    static ref DOMAIN_REGEX: Regex = Regex::new(
        r"^([a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$"
    ).unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub is_active: bool,
    pub settings: TenantSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantSettings {
    pub max_users: i32,
    pub storage_limit: i64,  // in bytes
    pub api_rate_limit: i32, // requests per minute
    pub features: TenantFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantFeatures {
    pub advanced_security: bool,
    pub custom_branding: bool,
    pub api_access: bool,
    pub audit_logging: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TenantContext {
    pub tenant: Tenant,
    pub request_id: String,
}

impl Tenant {
    // Main validation method that checks all tenant fields
    pub fn validate(&self) -> AppResult<()> {
        self.validate_name()?;
        self.validate_domain()?;
        self.validate_settings()?;
        Ok(())
    }

    // Validate tenant name
    fn validate_name(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Validation("Tenant name cannot be empty".into()));
        }
        if self.name.len() > 100 {
            return Err(AppError::Validation("Tenant name cannot exceed 100 characters".into()));
        }
        Ok(())
    }

    // Validate domain format
    fn validate_domain(&self) -> AppResult<()> {
        if !DOMAIN_REGEX.is_match(&self.domain) {
            return Err(AppError::Validation("Invalid domain format".into()));
        }
        Ok(())
    }

    // Validate tenant settings
    fn validate_settings(&self) -> AppResult<()> {
        // Validate max users
        if self.settings.max_users < 1 {
            return Err(AppError::Validation("Max users must be at least 1".into()));
        }

        // Validate storage limit (minimum 1MB)
        if self.settings.storage_limit < 1_048_576 {
            return Err(AppError::Validation("Storage limit must be at least 1MB".into()));
        }

        // Validate API rate limit
        if self.settings.api_rate_limit < 1 {
            return Err(AppError::Validation("API rate limit must be at least 1".into()));
        }

        Ok(())
    }

    // Validate active status with i18n support
    #[allow(dead_code)]
    pub async fn validate_with_i18n(&self, i18n: &I18nManager, lang: &str) -> Result<(), AppError> {
        if !self.is_active {
            let msg = i18n.format_message(lang, "tenant-not-active", None).await;
            return Err(AppError::Tenant(msg));
        }
        Ok(())
    }

    // Simple active status validation
    pub fn validate_active(&self) -> Result<(), AppError> {
        if !self.is_active {
            return Err(AppError::Tenant("Tenant is not active".into()));
        }
        Ok(())
    }
}

#[allow(dead_code)]
impl TenantContext {
    pub fn new(tenant: Tenant, request_id: impl Into<String>) -> Self {
        Self {
            tenant,
            request_id: request_id.into(),
        }
    }

    pub fn validate_active(&self) -> AppResult<()> {
        if !self.tenant.is_active {
            return Err((
                AppError::Tenant("Tenant is not active".into()),
                ErrorContext::new()
                    .with_tenant(self.tenant.id.to_string())
                    .with_request(self.request_id.clone()),
            ));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait TenantService: Send + Sync + 'static {
    async fn find_by_id(&self, id: &str) -> Result<Tenant, AppError>;
    async fn find_by_domain(&self, domain: &str) -> Result<Tenant, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tenant(is_active: bool) -> Tenant {
        Tenant {
            id: Uuid::new_v4(),
            name: "Test Tenant".to_string(),
            domain: "test.example.com".to_string(),
            is_active,
            settings: TenantSettings {
                max_users: 100,
                storage_limit: 1024 * 1024 * 1024, // 1GB
                api_rate_limit: 1000,
                features: TenantFeatures {
                    advanced_security: true,
                    custom_branding: true,
                    api_access: true,
                    audit_logging: true,
                },
            },
        }
    }

    #[test]
    fn test_tenant_validation() {
        let tenant = create_test_tenant(true);
        assert!(tenant.validate().is_ok());
    }

    #[test]
    fn test_invalid_domain() {
        let mut tenant = create_test_tenant(true);
        tenant.domain = "invalid domain".to_string();
        assert!(tenant.validate_domain().is_err());
    }

    #[test]
    fn test_invalid_settings() {
        let mut tenant = create_test_tenant(true);
        tenant.settings.max_users = 0;
        assert!(tenant.validate_settings().is_err());

        tenant.settings.max_users = 100;
        tenant.settings.storage_limit = 0;
        assert!(tenant.validate_settings().is_err());

        tenant.settings.storage_limit = 1024 * 1024 * 1024;
        tenant.settings.api_rate_limit = 0;
        assert!(tenant.validate_settings().is_err());
    }

    #[test]
    fn test_tenant_context_new() {
        let tenant = create_test_tenant(true);
        let request_id = "test-request-123";
        let context = TenantContext::new(tenant.clone(), request_id);

        assert_eq!(context.tenant.id, tenant.id);
        assert_eq!(context.request_id, request_id);
    }

    #[test]
    fn test_validate_active_tenant() {
        let tenant = create_test_tenant(true);
        let context = TenantContext::new(tenant, "test-request-123");

        assert!(context.validate_active().is_ok());
    }

    #[test]
    fn test_validate_inactive_tenant() {
        let tenant = create_test_tenant(false);
        let context = TenantContext::new(tenant, "test-request-123");

        let result = context.validate_active();
        assert!(result.is_err());

        match result {
            Err((error, _context)) => match error {
                AppError::Tenant(msg) => assert_eq!(msg, "Tenant is not active"),
                _ => panic!("Expected AppError::Tenant"),
            },
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_error_context_for_inactive_tenant() {
        let tenant = create_test_tenant(false);
        let request_id = "test-request-123";
        let context = TenantContext::new(tenant.clone(), request_id);

        let result = context.validate_active();
        assert!(result.is_err());

        match result {
            Err((_, error_context)) => {
                assert_eq!(error_context.tenant_id.unwrap(), tenant.id.to_string());
                assert_eq!(error_context.request_id.unwrap(), request_id);
            },
            Ok(_) => panic!("Expected error"),
        }
    }
}
