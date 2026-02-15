use application::chat::dtos::SyncMessageDto;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {
    /// Send a Signal Protocol encrypted message
    SignalMessage {
        conversation_id: Uuid,
        client_message_id: Uuid,
        sender_id: Option<Uuid>, // Optional for inbound (server sets it), Mandatory for outbound
        sender_device_id: Option<i64>,
        recipient_id: Uuid,
        recipient_device_id: i64,
        content: Vec<u8>, // Encrypted blob
        iv: Vec<u8>,
        message_type: i16,
        attachment_url: Option<String>,
        thumbnail_url: Option<String>,
        reply_to_message_id: Option<i64>,
    },
    /// Acknowledge receipt of a message
    Ack {
        message_id: i64,
    },
    /// Request to sync offline messages
    SyncRequest {
        last_message_id: Option<i64>,
    },
    /// Response with offline messages
    SyncResponse {
        messages: Vec<SyncMessageDto>,
    },
    /// WebRTC Signaling: SDP Offer
    SdpOffer {
        recipient_id: Uuid,
        recipient_device_id: i64,
        sdp: String,
    },
    /// WebRTC Signaling: SDP Answer
    SdpAnswer {
        recipient_id: Uuid,
        recipient_device_id: i64,
        sdp: String,
    },
    /// WebRTC Signaling: ICE Candidate
    IceCandidate {
        recipient_id: Uuid,
        recipient_device_id: i64,
        candidate: String,
    },
    /// Update message delivery status
    DeliveryStatus {
        message_id: i64,
        conversation_id: Uuid,
        sender_id: Uuid, // The original sender who should receive this update
        status: DeliveryStatusType,
    },
    /// Typing indicator
    Typing {
        conversation_id: Uuid,
        recipient_id: Uuid,
        is_typing: bool,
    },
    /// Error message from server
    Error {
        code: String,
        message: String,
    },
    /// Client heartbeat ping
    Ping {},
    /// Server heartbeat pong
    Pong {},
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum DeliveryStatusType {
    Delivered,
    Read,
}
