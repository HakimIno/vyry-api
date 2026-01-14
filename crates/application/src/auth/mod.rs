pub mod dtos;
pub mod use_cases;
mod validation;

pub use validation::{PHONE_REGEX, USERNAME_REGEX, validate_phone_number, validate_username};