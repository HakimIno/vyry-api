use thiserror::Error;

/// Application-level errors
#[derive(Debug, Error)]
pub enum AppError {
    /// Authentication errors (401)
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Authorization errors (403)
    #[error("Authorization failed: {0}")]
    Authorization(String),

    /// Validation errors (400)
    #[error("Validation failed: {0}")]
    Validation(String),

    /// Not found errors (404)
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Rate limiting errors (429)
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Conflict errors (409) - e.g. duplicate unique key
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Database errors (500 or mapped)
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Redis errors (500)
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Cryptographic errors (500)
    #[error("Cryptographic error: {0}")]
    Cryptographic(String),

    /// Configuration errors (500)
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Internal server errors (500)
    #[error("Internal server error: {0}")]
    Internal(anyhow::Error),
}

impl AppError {
    /// Get HTTP status code for the error
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Authentication(_) => 401,
            AppError::Authorization(_) => 403,
            AppError::Validation(_) => 400,
            AppError::NotFound(_) => 404,
            AppError::Conflict(_) => 409,
            AppError::RateLimitExceeded(_) => 429,
            AppError::Database(e) => {
                // Map specific DB errors to HTTP status codes
                match e {
                    sea_orm::DbErr::RecordNotFound(_) => 404,
                    // Check for unique constraint violation in the error message or type if possible
                    // sea_orm::DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(db_err))) 
                    // if db_err.code().as_deref() == Some("23505") => 409, // 23505 is PostgreSQL unique_violation
                    // For now, we can check string representation for common unique violation messages if explicit matching is hard
                    e if e.to_string().contains("Duplicate entry") || e.to_string().contains("unique constraint") => 409,
                    _ => 500,
                }
            },
            AppError::Redis(_) | AppError::Internal(_) | AppError::Cryptographic(_) | AppError::Configuration(_) => 500,
        }
    }

    /// Get error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Authentication(_) => "AUTHENTICATION_FAILED",
            AppError::Authorization(_) => "AUTHORIZATION_FAILED",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound(_) | AppError::Database(sea_orm::DbErr::RecordNotFound(_)) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::RateLimitExceeded(_) => "RATE_LIMITED",
            AppError::Database(e) => {
                 if e.to_string().contains("unique constraint") {
                     "CONFLICT"
                 } else {
                     "DATABASE_ERROR"
                 }
            },
            AppError::Redis(_) => "REDIS_ERROR",
            AppError::Cryptographic(_) => "CRYPTOGRAPHIC_ERROR",
            AppError::Configuration(_) => "CONFIGURATION_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Get retry after seconds (for rate limiting)
    pub fn retry_after_seconds(&self) -> Option<u64> {
        match self {
            AppError::RateLimitExceeded(_) => Some(60),
            _ => None,
        }
    }
}

// Additional From implementations for conversion

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Authentication(format!("JWT error: {}", err))
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AppError::Cryptographic(format!("Password hashing error: {}", err))
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = err
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    format!(
                        "{}: {}",
                        field,
                        e.message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "validation failed".to_string())
                    )
                })
            })
            .collect();
        AppError::Validation(messages.join(", "))
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::Validation(format!("Invalid UUID: {}", err))
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::Validation(format!("Parse error: {}", err))
    }
}


impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}

/// Result type alias for application errors
pub type AppResult<T> = Result<T, AppError>;
