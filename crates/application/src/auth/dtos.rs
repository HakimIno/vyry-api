use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============ JWT Claims ============

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub device_id: i64,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

// ============ OTP ============

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestOtpRequest {
    pub phone_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestOtpResponse {
    pub message: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    pub phone_number: String,
    pub otp: String,
    pub device_uuid: Uuid,
    pub device_name: Option<String>,
    pub platform: Option<i16>, // 1 = iOS, 2 = Android, 3 = Web, 4 = Desktop
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub device_id: i64,
    pub is_new_user: bool,
    pub requires_profile_setup: bool,
    pub requires_pin: bool, // true if registration_lock is enabled
}

// ============ Profile Setup ============

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupProfileRequest {
    pub display_name: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub profile_picture_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupProfileResponse {
    pub user_id: Uuid,
    pub display_name: String,
    pub username: Option<String>,
    pub bio: Option<String>,
    pub profile_picture_url: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetProfileResponse {
    pub user_id: Uuid,
    pub phone_number: String, // Masked phone number
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub bio: Option<String>,
    pub profile_picture_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============ PIN / 2FA ============

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupPinRequest {
    pub pin: String, // 4-6 digits or alphanumeric passphrase
    pub confirm_pin: String,
    #[serde(default)]
    pub enable_registration_lock: bool, // Auto-enable 2FA
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupPinResponse {
    pub registration_lock_enabled: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyPinRequest {
    pub pin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyPinResponse {
    pub verified: bool,
    pub attempts_remaining: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemovePinRequest {
    pub current_pin: String,
}

// ============ Token Refresh ============

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

// ============ Device Linking ============

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLinkingSessionRequest {
    pub device_id: i64, // Primary device ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLinkingSessionResponse {
    pub session_id: Uuid,
    pub qr_code_data: String, // Base64 encoded QR data
    pub qr_code_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteLinkingRequest {
    pub qr_code_token: String,
    pub device_uuid: Uuid,
    pub device_name: Option<String>,
    pub platform: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteLinkingResponse {
    pub session_id: Uuid,
    pub status: String, // "pending_approval"
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveLinkingRequest {
    pub session_id: Uuid,
    pub approve: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveLinkingResponse {
    pub session_id: Uuid,
    pub new_device_id: Option<i64>,
    pub status: String, // "approved" or "rejected"
}

// ============ Device Management ============

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: i64,
    pub device_uuid: Uuid,
    pub device_name: Option<String>,
    pub platform: i16,
    pub device_type: String, // "primary" or "linked"
    pub is_active: bool,
    pub linked_at: Option<DateTime<Utc>>,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDevicesResponse {
    pub devices: Vec<DeviceInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnlinkDeviceRequest {
    pub device_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnlinkDeviceResponse {
    pub unlinked: bool,
    pub message: String,
}

// ============ Auth Error Types ============

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
}

