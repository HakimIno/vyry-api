use actix_web::HttpResponse;
use application::AppError;
use application::auth::dtos::AuthErrorResponse;

/// Convert AppError to HTTP response
pub fn app_error_to_response(err: AppError) -> HttpResponse {
    let status_code = err.status_code();
    let error_code = err.error_code();
    let error_message = err.to_string();
    let retry_after = err.retry_after_seconds();

    let error_response = AuthErrorResponse {
        error: error_message,
        error_code: error_code.to_string(),
        retry_after_seconds: retry_after,
    };

    match status_code {
        400 => HttpResponse::BadRequest().json(error_response),
        401 => HttpResponse::Unauthorized().json(error_response),
        403 => HttpResponse::Forbidden().json(error_response),
        404 => HttpResponse::NotFound().json(error_response),
        429 => HttpResponse::TooManyRequests().json(error_response),
        _ => HttpResponse::InternalServerError().json(error_response),
    }
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
