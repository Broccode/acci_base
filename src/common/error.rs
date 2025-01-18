use serde::Serialize;

#[derive(Debug)]
pub struct AppError {
    pub kind: Box<ErrorKind>,
    pub context: ErrorContext,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ErrorKind {
    ValidationError(String),
    DatabaseError(String),
    I18nError(String),
    TenantError(String),
    UserError(String),
    NotFoundError(String),
    #[allow(dead_code)]
    AuthError(String),
    AuthenticationError(String),
    #[allow(dead_code)]
    AuthorizationError(String),
    ConfigurationError(String),
    SerializationError(String),
    InternalError(String),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::I18nError(msg) => write!(f, "I18n error: {}", msg),
            Self::TenantError(msg) => write!(f, "Tenant error: {}", msg),
            Self::UserError(msg) => write!(f, "User error: {}", msg),
            Self::NotFoundError(msg) => write!(f, "Not found: {}", msg),
            Self::AuthError(msg) => write!(f, "Auth error: {}", msg),
            Self::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            Self::AuthorizationError(msg) => write!(f, "Authorization error: {}", msg),
            Self::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for AppError {}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
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

#[derive(Debug, Default, Serialize)]
pub struct ErrorContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
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

    #[allow(dead_code)]
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
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

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Context {{ user_id: {:?}, tenant_id: {:?}, request_id: {:?}, message: {:?} }}",
            self.user_id, self.tenant_id, self.request_id, self.message
        )
    }
}

impl AppError {
    #[allow(dead_code)]
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind: Box::new(kind),
            context: ErrorContext::new(),
        }
    }

    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::ValidationError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    pub fn database(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::DatabaseError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    pub fn i18n(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::I18nError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    pub fn tenant(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::TenantError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    pub fn user(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::UserError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::NotFoundError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    #[allow(dead_code)]
    pub fn auth(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::AuthError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    pub fn authentication(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::AuthenticationError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    #[allow(dead_code)]
    pub fn authorization(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::AuthorizationError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    pub fn configuration(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::ConfigurationError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    pub fn serialization(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::SerializationError(msg.into())),
            context: ErrorContext::new(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            kind: Box::new(ErrorKind::InternalError(msg.into())),
            context: ErrorContext::new(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
