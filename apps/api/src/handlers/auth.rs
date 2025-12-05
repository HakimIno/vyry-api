use crate::config::Config;
use actix_web::{post, web, HttpResponse, Responder};
use application::auth::{
    dtos::{RequestOtpRequest, VerifyOtpRequest},
    use_cases::{AuthConfig, RequestOtpUseCase, VerifyOtpUseCase},
};
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;

#[post("/api/v1/auth/request-otp")]
pub async fn request_otp(
    redis_conn: web::Data<MultiplexedConnection>,
    req: web::Json<RequestOtpRequest>,
) -> impl Responder {
    let mut conn = redis_conn.get_ref().clone();

    match RequestOtpUseCase::execute(&mut conn, req.into_inner()).await {
        Ok(otp) => {
            println!("OTP sent: {}", otp); // Keep logging for now
            HttpResponse::Ok().json(serde_json::json!({
                "message": "OTP sent successfully"
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[post("/api/v1/auth/verify-otp")]
pub async fn verify_otp(
    db: web::Data<DatabaseConnection>,
    redis_conn: web::Data<MultiplexedConnection>,
    config: web::Data<Config>,
    req: web::Json<VerifyOtpRequest>,
) -> impl Responder {
    let mut conn = redis_conn.get_ref().clone();

    let auth_config = AuthConfig {
        jwt_secret: config.jwt_secret.clone(),
        jwt_expiration: config.jwt_expiration,
        refresh_token_expiration: config.refresh_token_expiration,
    };

    match VerifyOtpUseCase::execute(db.get_ref(), &mut conn, &auth_config, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            // Simple error handling for now. In a real app, we'd check error type.
            if e.to_string().contains("Invalid or expired OTP") {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e.to_string()
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        }
    }
}
