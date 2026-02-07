use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PreKeyBundleResponse {
    pub device_id: i64,
    pub registration_id: i32,
    pub identity_key: Vec<u8>,
    pub signed_prekey: SignedPreKeyDto,
    pub one_time_prekey: Option<PreKeyDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedPreKeyDto {
    pub id: i32,
    pub key: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreKeyDto {
    pub id: i32,
    pub key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadKeysDto {
    pub registration_id: i32,
    pub identity_key: Vec<u8>,
    pub signed_prekey: SignedPreKeyDto,
    pub one_time_prekeys: Vec<PreKeyDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceDto {
    pub device_id: i64,
    pub registration_id: i32,
    pub last_seen_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}
