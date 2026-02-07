use super::dtos::AcceptFriendRequest;
use vyry_core::entities::friends;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait, ModelTrait,
};
use chrono::Utc;
use crate::AppError;

pub struct AcceptFriendUseCase;

impl AcceptFriendUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        req: AcceptFriendRequest,
    ) -> Result<(), AppError> {
        // Find the pending request (Requester -> User)
        // Here, req.user_id is the acceptor (B), req.requester_id is A
        // We look for A -> B with status Pending
        
        let friend_model = friends::Entity::find()
            .filter(
                friends::Column::UserId.eq(req.requester_id)
                    .and(friends::Column::FriendId.eq(req.user_id))
                    .and(friends::Column::Status.eq(0)) // Pending
            )
            .one(db)
            .await
            .map_err(AppError::from)?;

        let friend_record = friend_model.ok_or_else(|| AppError::NotFound("Friend request not found".to_string()))?;

        if !req.accept {
            // Reject: Delete the request
            friend_record.delete(db).await.map_err(AppError::from)?;
            return Ok(());
        }

        let txn = db.begin().await.map_err(AppError::from)?;

        // 1. Update A->B to Accepted
        let mut active_model: friends::ActiveModel = friend_record.into();
        active_model.status = Set(1); // Accepted
        active_model.updated_at = Set(Utc::now().into());
        active_model.update(&txn).await.map_err(AppError::from)?;

        // 2. Create B->A as Accepted (Bidirectional)
        let reverse_friend = friends::ActiveModel {
            user_id: Set(req.user_id),
            friend_id: Set(req.requester_id),
            status: Set(1), // Accepted
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        
        // Handle case where B->A might already exist (e.g., they both requested each other)
        // For simplicity, we assume generic insert here, but in production, check existence.
        // Or use on_conflict (upsert). SeaORM supports on_conflict.
        
        // Simple check
        let reverse_exists = friends::Entity::find()
             .filter(
                friends::Column::UserId.eq(req.user_id)
                    .and(friends::Column::FriendId.eq(req.requester_id))
            )
            .one(&txn)
            .await
            .map_err(AppError::from)?;

        if let Some(existing) = reverse_exists {
            let mut rev_active: friends::ActiveModel = existing.into();
            rev_active.status = Set(1);
            rev_active.updated_at = Set(Utc::now().into());
            rev_active.update(&txn).await.map_err(AppError::from)?;
        } else {
            reverse_friend.insert(&txn).await.map_err(AppError::from)?;
        }

        txn.commit().await.map_err(AppError::from)?;

        Ok(())
    }
}
