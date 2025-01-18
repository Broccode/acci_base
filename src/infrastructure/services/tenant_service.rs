use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, Set,
};
use tracing::{error, info, instrument};

use crate::{
    common::error::{AppError, AppResult, ErrorContext},
    domain::tenant::{Tenant, TenantService},
    infrastructure::database::entities::{tenant, tenant::Entity as TenantEntity},
};

#[derive(Clone)]
pub struct TenantServiceImpl {
    db: Arc<DatabaseConnection>,
}

impl TenantServiceImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn map_to_domain(&self, model: tenant::Model) -> Tenant {
        Tenant {
            id: model.id,
            name: model.name,
            domain: model.domain,
            is_active: model.is_active,
            settings: serde_json::from_value(model.settings).unwrap_or_default(),
        }
    }
}

#[async_trait]
impl TenantService for TenantServiceImpl {
    #[instrument(skip(self))]
    async fn list(&self) -> AppResult<Vec<Tenant>> {
        let models = TenantEntity::find().all(&*self.db).await.map_err(|e| {
            error!("Failed to list tenants: {}", e);
            AppError::database(e.to_string()).with_context(
                ErrorContext::new().with_message("Failed to list tenants".to_string()),
            )
        })?;

        Ok(models.into_iter().map(|m| self.map_to_domain(m)).collect())
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: &str) -> AppResult<Tenant> {
        let uuid = uuid::Uuid::parse_str(id).map_err(|e| {
            error!("Invalid UUID format: {}", e);
            AppError::validation("Invalid UUID format")
        })?;

        let model = TenantEntity::find_by_id(uuid)
            .one(&*self.db)
            .await
            .map_err(|e| {
                error!("Failed to find tenant: {}", e);
                AppError::database(e.to_string()).with_context(
                    ErrorContext::new().with_message("Failed to find tenant".to_string()),
                )
            })?
            .ok_or_else(|| AppError::not_found("Tenant not found"))?;

        Ok(self.map_to_domain(model))
    }

    #[instrument(skip(self))]
    async fn find_by_domain(&self, domain: &str) -> AppResult<Tenant> {
        let model = TenantEntity::find()
            .filter(tenant::Column::Domain.eq(domain))
            .one(&*self.db)
            .await
            .map_err(|e| {
                error!("Failed to find tenant by domain: {}", e);
                AppError::database(e.to_string()).with_context(
                    ErrorContext::new().with_message("Failed to find tenant by domain".to_string()),
                )
            })?
            .ok_or_else(|| AppError::not_found("Tenant not found"))?;

        Ok(self.map_to_domain(model))
    }

    #[instrument(skip(self, tenant))]
    async fn create(&self, tenant: Tenant) -> AppResult<Tenant> {
        let model = tenant::ActiveModel {
            id: Set(tenant.id),
            name: Set(tenant.name),
            domain: Set(tenant.domain),
            is_active: Set(tenant.is_active),
            settings: Set(serde_json::to_value(&tenant.settings)?),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        let result = model.insert(&*self.db).await.map_err(|e| {
            error!("Failed to create tenant: {}", e);
            AppError::database(e.to_string()).with_context(
                ErrorContext::new().with_message("Failed to create tenant".to_string()),
            )
        })?;

        Ok(self.map_to_domain(result))
    }

    #[instrument(skip(self, tenant))]
    async fn update(&self, tenant: Tenant) -> AppResult<Tenant> {
        let model = tenant::ActiveModel {
            id: Set(tenant.id),
            name: Set(tenant.name),
            domain: Set(tenant.domain),
            is_active: Set(tenant.is_active),
            settings: Set(serde_json::to_value(&tenant.settings)?),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        let result = model.update(&*self.db).await.map_err(|e| {
            error!("Failed to update tenant: {}", e);
            AppError::database(e.to_string()).with_context(
                ErrorContext::new().with_message("Failed to update tenant".to_string()),
            )
        })?;

        Ok(self.map_to_domain(result))
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &str) -> AppResult<()> {
        let uuid = uuid::Uuid::parse_str(id).map_err(|e| {
            error!("Invalid UUID format: {}", e);
            AppError::validation("Invalid UUID format")
        })?;

        let model = TenantEntity::find_by_id(uuid)
            .one(&*self.db)
            .await
            .map_err(|e| {
                error!("Failed to find tenant: {}", e);
                AppError::database(e.to_string()).with_context(
                    ErrorContext::new().with_message("Failed to find tenant".to_string()),
                )
            })?
            .ok_or_else(|| AppError::not_found("Tenant not found"))?;

        model.delete(&*self.db).await.map_err(|e| {
            error!("Failed to delete tenant: {}", e);
            AppError::database(e.to_string()).with_context(
                ErrorContext::new().with_message("Failed to delete tenant".to_string()),
            )
        })?;

        info!("Deleted tenant with ID: {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::error::ErrorKind,
        domain::tenant::{TenantFeatures, TenantSettings},
    };
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn create_test_tenant() -> Tenant {
        Tenant {
            id: uuid::Uuid::new_v4(),
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

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results::<tenant::Model, _, _>(vec![vec![]])
            .into_connection();

        let service = TenantServiceImpl::new(Arc::new(db));
        let result = service.find_by_id(&uuid::Uuid::new_v4().to_string()).await;

        match result {
            Err(error) => match *error.kind {
                ErrorKind::NotFoundError(_) => (),
                _ => panic!("Expected NotFoundError, got {:?}", error),
            },
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    #[tokio::test]
    async fn test_create_tenant() {
        let tenant = create_test_tenant();
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![tenant::Model {
                id: tenant.id,
                name: tenant.name.clone(),
                domain: tenant.domain.clone(),
                is_active: tenant.is_active,
                settings: serde_json::to_value(&tenant.settings).unwrap(),
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            }]])
            .into_connection();

        let service = TenantServiceImpl::new(Arc::new(db));
        let result = service.create(tenant.clone()).await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.id, tenant.id);
        assert_eq!(created.name, tenant.name);
        assert_eq!(created.domain, tenant.domain);
        assert_eq!(created.is_active, tenant.is_active);
    }

    #[tokio::test]
    async fn test_update_tenant() {
        let tenant = create_test_tenant();
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![tenant::Model {
                id: tenant.id,
                name: tenant.name.clone(),
                domain: tenant.domain.clone(),
                is_active: tenant.is_active,
                settings: serde_json::to_value(&tenant.settings).unwrap(),
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            }]])
            .into_connection();

        let service = TenantServiceImpl::new(Arc::new(db));
        let result = service.update(tenant.clone()).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.id, tenant.id);
        assert_eq!(updated.name, tenant.name);
        assert_eq!(updated.domain, tenant.domain);
        assert_eq!(updated.is_active, tenant.is_active);
    }
}
