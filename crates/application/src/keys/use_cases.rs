use super::dtos::{PreKeyBundleResponse, PreKeyDto, SignedPreKeyDto};
use core::entities::{devices, one_time_prekeys};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct GetPreKeyBundleUseCase;

impl GetPreKeyBundleUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
        device_id: i64,
    ) -> Result<PreKeyBundleResponse, String> {
        // 1. Fetch Device
        let device = devices::Entity::find()
            .filter(devices::Column::UserId.eq(user_id))
            .filter(devices::Column::DeviceId.eq(device_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Device not found")?;

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
            .map_err(|e| e.to_string())?;

        // If we found a prekey, we should ideally delete it.
        if let Some(ref pk) = prekey {
             // TODO: Delete prekey in a transaction? 
             // For now, let's just return it. 
             // If we delete it, we need to make sure we don't run out.
             // The client should replenish keys.
             let _ = one_time_prekeys::Entity::delete_by_id((pk.device_id, pk.prekey_id))
                 .exec(db)
                 .await
                 .map_err(|e| e.to_string())?;
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
