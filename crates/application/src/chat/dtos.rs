use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub sender_id: Uuid,
    pub sender_device_id: i64,
    pub recipient_id: Uuid,
    pub recipient_device_id: i64,
    pub conversation_id: Uuid,
    pub client_message_id: Uuid,
    pub content: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncMessageDto {
    pub message_id: i64,
    pub conversation_id: Uuid,
    pub client_message_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub sender_device_id: i64,
    pub content: Vec<u8>,
    pub sent_at: i64,
}
