use super::dtos::{ConversationResponse, CreateDirectConversationRequest};
use vyry_core::entities::{conv_members, conversations};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter, Set, TransactionTrait, sea_query,
};
use uuid::Uuid;
use chrono::Utc;
use crate::AppError;

pub struct CreateDirectConversationUseCase;

impl CreateDirectConversationUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
        req: CreateDirectConversationRequest,
    ) -> Result<ConversationResponse, AppError> {
        // Direct conversation type constant
        const CONV_TYPE_DIRECT: i16 = 1;

        // 1. Check if direct conversation already exists
        // SELECT c.conv_id FROM conversations c
        // JOIN conv_members cm1 ON c.conv_id = cm1.conv_id
        // JOIN conv_members cm2 ON c.conv_id = cm2.conv_id
        // WHERE cm1.user_id = user_id AND cm2.user_id = friend_id AND c.conv_type = 1
        
        let existing_conv = conversations::Entity::find()
            .filter(conversations::Column::ConvType.eq(CONV_TYPE_DIRECT))
            .filter(
                Condition::any()
                    .add(
                        conversations::Column::ConvId.in_subquery(
                            sea_query::Query::select()
                                .column(conv_members::Column::ConvId)
                                .from(conv_members::Entity)
                                .and_where(conv_members::Column::UserId.eq(user_id))
                                .to_owned()
                        )
                    )
            )
            .filter(
                Condition::any()
                     .add(
                        conversations::Column::ConvId.in_subquery(
                            sea_query::Query::select()
                                .column(conv_members::Column::ConvId)
                                .from(conv_members::Entity)
                                .and_where(conv_members::Column::UserId.eq(req.friend_id))
                                .to_owned()
                        )
                    )
            )
            .one(db)
            .await
            .map_err(AppError::from)?;

        if let Some(conv) = existing_conv {
             return Ok(ConversationResponse {
                id: conv.conv_id,
                friend_id: req.friend_id,
                created_at: conv.created_at.to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(), // Approximate
            });
        }

        // 2. Create new conversation
        let txn = db.begin().await.map_err(AppError::from)?;

        let conv_id = Uuid::new_v4();
        let now = Utc::now().into();

        let new_conv = conversations::ActiveModel {
            conv_id: Set(conv_id),
            conv_type: Set(CONV_TYPE_DIRECT),
            created_at: Set(now),
            creator_id: Set(Some(user_id)),
            metadata: Set(serde_json::json!({})),
            ..Default::default()
        };

        let inserted_conv = new_conv.insert(&txn).await.map_err(AppError::from)?;

        // Add user
        let member1 = conv_members::ActiveModel {
            conv_id: Set(conv_id),
            user_id: Set(user_id),
            role: Set(1), // Member
            joined_at: Set(now),
             ..Default::default()
        };
        member1.insert(&txn).await.map_err(AppError::from)?;

        // Add friend
        let member2 = conv_members::ActiveModel {
            conv_id: Set(conv_id),
            user_id: Set(req.friend_id),
            role: Set(1), // Member
            joined_at: Set(now),
             ..Default::default()
        };
        member2.insert(&txn).await.map_err(AppError::from)?;

        txn.commit().await.map_err(AppError::from)?;

        Ok(ConversationResponse {
            id: inserted_conv.conv_id,
            friend_id: req.friend_id,
            created_at: inserted_conv.created_at.to_rfc3339(),
            updated_at: inserted_conv.created_at.to_rfc3339(),
        })
    }
}
