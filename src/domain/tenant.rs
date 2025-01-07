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
