use crate::common::error::{AppError, AppResult, ErrorContext};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub is_active: bool,
    pub settings: TenantSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    pub max_users: i32,
    pub storage_limit: i64,  // in bytes
    pub api_rate_limit: i32, // requests per minute
    pub features: TenantFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantFeatures {
    pub advanced_security: bool,
    pub custom_branding: bool,
    pub api_access: bool,
    pub audit_logging: bool,
}

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant: Tenant,
    pub request_id: String,
}

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
            }
            Ok(_) => panic!("Expected error"),
        }
    }
}
