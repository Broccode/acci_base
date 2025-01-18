use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use headers::{Cookie, HeaderMapExt};
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::common::error::AppError;
use crate::common::middleware::auth::AuthState;

#[allow(dead_code)]
const CSRF_COOKIE_NAME: &str = "csrf_state";
#[allow(dead_code)]
const PKCE_VERIFIER_COOKIE_NAME: &str = "pkce_verifier";

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    auth_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

#[derive(Debug, Serialize)]
pub struct TokenInfo {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
}

#[instrument(skip(state))]
pub async fn login(State(state): State<AuthState>) -> Result<impl IntoResponse, AppError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = state
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // In production, you should store these in secure, HTTP-only cookies
    debug!("CSRF Token: {}", csrf_token.secret());
    debug!("PKCE Verifier: {}", pkce_verifier.secret());

    let response = LoginResponse {
        auth_url: auth_url.to_string(),
    };

    Ok((
        StatusCode::OK,
        [
            (
                header::SET_COOKIE,
                format!(
                    "{}={}; HttpOnly; Secure; SameSite=Lax",
                    CSRF_COOKIE_NAME,
                    csrf_token.secret()
                ),
            ),
            (
                header::SET_COOKIE,
                format!(
                    "{}={}; HttpOnly; Secure; SameSite=Lax",
                    PKCE_VERIFIER_COOKIE_NAME,
                    pkce_verifier.secret()
                ),
            ),
        ],
        Json(response),
    ))
}

#[instrument(skip(state))]
pub async fn oauth_callback(
    State(state): State<AuthState>,
    query: axum::extract::Query<CallbackQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Response, AppError> {
    let cookies = headers
        .typed_get::<Cookie>()
        .ok_or_else(|| AppError::authentication("No cookies found".to_string()))?;

    let stored_csrf_token = cookies
        .get(CSRF_COOKIE_NAME)
        .ok_or_else(|| AppError::authentication("Missing CSRF token".to_string()))?;

    if stored_csrf_token != query.state {
        return Err(AppError::authentication("Invalid CSRF token".to_string()));
    }

    let pkce_verifier = cookies
        .get(PKCE_VERIFIER_COOKIE_NAME)
        .ok_or_else(|| AppError::authentication("Missing PKCE verifier".to_string()))?;

    let token_result = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| AppError::authentication(format!("Token exchange failed: {}", e)))?;

    let token_info = TokenInfo {
        access_token: token_result.access_token().secret().clone(),
        refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
        expires_in: token_result.expires_in().unwrap_or_default().as_secs(),
        token_type: token_result.token_type().as_ref().to_string(),
    };

    // Clear the CSRF and PKCE cookies
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::SET_COOKIE,
            format!(
                "{}=; HttpOnly; Secure; SameSite=Lax; Max-Age=0",
                CSRF_COOKIE_NAME
            ),
        )
        .header(
            header::SET_COOKIE,
            format!(
                "{}=; HttpOnly; Secure; SameSite=Lax; Max-Age=0",
                PKCE_VERIFIER_COOKIE_NAME
            ),
        )
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            serde_json::to_string(&token_info)
                .map_err(|e| AppError::serialization(e.to_string()))?
                .into(),
        )
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(response)
}

#[instrument(skip(state))]
pub async fn logout(State(state): State<AuthState>) -> impl IntoResponse {
    let logout_url = format!(
        "{}/realms/{}/protocol/openid-connect/logout",
        state.config.keycloak.url, state.config.keycloak.realm
    );

    Redirect::to(&logout_url)
}
