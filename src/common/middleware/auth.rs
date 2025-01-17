//! Authentication middleware for Keycloak integration
//!
//! This module provides middleware for authenticating requests using Keycloak as the identity provider.
//! It supports JWT validation, role-based access control, and multi-tenancy.
//!
//! # Features
//!
//! - JWT validation with JWKS key rotation
//! - Role-based access control (RBAC)
//! - Multi-tenant support
//! - Redis-based JWKS caching
//! - Comprehensive metrics and monitoring

use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use metrics::{counter, histogram};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use redis::AsyncCommands;
use reqwest;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};

use crate::common::{config::AppConfig, error::AppError};

#[allow(dead_code)]
const JWKS_CACHE_KEY: &str = "keycloak:jwks";

/// State for the authentication middleware
#[derive(Clone)]
#[allow(dead_code)]
pub struct AuthState {
    pub config: Arc<AppConfig>,
    pub oauth_client: Arc<BasicClient>,
    pub redis_client: Arc<redis::Client>,
}

/// Claims extracted from the JWT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject identifier
    pub sub: String,
    /// Username
    pub preferred_username: String,
    /// Email address (optional)
    pub email: Option<String>,
    /// Realm access containing roles
    pub realm_access: Option<RealmAccess>,
    /// Token expiration timestamp
    pub exp: usize,
}

/// Realm access containing user roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    /// List of roles assigned to the user
    pub roles: Vec<String>,
}

/// JWKS (JSON Web Key Set) structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    /// List of JSON Web Keys
    pub keys: Vec<JwksKey>,
}

/// Individual JSON Web Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksKey {
    /// Key ID
    pub kid: String,
    /// Key type
    pub kty: String,
    /// Modulus for RSA keys
    pub n: String,
    /// Exponent for RSA keys
    pub e: String,
}

/// User information extracted from the validated token
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserInfo {
    /// Subject identifier
    pub sub: String,
    /// Username
    pub preferred_username: String,
    /// Email address (optional)
    pub email: Option<String>,
    /// List of roles
    pub roles: Vec<String>,
    /// Tenant identifier (optional)
    pub tenant_id: Option<String>,
}

#[allow(dead_code)]
impl AuthState {
    /// Creates a new instance of AuthState
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration
    /// * `redis_client` - Redis client for JWKS caching
    ///
    /// # Returns
    ///
    /// Returns a Result containing the AuthState or an AppError
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

