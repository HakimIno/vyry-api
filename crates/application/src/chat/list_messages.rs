use super::dtos::SyncMessageDto;
use vyry_core::entities::{message_deliveries, messages};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;
use crate::AppError;

pub struct ListMessagesUseCase;

impl ListMessagesUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        conversation_id: Uuid,
        device_id: i64,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<SyncMessageDto>, AppError> {
        // Query message_deliveries joined with messages
        // We need messages for the conversation_id and ordering by sent_at

        let results = message_deliveries::Entity::find()
            .filter(message_deliveries::Column::DeviceId.eq(device_id))
            .find_also_related(messages::Entity)
            .filter(messages::Column::ConvId.eq(conversation_id))
            .order_by_desc(messages::Column::SentAt)
            .limit(limit)
            .offset(offset)
            .all(db)
            .await
            .map_err(AppError::from)?;

        let mut result = Vec::new();

        for (delivery, message) in results {
            if let Some(msg) = message {
                if let Some(content) = delivery.content {
                    result.push(SyncMessageDto {
                        message_id: msg.message_id,
                        conversation_id: msg.conv_id,
                        client_message_id: msg.client_message_id,
                        sender_id: msg.sender_user_id,
                        sender_device_id: msg.sender_device_id,
                        content,
                        iv: msg.iv,
                        message_type: msg.message_type,
                        attachment_url: msg.attachment_url,
                        thumbnail_url: msg.thumbnail_url,
                        reply_to_message_id: msg.reply_to_message_id,
                        sent_at: msg.sent_at.timestamp(),
                    });
                }
            }
        }

        Ok(result)
    }
}
