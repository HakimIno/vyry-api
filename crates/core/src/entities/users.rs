use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    #[sea_orm(unique)]
    pub phone_number: String,
    #[sea_orm(unique)]
    pub phone_number_hash: Vec<u8>,
    #[sea_orm(unique)]
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub profile_picture: Option<String>,
    pub background_image: Option<String>,
    pub last_seen_at: Option<DateTimeWithTimeZone>,
    pub is_online: bool,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    // PIN/2FA fields
    pub pin_hash: Option<String>,
    pub registration_lock: bool,
    pub registration_lock_expires_at: Option<DateTimeWithTimeZone>,
    pub pin_set_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::devices::Entity")]
    Devices,
    #[sea_orm(has_many = "super::conv_members::Entity")]
    ConvMembers,
    #[sea_orm(has_many = "super::messages::Entity")]
    Messages,
}

impl Related<super::devices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Devices.def()
    }
}

impl Related<super::conv_members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ConvMembers.def()
    }
}

impl Related<super::messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
