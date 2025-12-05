use actix_web::{get, web, HttpResponse, Responder};
use application::keys::use_cases::GetPreKeyBundleUseCase;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

#[get("/api/v1/keys/{user_id}/devices/{device_id}")]
pub async fn get_prekey_bundle(
    db: web::Data<DatabaseConnection>,
    path: web::Path<(Uuid, i64)>,
) -> impl Responder {
    let (user_id, device_id) = path.into_inner();

    match GetPreKeyBundleUseCase::execute(db.get_ref(), user_id, device_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            if e == "Device not found" {
                HttpResponse::NotFound().json(serde_json::json!({ "error": e }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({ "error": e }))
            }
        }
    }
}
