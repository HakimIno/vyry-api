use super::dtos::SyncMessageDto;
use core::entities::{message_deliveries, messages};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

pub struct SyncMessagesUseCase;

impl SyncMessagesUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        _user_id: Uuid,
        device_id: i64,
        last_message_id: Option<i64>,
    ) -> Result<Vec<SyncMessageDto>, String> {
        // Query message_deliveries joined with messages
        // Where device_id = device_id AND delivered_at IS NULL
        // AND message_id > last_message_id (if provided)
        
        let mut query = message_deliveries::Entity::find()
            .filter(message_deliveries::Column::DeviceId.eq(device_id))
            .filter(message_deliveries::Column::DeliveredAt.is_null());

        if let Some(last_id) = last_message_id {
            query = query.filter(message_deliveries::Column::MessageId.gt(last_id));
        }

        let deliveries = query
            .find_also_related(messages::Entity)
            .all(db)
            .await
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();

        for (delivery, message) in deliveries {
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
