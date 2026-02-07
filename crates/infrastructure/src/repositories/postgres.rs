// PostgreSQL implementation of repositories
// This is the current implementation using Sea-ORM
// Note: Using fully qualified async_trait to avoid conflict with crate::core

use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait};
use uuid::Uuid;

use super::traits::UserRepository;

pub struct PostgresUserRepository {
    db: DatabaseConnection,
}

impl PostgresUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[::async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<core::entities::users::Model>> {
        Ok(core::entities::users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?)
    }

    async fn find_by_phone_number(&self, phone_number: &str) -> anyhow::Result<Option<core::entities::users::Model>> {
        use sea_orm::ColumnTrait;
        use sea_orm::QueryFilter;
        
        Ok(core::entities::users::Entity::find()
            .filter(core::entities::users::Column::PhoneNumber.eq(phone_number))
            .one(&self.db)
            .await?)
    }

    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<core::entities::users::Model>> {
        use sea_orm::ColumnTrait;
        use sea_orm::QueryFilter;
        
        Ok(core::entities::users::Entity::find()
            .filter(core::entities::users::Column::Username.eq(username))
            .one(&self.db)
            .await?)
    }

    async fn create(&self, user: core::entities::users::ActiveModel) -> anyhow::Result<core::entities::users::Model> {
        Ok(user.insert(&self.db).await?)
    }

    async fn update(&self, user: core::entities::users::ActiveModel) -> anyhow::Result<core::entities::users::Model> {
        Ok(user.update(&self.db).await?)
    }
}

pub struct PostgresSignalRepository {
    db: DatabaseConnection,
}

impl PostgresSignalRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[::async_trait::async_trait]
impl super::traits::SignalRepository for PostgresSignalRepository {
    async fn get_prekey_bundle(&self, device_id: i64) -> anyhow::Result<Option<core::signal::wrapper::PreKeyBundle>> {
        // 1. Fetch device to get Identity Key and Signed PreKey
        let device = core::entities::devices::Entity::find_by_id(device_id)
            .one(&self.db)
            .await?;

        let device = match device {
            Some(d) => d,
            None => return Ok(None),
        };

        // 2. Fetch one One-Time PreKey (and consume it logically, usually we delete it or mark as used)
        // For now, let's just fetch one. A real implementation should delete it inside a transaction.
        use sea_orm::{QueryOrder, QuerySelect};
        let otpk = core::entities::one_time_prekeys::Entity::find()
            .filter(core::entities::one_time_prekeys::Column::DeviceId.eq(device_id))
            .order_by_asc(core::entities::one_time_prekeys::Column::PrekeyId)
            .one(&self.db)
            .await?;
            
        // If we found one, we should ideally delete it.
        // Assuming this is inside a transaction or we accept potential race conditions for now.
        // In a strict X3DH, the server serves it and deletes it.
        let one_time_prekey = if let Some(k) = otpk {
            // DELETE IT
            let res = core::entities::one_time_prekeys::Entity::delete_by_id((k.device_id, k.prekey_id))
                .exec(&self.db)
                .await?;
            
            if res.rows_affected == 0 {
                 // Race condition: someone else took it. Try again? Or just return None for OTPK.
                 None
            } else {
                 Some(core::signal::wrapper::PublicPreKey {
                    id: k.prekey_id as u32,
                    key: k.public_key,
                })
            }
        } else {
            None
        };

        Ok(Some(core::signal::wrapper::PreKeyBundle {
            device_id: device.device_id as u32,
            registration_id: device.registration_id as u32,
            identity_key: device.identity_key_public,
            signed_prekey: core::signal::wrapper::PublicSignedPreKey {
                id: device.signed_prekey_id as u32,
                key: device.signed_prekey_public,
                signature: device.signed_prekey_signature,
            },
            one_time_prekey,
        }))
    }

    async fn store_one_time_prekeys(&self, device_id: i64, keys: Vec<core::signal::wrapper::PublicPreKey>) -> anyhow::Result<()> {
        let models: Vec<core::entities::one_time_prekeys::ActiveModel> = keys.into_iter().map(|k| {
            use sea_orm::Set;
            core::entities::one_time_prekeys::ActiveModel {
                 device_id: Set(device_id),
                 prekey_id: Set(k.id as i32),
                 public_key: Set(k.key),
            }
        }).collect();

        if !models.is_empty() {
             core::entities::one_time_prekeys::Entity::insert_many(models)
                .exec(&self.db)
                .await?;
        }
        Ok(())
    }

    async fn count_one_time_prekeys(&self, device_id: i64) -> anyhow::Result<u64> {
         use sea_orm::{PaginatorTrait, QueryFilter};
         let count = core::entities::one_time_prekeys::Entity::find()
             .filter(core::entities::one_time_prekeys::Column::DeviceId.eq(device_id))
             .count(&self.db)
             .await?;
         Ok(count)
    }
}
