use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OneTimePrekeys::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(OneTimePrekeys::DeviceId).big_integer().not_null())
                    .col(ColumnDef::new(OneTimePrekeys::PrekeyId).integer().not_null())
                    .col(ColumnDef::new(OneTimePrekeys::PublicKey).binary().not_null())
                    .primary_key(Index::create().col(OneTimePrekeys::DeviceId).col(OneTimePrekeys::PrekeyId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_one_time_prekeys_device_id")
                            .from(OneTimePrekeys::Table, OneTimePrekeys::DeviceId)
                            .to(Devices::Table, Devices::DeviceId)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OneTimePrekeys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OneTimePrekeys {
    Table,
    DeviceId,
    PrekeyId,
    PublicKey,
}

#[derive(DeriveIden)]
enum Devices {
    Table,
    DeviceId,
}
