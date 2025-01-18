use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    body::Body,
    extract::Extension,
    http::{Request, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use redis::Client as RedisClient;
use tokio::test;
use tower::ServiceExt;

use crate::common::{
    config::{AppConfig, KeycloakConfig},
    middleware::auth::{auth_middleware, AuthState, Claims, RealmAccess, UserInfo},
};

// Mock Redis client for testing with actual storage
#[derive(Clone)]
struct MockRedisClient {
    storage: Arc<Mutex<HashMap<String, (String, std::time::Instant, u64)>>>,
}

impl MockRedisClient {
    fn open(_url: &str) -> redis::RedisResult<Self> {
        Ok(Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn get_async_connection(&self) -> redis::RedisResult<MockRedisConnection> {
        Ok(MockRedisConnection {
            storage: self.storage.clone(),
        })
    }
}

#[derive(Clone)]
struct MockRedisConnection {
    storage: Arc<Mutex<HashMap<String, (String, std::time::Instant, u64)>>>,
}

impl MockRedisConnection {
    async fn get<K: ToString>(&mut self, key: K) -> redis::RedisResult<Option<String>> {
        let storage = self.storage.lock().map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Failed to acquire lock",
                e.to_string(),
            ))
        })?;

        if let Some((value, timestamp, ttl)) = storage.get(&key.to_string()) {
            if timestamp.elapsed().as_secs() < *ttl {
                return Ok(Some(value.clone()));
            }
        }
        Ok(None)
    }

    async fn set_ex<K: ToString, V: ToString>(
        &mut self,
        key: K,
        value: V,
        ttl: u64,
    ) -> redis::RedisResult<()> {
        let mut storage = self.storage.lock().map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Failed to acquire lock",
                e.to_string(),
            ))
        })?;

        storage.insert(
            key.to_string(),
            (value.to_string(), std::time::Instant::now(), ttl),
        );
        Ok(())
    }
}

impl From<MockRedisClient> for RedisClient {
    fn from(_mock: MockRedisClient) -> Self {
        // In tests, we'll use the mock directly instead of converting
        RedisClient::open("redis://dummy").expect("Failed to create dummy Redis client")
    }
}

// Helper function to create test configuration and state
async fn create_test_state() -> (AuthState, Arc<AppConfig>) {
    let config = Arc::new(AppConfig {
        keycloak: KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test-realm".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            public_key_cache_ttl: 3600,
            verify_token: false, // Disable token verification for testing
        },
        ..Default::default()
    });

    let redis_client =
        MockRedisClient::open("redis://dummy").expect("Failed to create mock Redis client");
    let state = AuthState::new(config.clone(), Arc::new(redis_client.into()))
        .await
        .expect("Failed to create auth state");

    (state, config)
}

// Helper function to create test claims
fn create_test_claims(roles: Vec<String>) -> Claims {
    Claims {
        sub: "test-user".to_string(),
        preferred_username: "testuser".to_string(),
        email: Some("test@example.com".to_string()),
        realm_access: Some(RealmAccess { roles }),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    }
}

// Helper function to create test token
fn create_test_token(claims: &Claims) -> String {
    const TEST_KEY: &[u8] = b"acci_test_key_do_not_use_in_production_2024";

    let mut header = Header::default();
    header.kid = Some("test_key_id".to_string());

    encode(&header, claims, &EncodingKey::from_secret(TEST_KEY))
        .expect("Failed to create test token")
}

async fn test_handler(Extension(_user_info): Extension<UserInfo>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

#[test]
async fn test_valid_token() {
    let (state, _) = create_test_state().await;

    let claims = create_test_claims(vec!["user".to_string()]);
    let token = create_test_token(&claims);

    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(axum::middleware::from_fn_with_state(state, auth_middleware));

    let req = Request::builder()
        .uri("/test")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
async fn test_invalid_token() {
    let (state, _) = create_test_state().await;

    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(axum::middleware::from_fn_with_state(state, auth_middleware));

    let req = Request::builder()
        .uri("/test")
        .header("Authorization", "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
async fn test_missing_token() {
    let (state, _) = create_test_state().await;

    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(axum::middleware::from_fn_with_state(state, auth_middleware));

    let req = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
async fn test_tenant_access() {
    let (state, _) = create_test_state().await;

    // Create claims with tenant role
    let claims = create_test_claims(vec!["tenant_123".to_string(), "user".to_string()]);
    let token = create_test_token(&claims);

    let app = Router::new().route("/test", get(test_handler)).layer(
        axum::middleware::from_fn_with_state(state.clone(), auth_middleware),
    );

    // Test request with matching tenant
    let req = Request::builder()
        .uri("/test")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify tenant access directly
    let user_info = state.validate_keycloak_token(&token).await.unwrap();
    assert!(state.verify_tenant_access(&user_info, "123").await);
    assert!(!state.verify_tenant_access(&user_info, "456").await);
}

#[test]
async fn test_role_verification() {
    let (state, _) = create_test_state().await;

    // Create claims with specific roles
    let claims = create_test_claims(vec!["admin".to_string(), "user".to_string()]);
    let token = create_test_token(&claims);

    // Verify roles directly
    let user_info = state.validate_keycloak_token(&token).await.unwrap();
    assert!(state.verify_role(&user_info, "admin").await);
    assert!(state.verify_role(&user_info, "user").await);
    assert!(!state.verify_role(&user_info, "superadmin").await);
}

#[test]
async fn test_expired_token() {
    let (state, _) = create_test_state().await;

    // Create claims with expired timestamp
    let mut claims = create_test_claims(vec!["user".to_string()]);
    claims.exp = (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize;

    let token = create_test_token(&claims);

    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(axum::middleware::from_fn_with_state(state, auth_middleware));

    let req = Request::builder()
        .uri("/test")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
