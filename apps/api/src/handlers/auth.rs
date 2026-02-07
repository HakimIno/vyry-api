use crate::config::Config;
use crate::handlers::error_handler::HttpAppError;
use actix_web::{delete, get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use application::AppError;
use application::auth::{
    dtos::*,
    use_cases::*,
};
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use tracing::info;
use uuid::Uuid;

/// Extract user_id and device_id from JWT claims in request extensions
fn extract_auth_claims(req: &HttpRequest) -> Result<(Uuid, i64), AppError> {
    req.extensions()
        .get::<Claims>()
        .map(|claims| {
            let user_id = claims.sub.parse::<Uuid>().unwrap_or_default(); // Should be validated in middleware, but safe unwrap or error
            if user_id == Uuid::default() {
                 return None;
            }
            Some((user_id, claims.device_id))
        })
        .flatten()
        .ok_or_else(|| AppError::Authentication("Unauthorized".to_string()))
}

// ============ OTP Endpoints ============

#[post("/request-otp")]
pub async fn request_otp(
    redis_conn: web::Data<MultiplexedConnection>,
    req: web::Json<RequestOtpRequest>,
) -> Result<impl Responder, HttpAppError> {
    let mut conn = redis_conn.get_ref().clone();

    let otp = RequestOtpUseCase::execute(&mut conn, req.into_inner()).await?;
    info!("OTP generated for testing: {}", otp);
    
    Ok(HttpResponse::Ok().json(RequestOtpResponse {
        message: "OTP sent successfully".to_string(),
        expires_in_seconds: 180,
    }))
}

#[post("/verify-otp")]
pub async fn verify_otp(
    db: web::Data<DatabaseConnection>,
    redis_conn: web::Data<MultiplexedConnection>,
    config: web::Data<Config>,
    req: web::Json<VerifyOtpRequest>,
) -> Result<impl Responder, HttpAppError> {
    let mut conn = redis_conn.get_ref().clone();

    let auth_config = AuthConfig {
        jwt_secret: config.jwt_secret.clone(),
        jwt_expiration: config.jwt_expiration,
        refresh_token_expiration: config.refresh_token_expiration,
    };

    let response = VerifyOtpUseCase::execute(db.get_ref(), &mut conn, &auth_config, req.into_inner()).await?;
    info!("OTP verified successfully for user: {}", response.user_id);
    
    Ok(HttpResponse::Ok().json(response))
}

// ============ Profile Endpoints ============

#[get("/profile")]
pub async fn get_profile(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<impl Responder, HttpAppError> {
    let (user_id, _) = extract_auth_claims(&http_req)?;

    let response = GetProfileUseCase::execute(db.get_ref(), user_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[post("/setup-profile")]
pub async fn setup_profile(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    req: web::Json<SetupProfileRequest>,
) -> Result<impl Responder, HttpAppError> {
    let (user_id, _) = extract_auth_claims(&http_req)?;

    let response = SetupProfileUseCase::execute(db.get_ref(), user_id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

// ============ PIN Endpoints ============

#[post("/setup-pin")]
pub async fn setup_pin(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    req: web::Json<SetupPinRequest>,
) -> Result<impl Responder, HttpAppError> {
    let (user_id, _) = extract_auth_claims(&http_req)?;

    let response = SetupPinUseCase::execute(db.get_ref(), user_id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[post("/verify-pin")]
pub async fn verify_pin(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    redis_conn: web::Data<MultiplexedConnection>,
    req: web::Json<VerifyPinRequest>,
) -> Result<impl Responder, HttpAppError> {
    let (user_id, _) = extract_auth_claims(&http_req)?;

    let mut conn = redis_conn.get_ref().clone();

    // Special handling for legacy behavior (optional: standard AppError handling handles this too via 400/500)
    // But previous code mapped "No PIN set" to specific error code.
    // If UseCase returns AppError::Validation("No PIN set"), it maps to 400.
    // We can just propagate or wrap if needed. For now, strict propagation.
    let response = VerifyPinUseCase::execute(db.get_ref(), &mut conn, user_id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[get("/pin-status")]
pub async fn pin_status(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<impl Responder, HttpAppError> {
    let (user_id, _) = extract_auth_claims(&http_req)?;

    let response = CheckPinStatusUseCase::execute(db.get_ref(), user_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[post("/skip-pin-setup")]
pub async fn skip_pin_setup(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<impl Responder, HttpAppError> {
    let (user_id, _) = extract_auth_claims(&http_req)?;

    let response = SkipPinSetupUseCase::execute(db.get_ref(), user_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

// ============ Token Refresh ============

#[post("/refresh-token")]
pub async fn refresh_token(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    req: web::Json<RefreshTokenRequest>,
) -> Result<impl Responder, HttpAppError> {
    let auth_config = AuthConfig {
        jwt_secret: config.jwt_secret.clone(),
        jwt_expiration: config.jwt_expiration,
        refresh_token_expiration: config.refresh_token_expiration,
    };

    let response = RefreshTokenUseCase::execute(db.get_ref(), &auth_config, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

// ============ Device Linking Endpoints ============

#[post("/link/create")]
pub async fn create_linking_session(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<impl Responder, HttpAppError> {
    let (_, device_id) = extract_auth_claims(&http_req)?;

    let response = CreateLinkingSessionUseCase::execute(db.get_ref(), device_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[post("/link/complete")]
pub async fn complete_linking(
    db: web::Data<DatabaseConnection>,
    req: web::Json<CompleteLinkingRequest>,
) -> Result<impl Responder, HttpAppError> {
    let response = CompleteLinkingUseCase::execute(db.get_ref(), req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[post("/link/approve")]
pub async fn approve_linking(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    req: web::Json<ApproveLinkingRequest>,
) -> Result<impl Responder, HttpAppError> {
    let (_, device_id) = extract_auth_claims(&http_req)?;

    let response = ApproveLinkingUseCase::execute(db.get_ref(), device_id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

// ============ Device Management Endpoints ============

#[get("")]
pub async fn list_devices(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<impl Responder, HttpAppError> {
    let (user_id, _) = extract_auth_claims(&http_req)?;

    let response = ListDevicesUseCase::execute(db.get_ref(), user_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[delete("/{device_id}")]
pub async fn unlink_device(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<i64>,
) -> Result<impl Responder, HttpAppError> {
    let (user_id, current_device_id) = extract_auth_claims(&http_req)?;
    let target_device_id = path.into_inner();

    let response = UnlinkDeviceUseCase::execute(db.get_ref(), user_id, current_device_id, target_device_id).await?;
    Ok(HttpResponse::Ok().json(response))
}
