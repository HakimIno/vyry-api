use crate::config::Config;
use crate::handlers::error_handler::app_error_to_response;
use actix_web::{delete, get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use application::auth::{
    dtos::*,
    use_cases::*,
};
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use tracing::{error, info};
use uuid::Uuid;

/// Extract user_id and device_id from JWT claims in request extensions
fn extract_auth_claims(req: &HttpRequest) -> Option<(Uuid, i64)> {
    req.extensions()
        .get::<Claims>()
        .map(|claims| {
            let user_id = claims.sub.parse::<Uuid>().ok()?;
            Some((user_id, claims.device_id))
        })
        .flatten()
}

// ============ OTP Endpoints ============

#[post("/request-otp")]
pub async fn request_otp(
    redis_conn: web::Data<MultiplexedConnection>,
    req: web::Json<RequestOtpRequest>,
) -> impl Responder {
    let mut conn = redis_conn.get_ref().clone();

    match RequestOtpUseCase::execute(&mut conn, req.into_inner()).await {
        Ok(otp) => {
            info!("OTP generated for testing: {}", otp);
            HttpResponse::Ok().json(RequestOtpResponse {
                message: "OTP sent successfully".to_string(),
                expires_in_seconds: 180,
            })
        }
        Err(e) => {
            error!("Request OTP error: {}", e);
            app_error_to_response(e)
        }
    }
}

#[post("/verify-otp")]
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
        Ok(response) => {
            info!("OTP verified successfully for user: {}", response.user_id);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Verify OTP error: {}", e);
            app_error_to_response(e)
        }
    }
}

// ============ Profile Endpoints ============

#[get("/profile")]
pub async fn get_profile(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let (user_id, _) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    match GetProfileUseCase::execute(db.get_ref(), user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(AuthErrorResponse {
            error: e.to_string(),
            error_code: "INTERNAL_ERROR".to_string(),
            retry_after_seconds: None,
        }),
    }
}

#[post("/setup-profile")]
pub async fn setup_profile(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    req: web::Json<SetupProfileRequest>,
) -> impl Responder {
    let (user_id, _) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    match SetupProfileUseCase::execute(db.get_ref(), user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(AuthErrorResponse {
            error: e.to_string(),
            error_code: "INVALID_REQUEST".to_string(),
            retry_after_seconds: None,
        }),
    }
}

// ============ PIN Endpoints ============

#[post("/setup-pin")]
pub async fn setup_pin(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    req: web::Json<SetupPinRequest>,
) -> impl Responder {
    let (user_id, _) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    match SetupPinUseCase::execute(db.get_ref(), user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(AuthErrorResponse {
            error: e.to_string(),
            error_code: "INVALID_REQUEST".to_string(),
            retry_after_seconds: None,
        }),
    }
}

#[post("/verify-pin")]
pub async fn verify_pin(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    redis_conn: web::Data<MultiplexedConnection>,
    req: web::Json<VerifyPinRequest>,
) -> impl Responder {
    let (user_id, _) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    let mut conn = redis_conn.get_ref().clone();

    match VerifyPinUseCase::execute(db.get_ref(), &mut conn, user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            // If user doesn't have a PIN, return special error code
            if e.to_string().contains("No PIN set") {
                return HttpResponse::BadRequest().json(AuthErrorResponse {
                    error: e.to_string(),
                    error_code: "NO_PIN_SET".to_string(),
                    retry_after_seconds: None,
                });
            }
            HttpResponse::BadRequest().json(AuthErrorResponse {
                error: e.to_string(),
                error_code: "INVALID_REQUEST".to_string(),
                retry_after_seconds: None,
            })
        },
    }
}

#[get("/pin-status")]
pub async fn pin_status(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let (user_id, _) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    match CheckPinStatusUseCase::execute(db.get_ref(), user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(AuthErrorResponse {
            error: e.to_string(),
            error_code: "INTERNAL_ERROR".to_string(),
            retry_after_seconds: None,
        }),
    }
}

