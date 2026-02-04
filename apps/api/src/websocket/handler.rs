use super::connection::{ConnectionManager, WsConnection};
use crate::config::Config;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use application::auth::dtos::Claims;
use futures::StreamExt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct WsQuery {
    token: String,
}

use application::chat::{
    dtos::{SendMessageRequest, DeliveryStatusType},
    use_cases::SendMessageUseCase,
    update_status::UpdateDeliveryStatusUseCase,
};
use sea_orm::DatabaseConnection;

#[get("/ws/")]
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<ConnectionManager>,
    config: web::Data<Config>,
    db: web::Data<DatabaseConnection>,
    query: web::Query<WsQuery>,
) -> Result<HttpResponse, Error> {
    // Validate JWT
    let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
    let validation = Validation::default();
    
    let claims = match decode::<Claims>(&query.token, &decoding_key, &validation) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            tracing::error!("Invalid WebSocket token: {}", e);
            return Ok(HttpResponse::Unauthorized().finish());
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(uid) => uid,
        Err(_) => return Ok(HttpResponse::Unauthorized().finish()),
    };
    let device_id = claims.device_id;

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let conn_id = Uuid::new_v4();
    let ws_conn = WsConnection {
        user_id,
        device_id,
        conn_id,
        session: session.clone(),
    };

    manager.add_connection(ws_conn.clone()).await;
    tracing::info!("User {} Device {} connected (Conn ID: {})", user_id, device_id, conn_id);

    let db = db.get_ref().clone();

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    tracing::debug!("Received text message: {}", text);
                    // Parse message
                    match serde_json::from_str::<super::messages::WsMessage>(&text) {
                        Ok(ws_msg) => {
                            match ws_msg {
                                super::messages::WsMessage::SignalMessage {
                                    conversation_id,
                                    client_message_id,
                                    recipient_id,
                                    recipient_device_id,
                                    content,
                                    iv,
                                    message_type,
                                    attachment_url,
                                    thumbnail_url,
                                    reply_to_message_id,
                                    ..
                                } => {
                                    tracing::info!("Routing SignalMessage to User {} Device {}", recipient_id, recipient_device_id);
                                    
                                    // Persist message to DB (Store & Forward)
                                    let req = SendMessageRequest {
                                        sender_id: user_id,
                                        sender_device_id: device_id,
                                        recipient_id,
                                        recipient_device_id,
                                        conversation_id,
                                        client_message_id,
                                        content: content.clone(),
                                        iv: iv.clone(),
                                        message_type,
                                        attachment_url: attachment_url.clone(),
                                        thumbnail_url: thumbnail_url.clone(),
                                        reply_to_message_id,
                                    };

                                    if let Err(e) = SendMessageUseCase::execute(&db, req).await {
                                        tracing::error!("Failed to persist message: {}", e);
                                        // TODO: Send error back to client?
                                    }

                                    // Forward to specific device
                                    if let Some(mut target_conn) = manager.get_device_connection(&recipient_id, recipient_device_id).await {
                                        let outbound = super::messages::WsMessage::SignalMessage {
                                            conversation_id,
                                            client_message_id,
                                            sender_id: Some(user_id),
                                            sender_device_id: Some(device_id),
                                            recipient_id,
                                            recipient_device_id,
                                            content,
                                            iv,
                                            message_type,
                                            attachment_url,
                                            thumbnail_url,
                                            reply_to_message_id,
                                        };
                                        if let Ok(json) = serde_json::to_string(&outbound) {
                                            let _ = target_conn.session.text(json).await;
                                        }
                                    } else {
                                        tracing::warn!("Recipient {} device {} not online", recipient_id, recipient_device_id);
                                        // Message is already stored in DB, so it will be fetched on sync (Phase 4 task)
                                    }
                                }
                                super::messages::WsMessage::Ack { message_id } => {
                                    tracing::info!("Received Ack for message {}", message_id);
                                    if let Err(e) = UpdateDeliveryStatusUseCase::execute(&db, message_id, device_id, DeliveryStatusType::Delivered).await {
                                        tracing::error!("Failed to update delivery status: {}", e);
                                    }
                                }
                                super::messages::WsMessage::SyncRequest { last_message_id } => {
                                    tracing::info!("Received SyncRequest from User {} Device {}", user_id, device_id);
                                    match application::chat::sync_messages::SyncMessagesUseCase::execute(&db, user_id, device_id, last_message_id).await {
                                        Ok(messages) => {
                                            let response = super::messages::WsMessage::SyncResponse { messages };
                                            if let Ok(json) = serde_json::to_string(&response) {
                                                let _ = session.text(json).await;
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to sync messages: {}", e);
                                        }
                                    }
                                }
                                super::messages::WsMessage::SdpOffer { recipient_id, recipient_device_id, sdp } => {
                                    tracing::info!("Routing SdpOffer to User {} Device {}", recipient_id, recipient_device_id);
                                    if let Some(mut target_conn) = manager.get_device_connection(&recipient_id, recipient_device_id).await {
                                        let outbound = super::messages::WsMessage::SdpOffer {
                                            recipient_id: user_id, // From sender
                                            recipient_device_id: device_id,
                                            sdp,
                                        };
                                        if let Ok(json) = serde_json::to_string(&outbound) {
                                            let _ = target_conn.session.text(json).await;
                                        }
                                    }
                                }
                                super::messages::WsMessage::SdpAnswer { recipient_id, recipient_device_id, sdp } => {
                                    tracing::info!("Routing SdpAnswer to User {} Device {}", recipient_id, recipient_device_id);
                                    if let Some(mut target_conn) = manager.get_device_connection(&recipient_id, recipient_device_id).await {
                                        let outbound = super::messages::WsMessage::SdpAnswer {
                                            recipient_id: user_id,
                                            recipient_device_id: device_id,
                                            sdp,
                                        };
                                        if let Ok(json) = serde_json::to_string(&outbound) {
                                            let _ = target_conn.session.text(json).await;
                                        }
                                    }
                                }
                                super::messages::WsMessage::IceCandidate { recipient_id, recipient_device_id, candidate } => {
                                    tracing::info!("Routing IceCandidate to User {} Device {}", recipient_id, recipient_device_id);
                                    if let Some(mut target_conn) = manager.get_device_connection(&recipient_id, recipient_device_id).await {
                                        let outbound = super::messages::WsMessage::IceCandidate {
                                            recipient_id: user_id,
                                            recipient_device_id: device_id,
                                            candidate,
                                        };
                                        if let Ok(json) = serde_json::to_string(&outbound) {
                                            let _ = target_conn.session.text(json).await;
                                        }
                                    }
                                }
                                super::messages::WsMessage::DeliveryStatus { message_id, conversation_id, sender_id, status } => {
                                    tracing::info!("Received DeliveryStatus for msg {} from User {} Device {}", message_id, user_id, device_id);
                                    
                                    // 1. Update DB
                                    // Map API status to Application status
                                    let app_status = match status {
                                        super::messages::DeliveryStatusType::Delivered => application::chat::dtos::DeliveryStatusType::Delivered,
                                        super::messages::DeliveryStatusType::Read => application::chat::dtos::DeliveryStatusType::Read,
                                    };

                                    if let Err(e) = application::chat::update_status::UpdateDeliveryStatusUseCase::execute(&db, message_id, device_id, app_status).await {
                                        tracing::error!("Failed to update delivery status: {}", e);
                                    }

                                    // 2. Forward to original sender (if online)
                                    // We need to find all connections of the sender_id
                                    let sender_conns = manager.get_user_connections(&sender_id).await;
                                    for mut conn in sender_conns {
                                        let outbound = super::messages::WsMessage::DeliveryStatus {
                                            message_id,
                                            conversation_id,
                                            sender_id, // Echo back? Or maybe recipient_id? The client needs to know WHO read it.
                                            // Actually, the message structure might need 'recipient_id' (who read it) for the sender to know.
                                            // But 'user_id' (from context) IS the one who read it.
                                            // Let's assume the client uses the context of who sent this status update.
                                            // But wait, WsMessage::DeliveryStatus definition:
                                            // sender_id: Uuid, // The original sender who should receive this update
                                            // We should probably include 'updated_by' or similar if it's a group, but for 1-on-1, the sender knows it's the other person.
                                            status,
                                        };
                                        if let Ok(json) = serde_json::to_string(&outbound) {
                                            let _ = conn.session.text(json).await;
                                        }
                                    }
                                }
                                super::messages::WsMessage::Typing { conversation_id, recipient_id, is_typing } => {
                                    // Forward to recipient(s)
                                    // For 1-on-1, find recipient connections
                                    let recipient_conns = manager.get_user_connections(&recipient_id).await;
                                    for mut conn in recipient_conns {
                                        let outbound = super::messages::WsMessage::Typing {
                                            conversation_id,
                                            recipient_id: user_id, // From the perspective of the receiver, the 'recipient' of the typing event is the one TYPING.
                                            // Wait, the struct field is 'recipient_id'. 
                                            // In the outbound message, we should probably put the 'typer_id'.
                                            // Let's reuse the field but interpret it as 'who is typing' when receiving?
                                            // Or better, change the struct to have 'user_id' or 'sender_id'.
                                            // For now, let's assume the client handles it. 
                                            // Let's send the typer's ID in the 'recipient_id' slot? No that's confusing.
                                            // Let's just forward it as is, but the client needs to know WHO is typing.
                                            // The 'recipient_id' in the struct is the TARGET.
                                            // We should probably add 'sender_id' to the Typing struct or rely on the client knowing the peer.
                                            // Let's modify the struct in the next step if needed, but for now let's assume 1-on-1 context.
                                            // Actually, let's just forward it. The client might need to know who sent it.
                                            // Let's hack it: Put the sender's ID in 'recipient_id' for the outbound message?
                                            // No, let's just send it. The client receiving it knows it came from the peer in that conversation.
                                            is_typing,
                                        };
                                        if let Ok(json) = serde_json::to_string(&outbound) {
                                            let _ = conn.session.text(json).await;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse message: {}", e);
                        }
                    }
                }
                Message::Binary(bin) => {
                    tracing::debug!("Received binary message: {} bytes", bin.len());
                    // TODO: Support binary protocol (e.g. MessagePack)
                }
                Message::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }
                Message::Close(reason) => {
                    tracing::info!("WebSocket closed: {:?}", reason);
                    break;
                }
                _ => {}
            }
        }

        manager.remove_connection(&conn_id).await;
        tracing::info!("Connection {} closed", conn_id);
    });

    Ok(response)
}
