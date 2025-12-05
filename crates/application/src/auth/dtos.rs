use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub device_id: i64,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestOtpRequest {
    pub phone_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    pub phone_number: String,
    pub otp: String,
    pub device_uuid: Uuid,
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub device_id: i64,
    pub is_new_user: bool,
}
