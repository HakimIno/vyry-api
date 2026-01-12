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
