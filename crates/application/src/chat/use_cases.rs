use super::dtos::SendMessageRequest;
use core::entities::{message_deliveries, messages};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use chrono::Utc;

pub struct SendMessageUseCase;

impl SendMessageUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        req: SendMessageRequest,
    ) -> Result<(), String> {
        let txn = db.begin().await.map_err(|e| e.to_string())?;

        // 1. Check if message exists (deduplication)
        let message = messages::Entity::find()
            .filter(messages::Column::ClientMessageId.eq(req.client_message_id))
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?;

        let message_id = if let Some(msg) = message {
            msg.message_id
        } else {
            // Create new message
            let new_msg = messages::ActiveModel {
                conv_id: Set(req.conversation_id),
                client_message_id: Set(Some(req.client_message_id)),
                sender_user_id: Set(req.sender_id),
                sender_device_id: Set(req.sender_device_id),
                message_type: Set(req.message_type),
                content: Set("".to_string()), // Placeholder for sender's copy
                iv: Set(req.iv),
                attachment_url: Set(req.attachment_url),
                thumbnail_url: Set(req.thumbnail_url),
                reply_to_message_id: Set(req.reply_to_message_id),
                sent_at: Set(Utc::now().into()),
                ..Default::default()
            };
            let inserted_msg = new_msg.insert(&txn).await.map_err(|e| e.to_string())?;
            inserted_msg.message_id
        };

        // 2. Insert Delivery
        // Check if delivery already exists
        let delivery_exists = message_deliveries::Entity::find()
            .filter(message_deliveries::Column::MessageId.eq(message_id))
            .filter(message_deliveries::Column::DeviceId.eq(req.recipient_device_id))
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?
            .is_some();

        if !delivery_exists {
            let delivery = message_deliveries::ActiveModel {
                message_id: Set(message_id),
                device_id: Set(req.recipient_device_id),
                content: Set(Some(req.content)),
                ..Default::default()
            };
            delivery.insert(&txn).await.map_err(|e| e.to_string())?;
        }

        txn.commit().await.map_err(|e| e.to_string())?;

        Ok(())
    }
}
