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

use application::chat::{dtos::SendMessageRequest, use_cases::SendMessageUseCase};
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
                                super::messages::WsMessage::SignalMessage { conversation_id, client_message_id, recipient_id, recipient_device_id, content } => {
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
                                            recipient_id, // In outbound, this might be sender_id? Or client knows context.
                                            recipient_device_id,
                                            content,
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
                                    // TODO: Update message status in DB
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
