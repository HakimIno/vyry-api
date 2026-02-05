use super::dtos::FriendDto;
use core::entities::{friends, users};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, RelationTrait, JoinType, FromQueryResult, prelude::DateTimeWithTimeZone
};
use uuid::Uuid;

pub struct ListPendingRequestsUseCase;

#[derive(FromQueryResult)]
struct RequestData {
    user_id: Uuid,
    username: Option<String>,
    display_name: Option<String>,
    profile_picture: Option<String>,
    status: i16,
    created_at: DateTimeWithTimeZone,
}

impl ListPendingRequestsUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<FriendDto>, String> {
        // Query friends where friend_id = user_id (Me) AND status = 0 (Pending)
        // Join with User via Users1 (friends.user_id -> users.user_id) to get Requester
        
        let requests = friends::Entity::find()
            .filter(
                friends::Column::FriendId.eq(user_id)
                    .and(friends::Column::Status.eq(0)) 
            )
            .join(JoinType::InnerJoin, friends::Relation::Users1.def())
            .select_only()
            .column(users::Column::UserId)
            .column(users::Column::Username)
            .column(users::Column::DisplayName)
            .column(users::Column::ProfilePicture)
            .column(friends::Column::Status)
            .column(friends::Column::CreatedAt)
            .into_model::<RequestData>()
            .all(db)
            .await
            .map_err(|e| e.to_string())?;

        let requests_list = requests.into_iter().map(|r| FriendDto {
            user_id: r.user_id,
            username: r.username,
            display_name: r.display_name,
            profile_picture: r.profile_picture,
            status: r.status,
            created_at: r.created_at.timestamp(),
        }).collect();

        Ok(requests_list)
    }
}
