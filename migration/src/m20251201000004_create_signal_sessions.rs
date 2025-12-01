use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SignalSessions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SignalSessions::DeviceId).big_integer().not_null())
                    .col(ColumnDef::new(SignalSessions::Address).text().not_null())
                    .col(ColumnDef::new(SignalSessions::SessionRecord).binary().not_null())
                    .col(ColumnDef::new(SignalSessions::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .primary_key(Index::create().col(SignalSessions::DeviceId).col(SignalSessions::Address))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_signal_sessions_device_id")
                            .from(SignalSessions::Table, SignalSessions::DeviceId)
                            .to(Devices::Table, Devices::DeviceId)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SignalSessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SignalSessions {
    Table,
    DeviceId,
    Address,
    SessionRecord,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Devices {
    Table,
    DeviceId,
}
