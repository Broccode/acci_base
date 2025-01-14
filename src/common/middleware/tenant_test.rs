use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use sea_orm::DbErr;
use tower::ServiceExt;

use super::{
    auth::UserInfo,
    tenant::{tenant_middleware, TenantState},
};
use crate::infrastructure::database::DatabaseConnectionTrait;

// Mock für die Datenbankverbindung
#[derive(Clone)]
struct MockDb;

#[async_trait::async_trait]
impl DatabaseConnectionTrait for MockDb {
    async fn ping(&self) -> Result<(), DbErr> {
        Ok(())
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

#[tokio::test]
async fn test_tenant_middleware_no_user_info() {
    let db = Arc::new(MockDb);
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
    let db = Arc::new(MockDb);
    let tenant_state = TenantState::new(db);
    let app = create_test_router(tenant_state);

    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    request.extensions_mut().insert(create_test_user(None));

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_tenant_middleware_valid_tenant() {
    let db = Arc::new(MockDb);
    let tenant_state = TenantState::new(db);
    let app = create_test_router(tenant_state);

    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(create_test_user(Some("tenant1")));

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"Hello, World!");
}

#[tokio::test]
async fn test_tenant_middleware_tenant_info_added() {
    let db = Arc::new(MockDb);
    let tenant_state = TenantState::new(db);
    let app = create_test_router(tenant_state);

    let mut request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(create_test_user(Some("tenant1")));

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"Hello, World!");
}
