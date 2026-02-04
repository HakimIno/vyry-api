use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddFriendRequest {
    pub user_id: Uuid,       // The requester
    pub friend_id: Uuid,     // The target
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptFriendRequest {
    pub user_id: Uuid,       // The acceptor (User B)
    pub requester_id: Uuid,  // The original requester (User A)
    pub accept: bool,        // True = Accept, False = Reject (Delete request)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendDto {
    pub user_id: Uuid,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub profile_picture: Option<String>,
    pub status: i16,         // 0=Pending, 1=Accepted, 2=Blocked
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchUserResponse {
    pub user_id: Uuid,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub profile_picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetUsernameRequest {
    pub user_id: Uuid,
    pub username: String,
}
