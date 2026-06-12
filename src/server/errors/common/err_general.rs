use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum UserError {
    #[error("User record not found for ID: {id}")]
    NotFound { id: String },

    #[error("Database constraint violation: {reason}")]
    DatabaseFailure { reason: String },

    #[error("The email address '{email}' is already registered")]
    EmailAlreadyExists { email: String },
}

impl UserError {
    // You can still expose a clean mechanical 'code' if your frontend needs it
    pub fn code(&self) -> &'static str {
        match self {
            UserError::General { .. } => "USER_NOT_FOUND",
            UserError::DatabaseFailure { .. } => "DATABASE_FAILURE",
            UserError::EmailAlreadyExists { .. } => "EMAIL_ALREADY_EXISTS",
        }
    }
}
