use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use regex::Regex;
use lazy_static::lazy_static;

use crate::common::error::{AppError, AppResult};
use crate::domain::tenant::TenantContext;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]{3,32}$").unwrap();
}

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

impl User {
    // Validate all user fields
    pub fn validate(&self) -> AppResult<()> {
        self.validate_email()?;
        self.validate_username()?;
        self.validate_full_name()?;
        self.validate_settings()?;
        Ok(())
    }

    // Validate email format using regex
    fn validate_email(&self) -> AppResult<()> {
        if !EMAIL_REGEX.is_match(&self.email) {
            return Err(AppError::Validation("Invalid email format".into()));
        }
        Ok(())
    }

    // Validate username format and length
    fn validate_username(&self) -> AppResult<()> {
        if !USERNAME_REGEX.is_match(&self.username) {
            return Err(AppError::Validation(
                "Username must be 3-32 characters and contain only letters, numbers, and underscores"
                    .into(),
            ));
        }
        Ok(())
    }

    // Validate full name length and content
    fn validate_full_name(&self) -> AppResult<()> {
        if self.full_name.trim().is_empty() {
            return Err(AppError::Validation("Full name cannot be empty".into()));
        }
        if self.full_name.len() > 100 {
            return Err(AppError::Validation("Full name cannot exceed 100 characters".into()));
        }
        Ok(())
    }

    // Validate user settings
    fn validate_settings(&self) -> AppResult<()> {
        // Validate items per page range
        if self.settings.ui_preferences.items_per_page < 1 || self.settings.ui_preferences.items_per_page > 100 {
            return Err(AppError::Validation("Items per page must be between 1 and 100".into()));
        }

        // Validate language code format
        if !self.settings.language.chars().all(|c| c.is_ascii_alphabetic() || c == '-') {
            return Err(AppError::Validation("Invalid language code format".into()));
        }

        // Validate timezone format (basic check)
        if self.settings.timezone.trim().is_empty() {
            return Err(AppError::Validation("Timezone cannot be empty".into()));
        }

        Ok(())
    }
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

    #[test]
    fn test_email_validation() {
        let mut user = create_test_user(true);
        assert!(user.validate_email().is_ok());

        // Test invalid email
        user.email = "invalid-email".to_string();
        assert!(user.validate_email().is_err());
    }

    #[test]
    fn test_username_validation() {
        let mut user = create_test_user(true);
        assert!(user.validate_username().is_ok());

        // Test invalid username
        user.username = "a".to_string(); // Too short
        assert!(user.validate_username().is_err());

        user.username = "user@name".to_string(); // Invalid characters
        assert!(user.validate_username().is_err());
    }

    #[test]
    fn test_full_name_validation() {
        let mut user = create_test_user(true);
        assert!(user.validate_full_name().is_ok());

        // Test empty name
        user.full_name = "".to_string();
        assert!(user.validate_full_name().is_err());

        // Test too long name
        user.full_name = "a".repeat(101);
        assert!(user.validate_full_name().is_err());
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
