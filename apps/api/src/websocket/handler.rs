use super::connection::{ConnectionManager, WsConnection};
use crate::config::Config;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_ws::{Message, Session};
use application::auth::dtos::Claims;
use futures::StreamExt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

/// Serialize a WsMessage to MessagePack binary and send via WebSocket session.
use serde::Serialize;
use rmp_serde::Serializer;

/// Serialize a WsMessage to MessagePack binary and send via WebSocket session.
async fn send_msg(session: &mut Session, msg: &super::messages::WsMessage) {
    let mut buf = Vec::new();
    let mut serializer = Serializer::new(&mut buf).with_struct_map();

    match msg.serialize(&mut serializer) {
        Ok(_) => { 
            if let Err(e) = session.binary(buf).await {
                tracing::error!("Failed to send binary message to session: {}", e);
            } else {
                tracing::debug!("Successfully sent binary message");
            }
        }
        Err(e) => tracing::error!("Failed to serialize outbound message: {}", e),
    }
}

/// Parse a WsMessage from either MessagePack bytes or JSON string.
fn parse_ws_message(data: &[u8], is_binary: bool) -> Result<super::messages::WsMessage, String> {
    if is_binary {
        rmp_serde::from_slice(data).map_err(|e| format!("MessagePack parse error: {}", e))
    } else {
        serde_json::from_slice(data).map_err(|e| format!("JSON parse error: {}", e))
    }
}

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
                msg @ (Message::Binary(_) | Message::Text(_)) => {
                    let (data, is_binary): (&[u8], bool) = match &msg {
                        Message::Binary(bin) => (bin.as_ref(), true),
                        Message::Text(text) => (text.as_ref(), false),
                        _ => unreachable!(),
                    };
                    tracing::debug!("Received {} message: {} bytes", if is_binary { "binary" } else { "text" }, data.len());
                    match parse_ws_message(data, is_binary) {
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
                                        tracing::info!("Found active connection for User {} Device {}. Forwarding message...", recipient_id, recipient_device_id);
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
                                        send_msg(&mut target_conn.session, &outbound).await;
                                        tracing::info!("Message forwarded to session.");
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
                                            send_msg(&mut session, &response).await;
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
                                        send_msg(&mut target_conn.session, &outbound).await;
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
                                        send_msg(&mut target_conn.session, &outbound).await;
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
                                        send_msg(&mut target_conn.session, &outbound).await;
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
                                            sender_id,
                                            status,
                                        };
                                        send_msg(&mut conn.session, &outbound).await;
                                    }
                                }
                                super::messages::WsMessage::Typing { conversation_id, recipient_id, is_typing } => {
                                    // Forward to recipient(s)
                                    let recipient_conns = manager.get_user_connections(&recipient_id).await;
                                    for mut conn in recipient_conns {
                                        let outbound = super::messages::WsMessage::Typing {
                                            conversation_id,
                                            recipient_id: user_id,
                                            is_typing,
                                        };
                                        send_msg(&mut conn.session, &outbound).await;
                                    }
                                }
                                super::messages::WsMessage::Ping {} => {
                                    // Respond to application-level heartbeat with Pong
                                    let pong = super::messages::WsMessage::Pong {};
                                    send_msg(&mut session, &pong).await;
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse message: {}", e);
                        }
                    }
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
