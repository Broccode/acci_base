use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::error::AppError,
    domain::tenant::{Tenant, TenantFeatures, TenantSettings},
    infrastructure::state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateTenantDto {
    pub name: String,
    pub domain: String,
    pub settings: Option<TenantSettings>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTenantDto {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub is_active: Option<bool>,
    pub settings: Option<TenantSettings>,
}

#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub is_active: bool,
    pub settings: TenantSettings,
}

impl From<Tenant> for TenantResponse {
    fn from(tenant: Tenant) -> Self {
        Self {
            id: tenant.id,
            name: tenant.name,
            domain: tenant.domain,
            is_active: tenant.is_active,
            settings: tenant.settings,
        }
    }
}

pub fn tenant_routes() -> Router<AppState> {
    Router::new()
        .route("/tenants", get(list_tenants).post(create_tenant))
        .route(
            "/tenants/:id",
            get(get_tenant).put(update_tenant).delete(delete_tenant),
        )
}

#[axum::debug_handler]
async fn list_tenants(
    State(state): State<AppState>,
) -> Result<Json<Vec<TenantResponse>>, AppError> {
    let tenants = state.tenant_service.list().await?;
    Ok(Json(tenants.into_iter().map(Into::into).collect()))
}

#[axum::debug_handler]
async fn get_tenant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TenantResponse>, AppError> {
    let tenant = state.tenant_service.find_by_id(&id.to_string()).await?;
    Ok(Json(tenant.into()))
}

#[axum::debug_handler]
async fn create_tenant(
    State(state): State<AppState>,
    Json(payload): Json<CreateTenantDto>,
) -> Result<(StatusCode, Json<TenantResponse>), AppError> {
    let settings = payload.settings.unwrap_or(TenantSettings {
        max_users: 10,
        storage_limit: 1024 * 1024 * 1024, // 1GB
        api_rate_limit: 100,
        features: TenantFeatures {
            advanced_security: false,
            custom_branding: false,
            api_access: true,
            audit_logging: false,
        },
    });

    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: payload.name,
        domain: payload.domain,
        is_active: true,
        settings,
    };

    tenant.validate()?;
    let created_tenant = state.tenant_service.create(tenant).await?;
    Ok((StatusCode::CREATED, Json(created_tenant.into())))
}

#[axum::debug_handler]
async fn update_tenant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTenantDto>,
) -> Result<Json<TenantResponse>, AppError> {
    let mut tenant = state.tenant_service.find_by_id(&id.to_string()).await?;

    if let Some(name) = payload.name {
        tenant.name = name;
    }
    if let Some(domain) = payload.domain {
        tenant.domain = domain;
    }
    if let Some(is_active) = payload.is_active {
        tenant.is_active = is_active;
    }
    if let Some(settings) = payload.settings {
        tenant.settings = settings;
    }

    tenant.validate()?;
    let updated_tenant = state.tenant_service.update(tenant).await?;
    Ok(Json(updated_tenant.into()))
}

#[axum::debug_handler]
async fn delete_tenant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.tenant_service.delete(&id.to_string()).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tenant::TenantFeatures;

    fn create_test_tenant() -> Tenant {
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

    #[test]
    fn test_tenant_response_from_tenant() {
        let tenant = create_test_tenant();
        let response: TenantResponse = tenant.clone().into();

        assert_eq!(response.id, tenant.id);
        assert_eq!(response.name, tenant.name);
        assert_eq!(response.domain, tenant.domain);
        assert_eq!(response.is_active, tenant.is_active);
        assert_eq!(response.settings.max_users, tenant.settings.max_users);
    }
}
