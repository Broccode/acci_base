use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use redis::Client as RedisClient;
use tower::ServiceExt;

use super::auth::{auth_middleware, AuthState, Claims, RealmAccess};
use crate::common::config::AppConfig;

async fn test_endpoint() -> &'static str {
    "Hello, World!"
}

fn create_test_router(auth_state: AuthState) -> Router {
    Router::new()
        .route("/test", get(test_endpoint))
        .layer(axum::middleware::from_fn_with_state(
            auth_state,
            auth_middleware,
        ))
}

fn create_test_token(
    sub: &str,
    username: &str,
    tenant_id: Option<&str>,
    roles: Vec<String>,
) -> String {
    let mut roles = roles;
    if let Some(tid) = tenant_id {
        roles.push(format!("tenant_{}", tid));
    }

    let claims = Claims {
        sub: sub.to_string(),
        preferred_username: username.to_string(),
        email: Some(format!("{}@example.com", username)),
        realm_access: Some(RealmAccess { roles }),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test_secret"),
    )
    .unwrap()
}

#[tokio::test]
async fn test_auth_middleware_no_token() {
    let config = Arc::new(AppConfig::default());
    let redis_client = Arc::new(RedisClient::open("redis://localhost").unwrap());
    let auth_state = AuthState::new(config, redis_client).await.unwrap();
    let app = create_test_router(auth_state);

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_middleware_invalid_token() {
    let config = Arc::new(AppConfig::default());
    let redis_client = Arc::new(RedisClient::open("redis://localhost").unwrap());
    let auth_state = AuthState::new(config, redis_client).await.unwrap();
    let app = create_test_router(auth_state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", "Bearer invalid_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_middleware_no_tenant() {
    let config = Arc::new(AppConfig::default());
    let redis_client = Arc::new(RedisClient::open("redis://localhost").unwrap());
    let auth_state = AuthState::new(config, redis_client).await.unwrap();
    let app = create_test_router(auth_state);

    let token = create_test_token("user123", "testuser", None, vec!["user".to_string()]);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_auth_middleware_valid_token() {
    let config = Arc::new(AppConfig::default());
    let redis_client = Arc::new(RedisClient::open("redis://localhost").unwrap());
    let auth_state = AuthState::new(config, redis_client).await.unwrap();
    let app = create_test_router(auth_state);

    let token = create_test_token(
        "user123",
        "testuser",
        Some("tenant1"),
        vec!["user".to_string()],
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"Hello, World!");
}

#[tokio::test]
async fn test_auth_middleware_expired_token() {
    let config = Arc::new(AppConfig::default());
    let redis_client = Arc::new(RedisClient::open("redis://localhost").unwrap());
    let auth_state = AuthState::new(config, redis_client).await.unwrap();
    let app = create_test_router(auth_state);

    let claims = Claims {
        sub: "user123".to_string(),
        preferred_username: "testuser".to_string(),
        email: Some("testuser@example.com".to_string()),
        realm_access: Some(RealmAccess {
            roles: vec!["user".to_string(), "tenant_tenant1".to_string()],
        }),
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test_secret"),
    )
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_middleware_multiple_roles() {
    let config = Arc::new(AppConfig::default());
    let redis_client = Arc::new(RedisClient::open("redis://localhost").unwrap());
    let auth_state = AuthState::new(config, redis_client).await.unwrap();
    let app = create_test_router(auth_state);

    let token = create_test_token(
        "user123",
        "testuser",
        Some("tenant1"),
        vec!["user".to_string(), "admin".to_string()],
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
