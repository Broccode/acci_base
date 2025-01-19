use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug)]
pub struct AppError {
    pub kind: Box<ErrorKind>,
    pub context: ErrorContext,
}

#[derive(Debug, Error)]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum ErrorKind {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Not found error: {0}")]
    NotFoundError(String),
    #[error("I18n error: {0}")]
    I18nError(String),
    #[error("Tenant error: {0}")]
    TenantError(String),
    #[error("User error: {0}")]
    UserError(String),
    #[error("Auth error: {0}")]
    AuthError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
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

        let body = Json(ErrorResponse {
            message: self.kind.to_string(),
            context: self.context.message,
        });

        (status, body).into_response()
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self {
            kind: Box::new(ErrorKind::InternalError(error.to_string())),
            context: ErrorContext::new().with_message(error.to_string()),
        }
    }
}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        Self::database(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::internal(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal(err.to_string())
    }
}

impl From<oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>>
    for AppError
{
    fn from(
        err: oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>,
    ) -> Self {
        Self::authentication(err.to_string())
    }
}

impl From<oauth2::url::ParseError> for AppError {
    fn from(err: oauth2::url::ParseError) -> Self {
        Self::authentication(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        Self::internal(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(kind: ErrorKind, context_msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(kind),
            context: ErrorContext::new().with_message(context_msg.into()),
        }
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DatabaseError(message.into()), "Database error")
    }

    pub fn authentication(message: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::AuthenticationError(message.into()),
            "Authentication error",
        )
    }

    #[allow(dead_code)]
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::AuthorizationError(message.into()),
            "Authorization error",
        )
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::ValidationError(message.into()),
            "Validation error",
        )
    }

    pub fn configuration(message: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::ConfigurationError(message.into()),
            "Configuration error",
        )
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFoundError(message.into()), "Not found error")
    }

    pub fn i18n(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::I18nError(message.into()), "I18n error")
    }

    pub fn tenant(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TenantError(message.into()), "Tenant error")
    }

    pub fn user(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::UserError(message.into()), "User error")
    }

    #[allow(dead_code)]
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AuthError(message.into()), "Auth error")
    }

    pub fn serialization(message: impl Into<String>) -> Self {
        Self::new(
            ErrorKind::SerializationError(message.into()),
            "Serialization error",
        )
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InternalError(message.into()), "Internal error")
    }

    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.kind, self.context)
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ErrorContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn with_tenant(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_request(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}
