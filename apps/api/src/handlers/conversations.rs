use actix_web::{get, post, web, HttpResponse, Responder};
use application::chat::{dtos::CreateDirectConversationRequest, create_conversation::CreateDirectConversationUseCase};
use sea_orm::DatabaseConnection;
use crate::extractors::AuthUser;
use uuid::Uuid;
use crate::handlers::error_handler::HttpAppError;
use application::AppError;

#[post("/conversations/direct")]
pub async fn create_direct_conversation(
    db: web::Data<DatabaseConnection>,
    auth: AuthUser,
    body: web::Json<CreateDirectConversationRequest>,
) -> Result<impl Responder, HttpAppError> {
    let user_id = Uuid::parse_str(&auth.sub).unwrap_or_default(); // Should be validated by middleware
    if user_id == Uuid::default() {
         return Err(AppError::Authentication("Invalid user ID".to_string()).into());
    }
    
    let response = CreateDirectConversationUseCase::execute(db.get_ref(), user_id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[derive(serde::Deserialize)]
pub struct ListMessagesQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[get("/conversations/{conversation_id}/messages")]
pub async fn get_conversation_messages(
    db: web::Data<DatabaseConnection>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    query: web::Query<ListMessagesQuery>,
) -> Result<impl Responder, HttpAppError> {
    let _user_id = Uuid::parse_str(&auth.sub).unwrap_or_default();
    let device_id = auth.device_id;
    let conversation_id = path.into_inner();
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let messages = application::chat::list_messages::ListMessagesUseCase::execute(
        db.get_ref(),
        conversation_id,
        device_id,
        limit,
        offset,
    ).await?;

    Ok(HttpResponse::Ok().json(messages))
}
