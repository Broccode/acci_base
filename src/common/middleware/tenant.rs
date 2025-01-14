use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{debug, error, instrument};

use crate::common::error::AppError;
use crate::common::middleware::auth::UserInfo;
use crate::infrastructure::database::DatabaseConnectionTrait;

#[derive(Clone)]
#[allow(dead_code)]
pub struct TenantState {
    pub db: Arc<dyn DatabaseConnectionTrait>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TenantInfo {
    pub id: String,
    pub domain: String,
    pub is_active: bool,
}

impl TenantState {
    #[allow(dead_code)]
    pub fn new(db: Arc<dyn DatabaseConnectionTrait>) -> Self {
        Self { db }
    }

    #[allow(dead_code)]
    async fn get_tenant(&self, tenant_id: &str) -> Result<TenantInfo, AppError> {
        // TODO: Implement actual database query
        // For now, we just return a mock tenant
        Ok(TenantInfo {
            id: tenant_id.to_string(),
            domain: format!("{}.example.com", tenant_id),
            is_active: true,
        })
    }
}

#[instrument(skip(state, req, next))]
pub async fn tenant_middleware(
    State(state): State<TenantState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let user_info = req
        .extensions()
        .get::<UserInfo>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let tenant_id = user_info.tenant_id.as_ref().ok_or(StatusCode::FORBIDDEN)?;

    match state.get_tenant(tenant_id).await {
        Ok(tenant_info) => {
            if !tenant_info.is_active {
                error!("Tenant {} is not active", tenant_id);
                return Err(StatusCode::FORBIDDEN);
            }

            debug!("Tenant {} is valid", tenant_id);
            req.extensions_mut().insert(tenant_info);
            Ok(next.run(req).await)
        },
        Err(e) => {
            error!("Failed to get tenant information: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}
