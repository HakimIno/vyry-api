use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PushTokens::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PushTokens::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(PushTokens::DeviceId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PushTokens::Platform)
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PushTokens::Token).text().not_null())
                    .col(
                        ColumnDef::new(PushTokens::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(PushTokens::UserId)
                            .col(PushTokens::DeviceId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_push_tokens_user_id")
                            .from(PushTokens::Table, PushTokens::UserId)
                            .to(Users::Table, Users::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_push_tokens_device_id")
                            .from(PushTokens::Table, PushTokens::DeviceId)
                            .to(Devices::Table, Devices::DeviceId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PushTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PushTokens {
    Table,
    UserId,
    DeviceId,
    Platform,
    Token,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum Devices {
    Table,
    DeviceId,
}
