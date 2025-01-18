use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::Serialize;

use crate::common::{
    error::{AppError, AppResult, ErrorKind},
    i18n::SupportedLanguage,
};
use crate::infrastructure::state::AppState;

pub fn health_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/ready", axum::routing::get(readiness_check))
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    message: String,
    timestamp: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match *self.kind {
            ErrorKind::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
            ErrorKind::AuthorizationError(_) => StatusCode::FORBIDDEN,
            ErrorKind::ValidationError(_) => StatusCode::BAD_REQUEST,
            ErrorKind::ConfigurationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::NotFoundError(_) => StatusCode::NOT_FOUND,
            ErrorKind::I18nError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::TenantError(_) => StatusCode::BAD_REQUEST,
            ErrorKind::UserError(_) => StatusCode::BAD_REQUEST,
            ErrorKind::AuthError(_) => StatusCode::UNAUTHORIZED,
            ErrorKind::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ErrorBody {
            message: self.to_string(),
            context: Some(self.context.to_string()),
        });

        (status, body).into_response()
    }
}

pub async fn health_check(State(state): State<AppState>) -> AppResult<Json<HealthResponse>> {
    let status = state
        .i18n
        .format_message(SupportedLanguage::En, "health-status", None)
        .await?;

    let message = state
        .i18n
        .format_message(SupportedLanguage::En, "system-status-healthy", None)
        .await?;

    let timestamp = Utc::now().to_rfc3339();

    Ok(Json(HealthResponse {
        status,
        message,
        timestamp,
    }))
}

pub async fn readiness_check(State(state): State<AppState>) -> AppResult<Json<HealthResponse>> {
    let status = state
        .i18n
        .format_message(SupportedLanguage::En, "system-status-ready", None)
        .await?;

    let message = state
        .i18n
        .format_message(SupportedLanguage::En, "system-ready-message", None)
        .await?;

    let timestamp = Utc::now().to_rfc3339();

    Ok(Json(HealthResponse {
        status,
        message,
        timestamp,
    }))
}
