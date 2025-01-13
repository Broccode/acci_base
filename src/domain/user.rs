use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::error::AppError;
use crate::domain::tenant::TenantContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub username: String,
    pub full_name: String,
    pub is_active: bool,
    pub role: UserRole,
    pub settings: UserSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    TenantAdmin,
    Manager,
    User,
    ReadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSettings {
    pub language: String,
    pub timezone: String,
    pub notification_preferences: NotificationPreferences,
    pub ui_preferences: UiPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationPreferences {
    pub email_notifications: bool,
    pub in_app_notifications: bool,
    pub notification_types: Vec<NotificationType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    System,
    Security,
    Updates,
    Mentions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiPreferences {
    pub theme: String,
    pub sidebar_collapsed: bool,
    pub items_per_page: i32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserContext {
    pub user: User,
    pub tenant_context: TenantContext,
    pub request_id: String,
}

#[allow(dead_code)]
impl UserContext {
    pub fn new(user: User, tenant_context: TenantContext, request_id: impl Into<String>) -> Self {
        Self {
            user,
            tenant_context,
            request_id: request_id.into(),
        }
    }

    pub fn validate_active(&self) -> Result<(), AppError> {
        if !self.user.is_active {
            return Err(AppError::User("User is not active".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub username: String,
    pub full_name: String,
    pub role: UserRole,
    pub settings: Option<UserSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub email: Option<String>,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub role: Option<UserRole>,
    pub settings: Option<UserSettings>,
}

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait UserService: Send + Sync + 'static {
    async fn find_by_id(&self, tenant_id: &Uuid, user_id: &Uuid) -> Result<User, AppError>;
    async fn find_by_email(&self, tenant_id: &Uuid, email: &str) -> Result<User, AppError>;
    async fn create(&self, tenant_id: &Uuid, user: CreateUserDto) -> Result<User, AppError>;
    async fn update(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
        user: UpdateUserDto,
    ) -> Result<User, AppError>;
    async fn deactivate(&self, tenant_id: &Uuid, user_id: &Uuid) -> Result<(), AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user(is_active: bool) -> User {
        User {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            full_name: "Test User".to_string(),
            is_active,
            role: UserRole::User,
            settings: UserSettings::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        }
    }

    #[test]
    fn test_user_context_validation() {
        // Test active user
        let active_user = create_test_user(true);
        let context = UserContext::new(
            active_user,
            TenantContext::new(create_test_tenant(), "test-request"),
            "test-request",
        );
        assert!(context.validate_active().is_ok());

        // Test inactive user
        let inactive_user = create_test_user(false);
        let context = UserContext::new(
            inactive_user,
            TenantContext::new(create_test_tenant(), "test-request"),
            "test-request",
        );
        assert!(context.validate_active().is_err());
    }

    fn create_test_tenant() -> crate::domain::tenant::Tenant {
        use crate::domain::tenant::{Tenant, TenantFeatures, TenantSettings};

        Tenant {
            id: Uuid::new_v4(),
            name: "Test Tenant".to_string(),
            domain: "test.example.com".to_string(),
            is_active: true,
            settings: TenantSettings {
                max_users: 100,
                storage_limit: 1024 * 1024 * 1024,
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
}
