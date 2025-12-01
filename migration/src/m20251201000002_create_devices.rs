use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Devices::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Devices::DeviceId).big_integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Devices::UserId).uuid().not_null())
                    .col(ColumnDef::new(Devices::DeviceUuid).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Devices::DeviceName).text())
                    .col(ColumnDef::new(Devices::Platform).small_integer().not_null())
                    .col(ColumnDef::new(Devices::IdentityKeyPublic).binary().not_null())
                    .col(ColumnDef::new(Devices::RegistrationId).integer().not_null())
                    .col(ColumnDef::new(Devices::SignedPrekeyId).integer().not_null())
                    .col(ColumnDef::new(Devices::SignedPrekeyPublic).binary().not_null())
                    .col(ColumnDef::new(Devices::SignedPrekeySignature).binary().not_null())
                    .col(ColumnDef::new(Devices::LastSeenAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Devices::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_devices_user_id")
                            .from(Devices::Table, Devices::UserId)
                            .to(Users::Table, Users::UserId)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Devices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Devices {
    Table,
    DeviceId,
    UserId,
    DeviceUuid,
    DeviceName,
    Platform,
    IdentityKeyPublic,
    RegistrationId,
    SignedPrekeyId,
    SignedPrekeyPublic,
    SignedPrekeySignature,
    LastSeenAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
}
