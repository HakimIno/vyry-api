use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

/// Phone number regex: supports international format with +
pub static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\+[1-9]\d{1,14}$").unwrap());

/// Username regex: alphanumeric, underscore, hyphen, 3-50 chars
pub static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]{3,50}$").unwrap());

/// Custom validator for phone number
pub fn validate_phone_number(phone: &str) -> Result<(), ValidationError> {
    if PHONE_REGEX.is_match(phone) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_phone_number"))
    }
}

/// Custom validator for username (handles Option)
pub fn validate_username(username: Option<&str>) -> Result<(), ValidationError> {
    match username {
        Some(u) if !u.is_empty() => {
            if USERNAME_REGEX.is_match(u) {
                Ok(())
            } else {
                Err(ValidationError::new("invalid_username"))
            }
        }
        _ => Ok(()), // None or empty is valid (optional field)
    }
}
