use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, HttpMessage};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use core::entities::{users, devices, one_time_prekeys};
use core::signal::wrapper::create_signal_keys;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub device_id: i64,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub phone_number: String,
    pub device_name: Option<String>,
    pub platform: i16,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub device_id: i64,
    pub device_uuid: Uuid,
}

#[post("/api/v1/register")]
pub async fn register(
    db: web::Data<DatabaseConnection>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let argon2 = Argon2::default();
    let salt = SaltString::from_b64("c29tZXNhbHQAAAAAAAAAAAAA").unwrap(); // "somesalt" in base64
    let phone_hash = argon2
        .hash_password(req.phone_number.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_default();

    let signal_keys = match create_signal_keys() {
        Ok(keys) => keys,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate Signal keys: {}", e)
        })),
    };

    let user = users::ActiveModel {
        user_id: Set(Uuid::new_v4()),
        phone_number: Set(req.phone_number.clone()),
        phone_number_hash: Set(phone_hash.as_bytes().to_vec()),
        username: Set(None),
        display_name: Set(None),
        bio: Set(None),
        profile_picture: Set(None),
        last_seen_at: Set(None),
        is_online: Set(false),
        is_deleted: Set(false),
        deleted_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };

    let user = match user.insert(db.get_ref()).await {
        Ok(u) => u,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create user: {}", e)
        })),
    };

    let device = devices::ActiveModel {
        device_id: Set(0),
        user_id: Set(user.user_id),
        device_uuid: Set(Uuid::new_v4()),
        device_name: Set(req.device_name.clone()),
        platform: Set(req.platform),
        identity_key_public: Set(signal_keys.identity_key_pair.public_key.clone()),
        registration_id: Set(signal_keys.registration_id as i32),
        signed_prekey_id: Set(signal_keys.signed_prekey.id as i32),
        signed_prekey_public: Set(signal_keys.signed_prekey.public_key.clone()),
        signed_prekey_signature: Set(signal_keys.signed_prekey.signature.clone()),
        last_seen_at: Set(chrono::Utc::now().into()),
        created_at: Set(chrono::Utc::now().into()),
    };

    let device = match device.insert(db.get_ref()).await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create device: {}", e)
        })),
    };

    for prekey in signal_keys.one_time_prekeys.iter() {
        let otpk = one_time_prekeys::ActiveModel {
            device_id: Set(device.device_id),
            prekey_id: Set(prekey.id as i32),
            public_key: Set(prekey.public_key.clone()),
        };
        let _ = otpk.insert(db.get_ref()).await;
    }

    HttpResponse::Ok().json(RegisterResponse {
        user_id: user.user_id,
        device_id: device.device_id,
        device_uuid: device.device_uuid,
    })
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub phone_number: String,
    pub device_uuid: Uuid,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id: Uuid,
    pub device_id: i64,
}

#[post("/api/v1/auth/login")]
pub async fn login(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;

    let user = match users::Entity::find()
        .filter(users::Column::PhoneNumber.eq(&req.phone_number))
        .one(db.get_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {}", e)
        })),
    };

    let device = match devices::Entity::find()
        .filter(devices::Column::DeviceUuid.eq(req.device_uuid))
        .filter(devices::Column::UserId.eq(user.user_id))
        .one(db.get_ref())
        .await
    {
        Ok(Some(d)) => d,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Device not found"
        })),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {}", e)
        })),
    };

    let now = Utc::now();

    let access_claims = Claims {
        sub: user.user_id.to_string(),
        device_id: device.device_id,
        iat: now.timestamp(),
        exp: (now + Duration::seconds(config.jwt_expiration)).timestamp(),
        token_type: "access".to_string(),
    };

    let refresh_claims = Claims {
        sub: user.user_id.to_string(),
        device_id: device.device_id,
        iat: now.timestamp(),
        exp: (now + Duration::seconds(config.refresh_token_expiration)).timestamp(),
        token_type: "refresh".to_string(),
    };

    let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());

    let access_token = match encode(&Header::default(), &access_claims, &encoding_key) {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to generate access token: {}", e)
            }))
        }
    };

    let refresh_token = match encode(&Header::default(), &refresh_claims, &encoding_key) {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to generate refresh token: {}", e)
            }))
        }
    };

    HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        user_id: user.user_id,
    })
}

#[get("/api/v1/auth/me")]
pub async fn get_me(req: HttpRequest) -> impl Responder {
    use crate::handlers::auth::Claims;

    if let Some(claims) = req.extensions().get::<Claims>() {
        let user_id = match Uuid::parse_str(&claims.sub) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid user id in token"
                }));
            }
        };

        HttpResponse::Ok().json(MeResponse {
            user_id,
            device_id: claims.device_id,
        })
    } else {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing or invalid token"
        }))
    }
}
