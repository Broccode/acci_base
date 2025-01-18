use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase};
use tower::ServiceExt;
use uuid::Uuid;

use super::{
    auth::UserInfo,
    tenant::{tenant_middleware, TenantState},
};
use crate::{
    common::error::AppResult,
    domain::tenant::{Tenant, TenantFeatures, TenantSettings},
    infrastructure::database::connection::DatabaseConnectionTrait,
};

#[derive(Clone)]
pub struct MockDatabaseConnection;

#[async_trait::async_trait]
impl DatabaseConnectionTrait for MockDatabaseConnection {
    async fn connect(&self) -> AppResult<DatabaseConnection> {
        Ok(MockDatabase::new(DatabaseBackend::Postgres).into_connection())
    }

    fn clone_box(&self) -> Box<dyn DatabaseConnectionTrait> {
        Box::new(self.clone())
    }
}

async fn test_endpoint() -> &'static str {
    "Hello, World!"
}

fn create_test_router(tenant_state: TenantState) -> Router {
    Router::new()
        .route("/test", get(test_endpoint))
        .layer(axum::middleware::from_fn_with_state(
            tenant_state,
            tenant_middleware,
        ))
}

fn create_test_user(tenant_id: Option<&str>) -> UserInfo {
    UserInfo {
        sub: "user123".to_string(),
        preferred_username: "testuser".to_string(),
        email: Some("testuser@example.com".to_string()),
        roles: vec!["user".to_string()],
        tenant_id: tenant_id.map(String::from),
    }
}

fn create_test_tenant(is_active: bool) -> Tenant {
    Tenant {
        id: Uuid::new_v4(),
        name: "Test Tenant".to_string(),
        domain: "test.example.com".to_string(),
        is_active,
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
async fn test_tenant_middleware_no_user_info() {
    let db = Arc::new(MockDatabaseConnection);
    let tenant_state = TenantState::new(db);
    let app = create_test_router(tenant_state);

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_tenant_middleware_no_tenant_id() {
    let db = Arc::new(MockDatabaseConnection);
    let tenant_state = TenantState::new(db);
    let app = create_test_router(tenant_state);

    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    request.extensions_mut().insert(create_test_user(None));

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_tenant_middleware_invalid_tenant_id() {
    let db = Arc::new(MockDatabaseConnection);
    let tenant_state = TenantState::new(db);
    let app = create_test_router(tenant_state);

    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(create_test_user(Some("invalid-uuid")));

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_tenant_middleware_tenant_not_found() {
    let db = Arc::new(MockDatabaseConnection);
    let tenant_state = TenantState::new(db);
    let app = create_test_router(tenant_state);

    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    request.extensions_mut().insert(create_test_user(Some(
        "00000000-0000-0000-0000-000000000002",
    )));

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_tenant_middleware_inactive_tenant() {
    let db = Arc::new(MockDatabaseConnection);
    let tenant_state = TenantState::new(db);
    let app = create_test_router(tenant_state);

    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    request.extensions_mut().insert(create_test_user(Some(
        "00000000-0000-0000-0000-000000000001",
    )));

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_tenant_middleware_valid_tenant() {
    let db = Arc::new(MockDatabaseConnection);
    let tenant_state = TenantState::new(db);
    let app = create_test_router(tenant_state);

    let tenant = create_test_tenant(true);
    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(create_test_user(Some(&tenant.id.to_string())));

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"Hello, World!");
}
