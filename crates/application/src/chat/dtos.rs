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
    pub iv: Vec<u8>,
    pub message_type: i16,
    pub attachment_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub reply_to_message_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncMessageDto {
    pub message_id: i64,
    pub conversation_id: Uuid,
    pub client_message_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub sender_device_id: i64,
    pub content: Vec<u8>,
    pub iv: Vec<u8>,
    pub message_type: i16,
    pub attachment_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub reply_to_message_id: Option<i64>,
    pub sent_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum DeliveryStatusType {
    Delivered,
    Read,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDirectConversationRequest {
    pub friend_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub friend_id: Uuid,
    // Using String for ISO dates to match TS interface
    pub created_at: String,
    pub updated_at: String,
}
