use actix_web::{get, post, web, HttpResponse, Responder};
use crate::extractors::AuthUser;
use application::friends::{
    add_friend::AddFriendUseCase,
    accept_friend::AcceptFriendUseCase,
    list_friends::ListFriendsUseCase,
    search_user::SearchUserUseCase,
    dtos::{AddFriendRequest, AcceptFriendRequest},
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use serde::Deserialize;

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
) -> impl Responder {
    let user_id = match Uuid::parse_str(&user.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid user ID in token"),
    };

    let req = AddFriendRequest {
        user_id, // requester
        friend_id: body.friend_id,
    };

    match AddFriendUseCase::execute(&db, req).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Friend request sent"})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

#[post("/friends/accept")]
pub async fn accept_friend_request(
    user: AuthUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<AcceptRequestInput>,
) -> impl Responder {
    let user_id = match Uuid::parse_str(&user.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid user ID in token"),
    };

    let req = AcceptFriendRequest {
        user_id, // acceptor
        requester_id: body.requester_id,
        accept: body.accept,
    };

    match AcceptFriendUseCase::execute(&db, req).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Friend request processed"})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

#[get("/friends")]
pub async fn list_friends(
    user: AuthUser,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let user_id = match Uuid::parse_str(&user.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid user ID in token"),
    };

    match ListFriendsUseCase::execute(&db, user_id).await {
        Ok(friends) => HttpResponse::Ok().json(friends),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

#[get("/users/search")]
pub async fn search_users( // Auth required
    _user: AuthUser, 
    db: web::Data<DatabaseConnection>,
    query: web::Query<SearchQuery>,
) -> impl Responder {
    match SearchUserUseCase::execute(&db, query.q.clone()).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}