#[post("/skip-pin-setup")]
pub async fn skip_pin_setup(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let (user_id, _) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    match SkipPinSetupUseCase::execute(db.get_ref(), user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(AuthErrorResponse {
            error: e.to_string(),
            error_code: "INVALID_REQUEST".to_string(),
            retry_after_seconds: None,
        }),
    }
}

// ============ Token Refresh ============

#[post("/refresh-token")]
pub async fn refresh_token(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    req: web::Json<RefreshTokenRequest>,
) -> impl Responder {
    let auth_config = AuthConfig {
        jwt_secret: config.jwt_secret.clone(),
        jwt_expiration: config.jwt_expiration,
        refresh_token_expiration: config.refresh_token_expiration,
    };

    match RefreshTokenUseCase::execute(db.get_ref(), &auth_config, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Invalid") || error_msg.contains("expired") {
                HttpResponse::Unauthorized().json(AuthErrorResponse {
                    error: error_msg,
                    error_code: "INVALID_TOKEN".to_string(),
                    retry_after_seconds: None,
                })
            } else {
                HttpResponse::InternalServerError().json(AuthErrorResponse {
                    error: error_msg,
                    error_code: "INTERNAL_ERROR".to_string(),
                    retry_after_seconds: None,
                })
            }
        }
    }
}

// ============ Device Linking Endpoints ============

#[post("/link/create")]
pub async fn create_linking_session(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let (_, device_id) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    match CreateLinkingSessionUseCase::execute(db.get_ref(), device_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(AuthErrorResponse {
            error: e.to_string(),
            error_code: "INVALID_REQUEST".to_string(),
            retry_after_seconds: None,
        }),
    }
}

#[post("/link/complete")]
pub async fn complete_linking(
    db: web::Data<DatabaseConnection>,
    req: web::Json<CompleteLinkingRequest>,
) -> impl Responder {
    match CompleteLinkingUseCase::execute(db.get_ref(), req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(AuthErrorResponse {
            error: e.to_string(),
            error_code: "INVALID_REQUEST".to_string(),
            retry_after_seconds: None,
        }),
    }
}

#[post("/link/approve")]
pub async fn approve_linking(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    req: web::Json<ApproveLinkingRequest>,
) -> impl Responder {
    let (_, device_id) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    match ApproveLinkingUseCase::execute(db.get_ref(), device_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(AuthErrorResponse {
            error: e.to_string(),
            error_code: "INVALID_REQUEST".to_string(),
            retry_after_seconds: None,
        }),
    }
}

// ============ Device Management Endpoints ============

#[get("")]
pub async fn list_devices(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let (user_id, _) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    match ListDevicesUseCase::execute(db.get_ref(), user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(AuthErrorResponse {
            error: e.to_string(),
            error_code: "INTERNAL_ERROR".to_string(),
            retry_after_seconds: None,
        }),
    }
}

#[delete("/{device_id}")]
pub async fn unlink_device(
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<i64>,
) -> impl Responder {
    let (user_id, current_device_id) = match extract_auth_claims(&http_req) {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(AuthErrorResponse {
                error: "Unauthorized".to_string(),
                error_code: "UNAUTHORIZED".to_string(),
                retry_after_seconds: None,
            })
        }
    };

    let target_device_id = path.into_inner();

    match UnlinkDeviceUseCase::execute(db.get_ref(), user_id, current_device_id, target_device_id)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Unauthorized") {
                HttpResponse::Forbidden().json(AuthErrorResponse {
                    error: error_msg,
                    error_code: "FORBIDDEN".to_string(),
                    retry_after_seconds: None,
                })
            } else if error_msg.contains("Cannot unlink current") {
                HttpResponse::BadRequest().json(AuthErrorResponse {
                    error: error_msg,
                    error_code: "INVALID_REQUEST".to_string(),
                    retry_after_seconds: None,
                })
            } else {
                HttpResponse::InternalServerError().json(AuthErrorResponse {
                    error: error_msg,
                    error_code: "INTERNAL_ERROR".to_string(),
                    retry_after_seconds: None,
                })
            }
        }
    }
}
