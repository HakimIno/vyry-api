use super::dtos::AddFriendRequest;
use core::entities::friends;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use chrono::Utc;

pub struct AddFriendUseCase;

impl AddFriendUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        req: AddFriendRequest,
    ) -> Result<(), String> {
        if req.user_id == req.friend_id {
            return Err("Cannot add yourself as friend".to_string());
        }

        // Check if relationship already exists
        let exists = friends::Entity::find()
            .filter(
                friends::Column::UserId.eq(req.user_id)
                    .and(friends::Column::FriendId.eq(req.friend_id))
            )
            .one(db)
            .await
            .map_err(|e| e.to_string())?;

        if exists.is_some() {
            return Err("Friend request already sent or users are already friends".to_string());
        }

        // Create forward relationship (Requester -> Target)
        let friend_req = friends::ActiveModel {
            user_id: Set(req.user_id),
            friend_id: Set(req.friend_id),
            status: Set(0), // Pending
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        // Create reverse relationship (Target -> Requester) - Optional?
        // Usually, for "Friend Request", only one record is needed initially, 
        // OR two records are created: one sent, one received.
        // Let's stick to single record for the request, but "Friends" usually implies bidirectional when accepted.
        // Strategy: 
        // - Record A->B (Status: Pending) means A asked B.
        // - When B accepts, we update A->B to Accepted AND create B->A as Accepted.
        
        friend_req.insert(db).await.map_err(|e| e.to_string())?;

        Ok(())
    }
}
