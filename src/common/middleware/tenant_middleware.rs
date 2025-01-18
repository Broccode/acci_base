use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::{
    common::error::{AppError, AppResult, ErrorContext},
    domain::tenant::Tenant,
    infrastructure::{
        database::connection::DatabaseConnectionTrait, services::tenant_service::TenantServiceImpl,
    },
};

#[derive(Clone)]
pub struct TenantState {
    db: Arc<dyn DatabaseConnectionTrait>,
}

impl TenantState {
    pub fn new(db: Arc<dyn DatabaseConnectionTrait>) -> Self {
        Self { db }
    }
}

#[derive(Clone)]
pub struct TenantInfo {
    pub tenant: Tenant,
    pub request_id: String,
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for TenantInfo
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let tenant_state = State::<TenantState>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                error!("Failed to extract tenant state: {}", e);
                AppError::internal("Failed to extract tenant state")
                    .with_context(ErrorContext::new())
                    .into_response()
            })?;

        let tenant_id = parts
            .headers
            .get("X-Tenant-ID")
            .ok_or_else(|| {
                AppError::validation("Missing X-Tenant-ID header")
                    .with_context(ErrorContext::new())
                    .into_response()
            })?
            .to_str()
            .map_err(|e| {
                error!("Invalid tenant ID format: {}", e);
                AppError::validation("Invalid tenant ID format")
                    .with_context(ErrorContext::new())
                    .into_response()
            })?;

        let tenant_id = Uuid::parse_str(tenant_id).map_err(|e| {
            error!("Invalid tenant ID: {}", e);
            AppError::validation("Invalid tenant ID")
                .with_context(ErrorContext::new())
                .into_response()
        })?;

        let request_id = parts
            .headers
            .get("X-Request-ID")
            .map(|h| h.to_str().unwrap_or_default().to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let db = tenant_state.db.connect().await.map_err(|e| {
            error!("Failed to connect to database: {}", e);
            AppError::database("Failed to connect to database")
                .with_context(
                    ErrorContext::new()
                        .with_request(request_id.clone())
                        .with_message(e.to_string()),
                )
                .into_response()
        })?;

        let tenant_service = TenantServiceImpl::new(Arc::new(db));
        let tenant = tenant_service
            .find_by_id(&tenant_id.to_string())
            .await
            .map_err(|e| {
                error!("Failed to find tenant: {}", e);
                AppError::tenant("Tenant not found")
                    .with_context(
                        ErrorContext::new()
                            .with_request(request_id.clone())
                            .with_tenant(tenant_id.to_string()),
                    )
                    .into_response()
            })?;

        if !tenant.is_active {
            return Err(AppError::tenant("Tenant is not active")
                .with_context(
                    ErrorContext::new()
                        .with_request(request_id.clone())
                        .with_tenant(tenant_id.to_string()),
                )
                .into_response());
        }

        Ok(TenantInfo { tenant, request_id })
    }
}
