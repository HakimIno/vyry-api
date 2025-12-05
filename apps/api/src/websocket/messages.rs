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
        recipient_id: Uuid,
        recipient_device_id: i64,
        content: Vec<u8>, // Encrypted blob
    },
    /// Acknowledge receipt of a message
    Ack {
        message_id: String,
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
    /// Error message from server
    Error {
        code: String,
        message: String,
    },
}
