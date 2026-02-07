use actix_web::{get, web, HttpResponse, Responder};
use application::keys::use_cases::GetPreKeyBundleUseCase;
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use crate::handlers::error_handler::HttpAppError;
use application::AppError;

#[get("/keys/{user_id}/devices/{device_id}")]
pub async fn get_prekey_bundle(
    db: web::Data<DatabaseConnection>,
    path: web::Path<(Uuid, i64)>,
) -> Result<impl Responder, HttpAppError> {
    let (user_id, device_id) = path.into_inner();

    let response = GetPreKeyBundleUseCase::execute(db.get_ref(), user_id, device_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

use application::keys::dtos::UploadKeysDto;
use application::keys::use_cases::UploadKeysUseCase;
use actix_web::post;

#[post("/keys")]
pub async fn upload_keys(
    db: web::Data<DatabaseConnection>,
    user: crate::extractors::AuthUser,
    path: web::Json<UploadKeysDto>,
) -> Result<impl Responder, HttpAppError> {
    let user_id = Uuid::parse_str(&user.sub).map_err(|_| AppError::Authentication("Invalid user ID".to_string()))?;
    let device_id = user.device_id;
    
    UploadKeysUseCase::execute(db.get_ref(), user_id, device_id, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true })))
}

use application::keys::use_cases::GetUserDevicesUseCase;

#[get("/keys/{user_id}/devices")]
pub async fn get_user_devices(
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, HttpAppError> {
    let user_id = path.into_inner();

    let devices = GetUserDevicesUseCase::execute(db.get_ref(), user_id).await?;
    Ok(HttpResponse::Ok().json(devices))
}
