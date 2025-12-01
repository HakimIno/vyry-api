use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MessageDeliveries::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MessageDeliveries::MessageId).big_integer().not_null())
                    .col(ColumnDef::new(MessageDeliveries::DeviceId).big_integer().not_null())
                    .col(ColumnDef::new(MessageDeliveries::DeliveredAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(MessageDeliveries::ReadAt).timestamp_with_time_zone())
                    .primary_key(Index::create().col(MessageDeliveries::MessageId).col(MessageDeliveries::DeviceId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_deliveries_message_id")
                            .from(MessageDeliveries::Table, MessageDeliveries::MessageId)
                            .to(Messages::Table, Messages::MessageId)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_deliveries_device_id")
                            .from(MessageDeliveries::Table, MessageDeliveries::DeviceId)
                            .to(Devices::Table, Devices::DeviceId)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MessageDeliveries::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MessageDeliveries {
    Table,
    MessageId,
    DeviceId,
    DeliveredAt,
    ReadAt,
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    MessageId,
}

#[derive(DeriveIden)]
enum Devices {
    Table,
    DeviceId,
}
