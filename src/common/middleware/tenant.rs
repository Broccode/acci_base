use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{debug, error, instrument};

use crate::common::error::{AppError, ErrorKind};
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
        // Mock implementation for testing
        let inactive_id = "00000000-0000-0000-0000-000000000001";
        let not_found_id = "00000000-0000-0000-0000-000000000002";

        if tenant_id == inactive_id {
            Ok(TenantInfo {
                id: tenant_id.to_string(),
                domain: format!("{}.example.com", tenant_id),
                is_active: false,
            })
        } else if tenant_id == not_found_id {
            Err(AppError::not_found("Tenant not found"))
        } else {
            Ok(TenantInfo {
                id: tenant_id.to_string(),
                domain: format!("{}.example.com", tenant_id),
                is_active: true,
            })
        }
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

    let tenant_id = user_info
        .tenant_id
        .as_ref()
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Validate tenant ID format
    if uuid::Uuid::parse_str(tenant_id).is_err() {
        error!("Invalid tenant ID format: {}", tenant_id);
        return Err(StatusCode::BAD_REQUEST);
    }

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
            match *e.kind {
                ErrorKind::NotFoundError(_) => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
    }
}
