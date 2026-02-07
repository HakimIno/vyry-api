use actix_web::{
    error::ResponseError,
    http::StatusCode,
    HttpResponse,
};
use application::AppError;
use application::auth::dtos::AuthErrorResponse;
use std::fmt;

/// Wrapper around AppError to implement ResponseError (which is defined in actix-web)
#[derive(Debug)]
pub struct HttpAppError(pub AppError);

impl fmt::Display for HttpAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<AppError> for HttpAppError {
    fn from(err: AppError) -> Self {
        HttpAppError(err)
    }
}

impl ResponseError for HttpAppError {
    fn status_code(&self) -> StatusCode {
        let code = self.0.status_code();
        StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_code = self.0.error_code();
        let error_message = self.0.to_string();
        let retry_after = self.0.retry_after_seconds();

        // Log 500 errors with full details (including stack trace if available via anyhow)
        if status_code == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(
                error_code = error_code,
                error_message = %error_message,
                "Internal Server Error: {:?}",
                self.0
            );
        } else if status_code == StatusCode::UNAUTHORIZED || status_code == StatusCode::FORBIDDEN {
             tracing::warn!(
                error_code = error_code,
                error_message = %error_message,
                "Auth Error"
            );
        }

        let error_response = AuthErrorResponse {
            error: error_message,
            error_code: error_code.to_string(),
            retry_after_seconds: retry_after,
        };

        HttpResponse::build(status_code)
            .json(error_response)
    }
}

/// Convert AppError to HTTP response (helper for legacy code)
pub fn app_error_to_response(err: AppError) -> HttpResponse {
    HttpAppError(err).error_response()
}

/// Helper macro to convert AppResult to HTTP response
#[macro_export]
macro_rules! handle_app_result {
    ($result:expr) => {
        match $result {
            Ok(value) => actix_web::HttpResponse::Ok().json(value),
            Err(e) => crate::handlers::error_handler::app_error_to_response(e),
        }
    };
}
