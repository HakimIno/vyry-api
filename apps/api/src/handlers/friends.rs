use actix_web::{get, post, web, HttpResponse, Responder};
use crate::extractors::AuthUser;
use application::friends::{
    add_friend::AddFriendUseCase,
    accept_friend::AcceptFriendUseCase,
    list_friends::ListFriendsUseCase,
    search_user::SearchUserUseCase,
    list_pending::ListPendingRequestsUseCase,
    dtos::{AddFriendRequest, AcceptFriendRequest},
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use serde::Deserialize;
use crate::handlers::error_handler::HttpAppError;
use application::AppError;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[derive(Deserialize)]
pub struct FriendRequestInput {
    friend_id: Uuid,
}

#[derive(Deserialize)]
pub struct AcceptRequestInput {
    requester_id: Uuid,
    accept: bool,
}

#[post("/friends/request")]
pub async fn send_friend_request(
    user: AuthUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<FriendRequestInput>,
) -> Result<impl Responder, HttpAppError> {
    let user_id = Uuid::parse_str(&user.sub).map_err(|_| AppError::Authentication("Invalid user ID".to_string()))?;

    let req = AddFriendRequest {
        user_id, // requester
        friend_id: body.friend_id,
    };

    AddFriendUseCase::execute(&db, req).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Friend request sent"})))
}

#[post("/friends/accept")]
pub async fn accept_friend_request(
    user: AuthUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<AcceptRequestInput>,
) -> Result<impl Responder, HttpAppError> {
    let user_id = Uuid::parse_str(&user.sub).map_err(|_| AppError::Authentication("Invalid user ID".to_string()))?;

    let req = AcceptFriendRequest {
        user_id, // acceptor
        requester_id: body.requester_id,
        accept: body.accept,
    };

    AcceptFriendUseCase::execute(&db, req).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Friend request processed"})))
}

#[get("/friends")]
pub async fn list_friends(
    user: AuthUser,
    db: web::Data<DatabaseConnection>,
) -> Result<impl Responder, HttpAppError> {
    let user_id = Uuid::parse_str(&user.sub).map_err(|_| AppError::Authentication("Invalid user ID".to_string()))?;

    let friends = ListFriendsUseCase::execute(&db, user_id).await?;
    Ok(HttpResponse::Ok().json(friends))
}

#[get("/users/search")]
pub async fn search_users( // Auth required
    _user: AuthUser, 
    db: web::Data<DatabaseConnection>,
    query: web::Query<SearchQuery>,
) -> Result<impl Responder, HttpAppError> {
    match SearchUserUseCase::execute(&db, query.q.clone()).await? {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Err(AppError::NotFound("User not found".to_string()).into()),
    }
}
#[get("/friends/requests")]
pub async fn list_pending_requests(
    user: AuthUser,
    db: web::Data<DatabaseConnection>,
) -> Result<impl Responder, HttpAppError> {
    let user_id = Uuid::parse_str(&user.sub).map_err(|_| AppError::Authentication("Invalid user ID".to_string()))?;

    let requests = ListPendingRequestsUseCase::execute(&db, user_id).await?;
    Ok(HttpResponse::Ok().json(requests))
}

