use super::dtos::{PreKeyBundleResponse, PreKeyDto, SignedPreKeyDto};
use vyry_core::entities::{devices, one_time_prekeys};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;
use crate::AppError;

pub struct GetPreKeyBundleUseCase;

impl GetPreKeyBundleUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
        device_id: i64,
    ) -> Result<PreKeyBundleResponse, AppError> {
        // 1. Fetch Device - with fallback to any device if requested device_id doesn't exist
        let device = devices::Entity::find()
            .filter(devices::Column::UserId.eq(user_id))
            .filter(devices::Column::DeviceId.eq(device_id))
            .one(db)
            .await
            .map_err(AppError::from)?;

        // Fallback: if requested device doesn't exist, get the first available device for this user
        let device = match device {
            Some(d) => d,
            None => {
                devices::Entity::find()
                    .filter(devices::Column::UserId.eq(user_id))
                    .order_by_asc(devices::Column::DeviceId)
                    .one(db)
                    .await
                    .map_err(AppError::from)?
                    .ok_or_else(|| AppError::NotFound("No devices found for user".to_string()))?
            }
        };

        // Guard: return error if client hasn't uploaded keys yet
        if device.identity_key_public.is_empty() {
            return Err(AppError::NotFound("Keys not yet uploaded for this device".to_string()));
        }

        // 2. Fetch One One-Time Prekey
        // We need to fetch one and delete it (or mark as used).
        // For simplicity, we just fetch one. In a real Signal implementation, 
        // we should delete it, but that requires a transaction and might run out of keys.
        // Signal server usually deletes it.
        
        let prekey = one_time_prekeys::Entity::find()
            .filter(one_time_prekeys::Column::DeviceId.eq(device_id))
            .order_by_asc(one_time_prekeys::Column::PrekeyId) // Get oldest
            .one(db)
            .await
            .map_err(AppError::from)?;

        // If we found a prekey, we should ideally delete it.
        if let Some(ref pk) = prekey {
             // TODO: Delete prekey in a transaction? 
             // For now, let's just return it. 
             // If we delete it, we need to make sure we don't run out.
             // The client should replenish keys.
             let _ = one_time_prekeys::Entity::delete_by_id((pk.device_id, pk.prekey_id))
                 .exec(db)
                 .await
                 .map_err(AppError::from)?;
        }

        Ok(PreKeyBundleResponse {
            device_id: device.device_id,
            registration_id: device.registration_id,
            identity_key: device.identity_key_public,
            signed_prekey: SignedPreKeyDto {
                id: device.signed_prekey_id,
                key: device.signed_prekey_public,
                signature: device.signed_prekey_signature,
            },
            one_time_prekey: prekey.map(|pk| PreKeyDto {
                id: pk.prekey_id,
                key: pk.public_key,
            }),
        })
    }
}

pub struct UploadKeysUseCase;

impl UploadKeysUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
        device_id: i64,
        dto: super::dtos::UploadKeysDto,
    ) -> Result<(), AppError> {
        use sea_orm::ActiveModelTrait;
        use sea_orm::Set;

        // 1. Update Device with Identity Key and Signed PreKey
        // We find the device first to ensure it belongs to the user
        let device = devices::Entity::find()
             .filter(devices::Column::UserId.eq(user_id))
             .filter(devices::Column::DeviceId.eq(device_id))
             .one(db)
             .await
             .map_err(AppError::from)?
             .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        let mut device_active: devices::ActiveModel = device.into();
        
        device_active.identity_key_public = Set(dto.identity_key);
        device_active.registration_id = Set(dto.registration_id);
        device_active.signed_prekey_id = Set(dto.signed_prekey.id);
        device_active.signed_prekey_public = Set(dto.signed_prekey.key);
        device_active.signed_prekey_signature = Set(dto.signed_prekey.signature);
        
        device_active.update(db).await.map_err(AppError::from)?;

        // 2. Insert One-Time PreKeys
        if !dto.one_time_prekeys.is_empty() {
            // First, delete existing prekeys for this device to avoid duplicates
            one_time_prekeys::Entity::delete_many()
                .filter(one_time_prekeys::Column::DeviceId.eq(device_id))
                .exec(db)
                .await
                .map_err(AppError::from)?;

            let keys: Vec<one_time_prekeys::ActiveModel> = dto.one_time_prekeys.into_iter().map(|k| {
                one_time_prekeys::ActiveModel {
                    device_id: Set(device_id),
                    prekey_id: Set(k.id),
                    public_key: Set(k.key),
                }
            }).collect();

            // Insert new prekeys
             one_time_prekeys::Entity::insert_many(keys)
                .exec(db)
                .await
                .map_err(AppError::from)?;
        }

        Ok(())
    }
}

pub struct GetUserDevicesUseCase;

impl GetUserDevicesUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<super::dtos::DeviceDto>, AppError> {
        let devices = devices::Entity::find()
            .filter(devices::Column::UserId.eq(user_id))
            .filter(devices::Column::IsActive.eq(true))
            .order_by_asc(devices::Column::LastSeenAt)
            .all(db)
            .await
            .map_err(AppError::from)?;

        Ok(devices.into_iter().map(|d| super::dtos::DeviceDto {
            device_id: d.device_id,
            registration_id: d.registration_id,
            last_seen_at: Some(d.last_seen_at),
        }).collect())
    }
}
