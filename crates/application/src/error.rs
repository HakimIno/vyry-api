use std::error::Error;
use std::fmt;

/// Application-level errors
#[derive(Debug)]
pub enum AppError {
    /// Authentication errors
    Authentication(String),

    /// Authorization errors
    Authorization(String),

    /// Validation errors
    Validation(String),

    /// Not found errors
    NotFound(String),

    /// Rate limiting errors
    RateLimitExceeded(String),

    /// Database errors
    Database(String),

    /// Redis errors
    Redis(String),

    /// Cryptographic errors
    Cryptographic(String),

    /// Configuration errors
    Configuration(String),

    /// Internal server errors
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Authentication(msg) => {
                f.write_str("Authentication failed: ")?;
                f.write_str(msg)
            }
            AppError::Authorization(msg) => {
                f.write_str("Authorization failed: ")?;
                f.write_str(msg)
            }
            AppError::Validation(msg) => {
                f.write_str("Validation failed: ")?;
                f.write_str(msg)
            }
            AppError::NotFound(msg) => {
                f.write_str("Resource not found: ")?;
                f.write_str(msg)
            }
            AppError::RateLimitExceeded(msg) => {
                f.write_str("Rate limit exceeded: ")?;
                f.write_str(msg)
            }
            AppError::Database(msg) => {
                f.write_str("Database error: ")?;
                f.write_str(msg)
            }
            AppError::Redis(msg) => {
                f.write_str("Redis error: ")?;
                f.write_str(msg)
            }
            AppError::Cryptographic(msg) => {
                f.write_str("Cryptographic error: ")?;
                f.write_str(msg)
            }
            AppError::Configuration(msg) => {
                f.write_str("Configuration error: ")?;
                f.write_str(msg)
            }
            AppError::Internal(msg) => {
                f.write_str("Internal server error: ")?;
                f.write_str(msg)
            }
        }
    }
}

impl Error for AppError {}

impl AppError {
    /// Get HTTP status code for the error
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Authentication(_) => 401,
            AppError::Authorization(_) => 403,
            AppError::Validation(_) => 400,
            AppError::NotFound(_) => 404,
            AppError::RateLimitExceeded(_) => 429,
            AppError::Database(_) | AppError::Redis(_) | AppError::Internal(_) => 500,
            AppError::Cryptographic(_) => 500,
            AppError::Configuration(_) => 500,
        }
    }

    /// Get error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Authentication(_) => "AUTHENTICATION_FAILED",
            AppError::Authorization(_) => "AUTHORIZATION_FAILED",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::RateLimitExceeded(_) => "RATE_LIMITED",
            AppError::Database(_) => "DATABASE_ERROR",
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

// Convert from common error types
impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::Redis(err.to_string())
    }
}

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

/// Result type alias for application errors
pub type AppResult<T> = Result<T, AppError>;
