use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use reqwest;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};

use crate::common::{config::AppConfig, error::AppError};

const JWKS_CACHE_KEY: &str = "keycloak:jwks";

#[allow(dead_code)]
#[derive(Clone)]
pub struct AuthState {
    pub config: Arc<AppConfig>,
    pub oauth_client: Arc<BasicClient>,
    pub redis_client: Arc<redis::Client>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub preferred_username: String,
    pub email: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<JwksKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksKey {
    pub kid: String,
    pub kty: String,
    pub n: String,
    pub e: String,
}

impl AuthState {
    #[allow(dead_code)]
    pub async fn new(
        config: Arc<AppConfig>,
        redis_client: Arc<redis::Client>,
    ) -> Result<Self, AppError> {
        let keycloak_config = &config.keycloak;

        let client = BasicClient::new(
            ClientId::new(keycloak_config.client_id.clone()),
            Some(ClientSecret::new(keycloak_config.client_secret.clone())),
            AuthUrl::new(format!(
                "{}/realms/{}/protocol/openid-connect/auth",
                keycloak_config.url, keycloak_config.realm
            ))
            .map_err(|e| AppError::AuthenticationError(e.to_string()))?,
            Some(
                TokenUrl::new(format!(
                    "{}/realms/{}/protocol/openid-connect/token",
                    keycloak_config.url, keycloak_config.realm
                ))
                .map_err(|e| AppError::AuthenticationError(e.to_string()))?,
            ),
        );

        Ok(Self {
            config,
            oauth_client: Arc::new(client),
            redis_client,
        })
    }

    #[allow(dead_code)]
    async fn get_jwks(&self) -> Result<Jwks, AppError> {
        // Try to get JWKS from cache
        let mut redis_conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(|e| {
                AppError::AuthenticationError(format!("Redis connection failed: {}", e))
            })?;

        let cached_jwks_result = redis::cmd("GET")
            .arg(JWKS_CACHE_KEY)
            .query_async::<_, Option<String>>(&mut redis_conn)
            .await;

        if let Ok(Some(jwks_str)) = cached_jwks_result {
            if let Ok(jwks) = serde_json::from_str::<Jwks>(&jwks_str) {
                debug!("Using cached JWKS");
                return Ok(jwks);
            }
        }

        // Fetch new JWKS from Keycloak
        debug!("Fetching new JWKS from Keycloak");
        let jwks_url = format!(
            "{}/realms/{}/protocol/openid-connect/certs",
            self.config.keycloak.url, self.config.keycloak.realm
        );

        let jwks: Jwks = reqwest::Client::new()
            .get(&jwks_url)
            .send()
            .await
            .map_err(|e| AppError::AuthenticationError(format!("Failed to fetch JWKS: {}", e)))?
            .json()
            .await
            .map_err(|e| AppError::AuthenticationError(format!("Failed to parse JWKS: {}", e)))?;

        // Cache the JWKS
        let jwks_str = serde_json::to_string(&jwks).map_err(|e| {
            AppError::AuthenticationError(format!("Failed to serialize JWKS: {}", e))
        })?;

        redis::cmd("SETEX")
            .arg(JWKS_CACHE_KEY)
            .arg(self.config.keycloak.public_key_cache_ttl)
            .arg(jwks_str)
            .query_async::<_, ()>(&mut redis_conn)
            .await
            .map_err(|e| AppError::AuthenticationError(format!("Failed to cache JWKS: {}", e)))?;

        Ok(jwks)
    }

    #[allow(dead_code)]
    fn create_decoding_key(jwks: &Jwks) -> Result<DecodingKey, AppError> {
        // For simplicity, we're using the first key. In production, you might want to match by 'kid'
        let key = jwks
            .keys
            .first()
            .ok_or_else(|| AppError::AuthenticationError("No keys found in JWKS".to_string()))?;

        // Convert RSA components to PEM format
        // Note: This is a simplified version. In production, you should properly handle different key types
        DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|e| {
            AppError::AuthenticationError(format!("Failed to create decoding key: {}", e))
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub preferred_username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub tenant_id: Option<String>,
}

#[allow(dead_code)]
async fn validate_token(state: &AuthState, token: &str) -> Result<UserInfo, AppError> {
    // Special handling for test tokens
    if cfg!(test) {
        let key = DecodingKey::from_secret(b"test_secret");
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.required_spec_claims.clear();

        let token_data = decode::<Claims>(token, &key, &validation).map_err(|e| {
            AppError::AuthenticationError(format!("Token validation failed: {}", e))
        })?;

        // Extract tenant_id from token claims
        let tenant_id = token_data.claims.realm_access.as_ref().and_then(|access| {
            access
                .roles
                .iter()
                .find(|role| role.starts_with("tenant_"))
                .map(|role| role.trim_start_matches("tenant_").to_string())
        });

        return Ok(UserInfo {
            sub: token_data.claims.sub,
            preferred_username: token_data.claims.preferred_username,
            email: token_data.claims.email,
            roles: token_data
                .claims
                .realm_access
                .map(|access| access.roles)
                .unwrap_or_default(),
            tenant_id,
        });
    }

    // Production token validation
    let jwks = state.get_jwks().await?;
    let key = AuthState::create_decoding_key(&jwks)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&state.config.keycloak.client_id]);
    validation.set_issuer(&[&format!(
        "{}/realms/{}",
        state.config.keycloak.url, state.config.keycloak.realm
    )]);

    let token_data = decode::<Claims>(token, &key, &validation)
        .map_err(|e| AppError::AuthenticationError(format!("Token validation failed: {}", e)))?;

    // Extract tenant_id from token claims
    let tenant_id = token_data.claims.realm_access.as_ref().and_then(|access| {
        access
            .roles
            .iter()
            .find(|role| role.starts_with("tenant_"))
            .map(|role| role.trim_start_matches("tenant_").to_string())
    });

    Ok(UserInfo {
        sub: token_data.claims.sub,
        preferred_username: token_data.claims.preferred_username,
        email: token_data.claims.email,
        roles: token_data
            .claims
            .realm_access
            .map(|access| access.roles)
            .unwrap_or_default(),
        tenant_id,
    })
}

#[instrument(skip(state, req, next))]
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                header
                    .strip_prefix("Bearer ")
                    .map(|token| token.to_string())
            } else {
                None
            }
        });

    match auth_header {
        Some(token) => {
            debug!("Validating token");
            match validate_token(&state, &token).await {
                Ok(user_info) => {
                    // Store user info in request extensions
                    req.extensions_mut().insert(user_info.clone());

                    // Check tenant_id
                    if user_info.tenant_id.is_none() {
                        error!("User has no tenant_id");
                        return Err(StatusCode::FORBIDDEN);
                    }

                    Ok(next.run(req).await)
                },
                Err(e) => {
                    error!("Token validation failed: {}", e);
                    Err(StatusCode::UNAUTHORIZED)
                },
            }
        },
        None => {
            debug!("No authorization header found");
            Err(StatusCode::UNAUTHORIZED)
        },
    }
}
