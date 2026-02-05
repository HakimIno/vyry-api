use super::dtos::SearchUserResponse;
use core::entities::users;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

pub struct SearchUserUseCase;

impl SearchUserUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        query: String, 
    ) -> Result<Option<SearchUserResponse>, String> {
        let mut condition = Condition::any()
            .add(users::Column::Username.contains(query.clone()))
            .add(users::Column::PhoneNumber.contains(query.clone()));

        if let Ok(uuid) = Uuid::parse_str(&query) {
            condition = condition.add(users::Column::UserId.eq(uuid));
        }

        let user = users::Entity::find()
            .filter(condition)
            .one(db)
            .await
            .map_err(|e| e.to_string())?;

        if user.is_none() {
             return Ok(None);
        }

        let u = user.unwrap();
        Ok(Some(SearchUserResponse {
            user_id: u.user_id,
            username: u.username,
            display_name: u.display_name,
            profile_picture: u.profile_picture,
        }))
    }
}
