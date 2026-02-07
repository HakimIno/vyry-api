use super::dtos::FriendDto;
use vyry_core::entities::{friends, users};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, 
};
use uuid::Uuid;
use crate::AppError;

pub struct ListFriendsUseCase;

impl ListFriendsUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<FriendDto>, AppError> {
        // Query friends where user_id = user_id AND status = 1 (Accepted)
        // Join with users table to get details
        
        let results = friends::Entity::find()
            .filter(
                friends::Column::UserId.eq(user_id)
                    .and(friends::Column::Status.eq(1)) // Accepted only
            )
            .find_also_related(users::Entity)
            .all(db)
            .await
            .map_err(AppError::from)?;

        let mut friends_list = Vec::new();

        for (friend_rel, user_rel) in results {
            if let Some(user) = user_rel {
                friends_list.push(FriendDto {
                    user_id: user.user_id,
                    username: user.username,
                    display_name: user.display_name,
                    profile_picture: user.profile_picture,
                    status: friend_rel.status,
                    created_at: friend_rel.created_at.timestamp(),
                });
            }
        }

        Ok(friends_list)
    }
}