    /// Retrieves the JWKS from cache or Keycloak
    ///
    /// First attempts to get the JWKS from Redis cache. If not found or invalid,
    /// fetches from Keycloak and caches the result.
    async fn get_jwks(&self) -> Result<Jwks, AppError> {
        // Try to get JWKS from cache
        let mut redis_conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                AppError::AuthenticationError(format!("Redis connection failed: {}", e))
            })?;

        // Use AsyncCommands trait for Redis operations
        let cached_jwks: Option<String> = redis_conn
            .get(JWKS_CACHE_KEY)
            .await
            .map_err(|e| AppError::AuthenticationError(format!("Redis get failed: {}", e)))?;

        if let Some(jwks_str) = cached_jwks {
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

        let _: () = redis_conn
            .set_ex(
                JWKS_CACHE_KEY,
                jwks_str,
                self.config.keycloak.public_key_cache_ttl,
            )
            .await
            .map_err(|e| AppError::AuthenticationError(format!("Failed to cache JWKS: {}", e)))?;

        Ok(jwks)
    }

    /// Creates a JWT decoding key from JWKS
    fn create_decoding_key(jwks: &Jwks, token: &str) -> Result<DecodingKey, AppError> {
        // Extract kid from token header if available
        let header = jsonwebtoken::decode_header(token).map_err(|e| {
            AppError::AuthenticationError(format!("Failed to decode token header: {}", e))
        })?;

        let key = if let Some(kid) = header.kid {
            // Find the key with matching kid
            jwks.keys.iter().find(|k| k.kid == kid).ok_or_else(|| {
                AppError::AuthenticationError(format!("No key found with kid: {}", kid))
            })?
        } else {
            // Fallback to first key if no kid in token
            jwks.keys
                .first()
                .ok_or_else(|| AppError::AuthenticationError("No keys found in JWKS".to_string()))?
        };

        // Convert RSA components to PEM format
        DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|e| {
            AppError::AuthenticationError(format!("Failed to create decoding key: {}", e))
        })
    }

    /// Validates a Keycloak token and extracts user information
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token to validate
    ///
    /// # Returns
    ///
    /// Returns a Result containing UserInfo or an AppError
    pub async fn validate_keycloak_token(&self, token: &str) -> Result<UserInfo, AppError> {
        // Test mode with simplified validation
        if !self.config.keycloak.verify_token {
            warn!("Running in test mode - token verification is disabled!");

            // Use a constant test key for consistent validation
            const TEST_KEY: &[u8] = b"acci_test_key_do_not_use_in_production_2024";
            let key = DecodingKey::from_secret(TEST_KEY);

            // Configure validation for test environment
            let mut validation = Validation::new(Algorithm::HS256);
            validation.validate_exp = true; // Check expiration even in test mode
            validation.validate_nbf = false;
            validation.validate_aud = false;
            validation.required_spec_claims.clear();
            validation.set_issuer(&[""]); // Empty issuer for test mode
            validation.leeway = 0; // No leeway for expiration in tests

            // Validate the token structure
            let token_data = decode::<Claims>(token, &key, &validation).map_err(|e| {
                AppError::AuthenticationError(format!("Test token validation failed: {}", e))
            })?;

            debug!("Test mode: Successfully validated token structure");

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

        let jwks = self.get_jwks().await?;
        let key = Self::create_decoding_key(&jwks, token)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.config.keycloak.client_id]);
        validation.set_issuer(&[&format!(
            "{}/realms/{}",
            self.config.keycloak.url, self.config.keycloak.realm
        )]);

        let token_data = decode::<Claims>(token, &key, &validation).map_err(|e| {
            AppError::AuthenticationError(format!("Token validation failed: {}", e))
        })?;

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

    /// Verifies if a user has a specific role
    ///
    /// # Arguments
    ///
    /// * `user_info` - User information containing roles
    /// * `required_role` - The role to check for
    ///
    /// # Returns
    ///
    /// Returns true if the user has the required role
    pub async fn verify_role(&self, user_info: &UserInfo, required_role: &str) -> bool {
        user_info.roles.contains(&required_role.to_string())
    }

    /// Verifies if a user has access to a specific tenant
    ///
    /// # Arguments
    ///
    /// * `user_info` - User information containing tenant ID
    /// * `tenant_id` - The tenant ID to check access for
    ///
    /// # Returns
    ///
    /// Returns true if the user has access to the tenant
    pub async fn verify_tenant_access(&self, user_info: &UserInfo, tenant_id: &str) -> bool {
        user_info
            .tenant_id
            .as_ref()
            .map(|id| id == tenant_id)
            .unwrap_or(false)
    }

    /// Records authentication metrics
    ///
    /// # Arguments
    ///
    /// * `success` - Whether authentication was successful
    /// * `duration` - Duration of the authentication process
    async fn record_auth_metrics(&self, success: bool, duration: std::time::Duration) {
        let status = if success { "success" } else { "failure" };
        let counter = counter!("auth_attempts_total", "status" => status.to_string());
        counter.increment(1);
        let hist = histogram!("auth_duration_seconds");
        hist.record(duration.as_secs_f64());
        debug!(
            status = status,
            duration_ms = duration.as_millis(),
            "Auth metrics recorded"
        );
    }
}

/// Authentication middleware for Axum
///
/// This middleware:
/// 1. Extracts the Bearer token from the Authorization header
/// 2. Validates the token using Keycloak
/// 3. Extracts user information and roles
/// 4. Adds user information to request extensions
/// 5. Records metrics and logs
///
/// # Arguments
///
/// * `state` - Authentication state containing configuration
/// * `req` - The incoming request
/// * `next` - The next middleware in the chain
///
/// # Returns
///
/// Returns a Result containing the Response or a StatusCode error
#[instrument(skip(state, req, next), fields(
    request_id = %uuid::Uuid::new_v4(),
    tenant_id,
    user_id
))]
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let auth_result = process_auth(&state, &mut req, next).await;
    let duration = start_time.elapsed();

    // Record metrics
    state
        .record_auth_metrics(auth_result.is_ok(), duration)
        .await;

    // Enhanced logging
    match &auth_result {
        Ok(_) => {
            if let Some(user_info) = req.extensions().get::<UserInfo>() {
                info!(
                    tenant_id = ?user_info.tenant_id,
                    user_id = ?user_info.sub,
                    "Authentication successful"
                );
            }
        },
        Err(status) => {
            warn!(
                status = ?status,
                "Authentication failed"
            );
        },
    }

    auth_result
}

#[instrument(skip(state, req, next))]
async fn process_auth(
    state: &AuthState,
    req: &mut Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            header
                .strip_prefix("Bearer ")
                .map(|token| token.to_string())
        });

    let token = auth_header.ok_or_else(|| {
        warn!("Missing authorization header");
        StatusCode::UNAUTHORIZED
    })?;

    match state.validate_keycloak_token(&token).await {
        Ok(user_info) => {
            debug!(
                user_id = ?user_info.sub,
                tenant_id = ?user_info.tenant_id,
                "Token validated successfully"
            );
            req.extensions_mut().insert(user_info);
            // Create a new request with the same parts but empty body
            let mut new_req = Request::new(Body::empty());
            *new_req.uri_mut() = req.uri().clone();
            *new_req.method_mut() = req.method().clone();
            *new_req.headers_mut() = req.headers().clone();
            *new_req.extensions_mut() = req.extensions().clone();
            Ok(next.run(new_req).await)
        },
        Err(e) => {
            error!(error = ?e, "Token validation failed");
            Err(StatusCode::UNAUTHORIZED)
        },
    }
}
