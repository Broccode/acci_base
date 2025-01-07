pub mod error;
pub mod logging;

pub use error::{AppError, AppResult, ErrorContext};
pub use logging::{setup_logging, with_context}; 