use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub message_id: i64,
    pub conv_id: Uuid,
    pub client_message_id: Option<Uuid>,
    pub sender_user_id: Uuid,
    pub sender_device_id: i64,
    pub message_type: i16,
    pub content: String,
    pub iv: Vec<u8>,
    pub attachment_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub sender_key_distribution: Option<Vec<u8>>,
    pub reply_to_message_id: Option<i64>,
    pub sent_at: DateTimeWithTimeZone,
    pub edited_at: Option<DateTimeWithTimeZone>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub extra: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::conversations::Entity",
        from = "Column::ConvId",
        to = "super::conversations::Column::ConvId"
    )]
    Conversations,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::SenderUserId",
        to = "super::users::Column::UserId"
    )]
    Users,
    #[sea_orm(has_many = "super::message_deliveries::Entity")]
    MessageDeliveries,
}

impl Related<super::conversations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversations.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::message_deliveries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MessageDeliveries.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
