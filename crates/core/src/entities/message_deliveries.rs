use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "message_deliveries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub message_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_id: i64,
    pub delivered_at: Option<DateTimeWithTimeZone>,
    pub read_at: Option<DateTimeWithTimeZone>,
    pub content: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::messages::Entity",
        from = "Column::MessageId",
        to = "super::messages::Column::MessageId",
        on_delete = "Cascade"
    )]
    Messages,
    #[sea_orm(
        belongs_to = "super::devices::Entity",
        from = "Column::DeviceId",
        to = "super::devices::Column::DeviceId",
        on_delete = "Cascade"
    )]
    Devices,
}

impl Related<super::messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl Related<super::devices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Devices.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
