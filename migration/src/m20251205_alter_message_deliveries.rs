use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MessageDeliveries::Table)
                    .add_column(ColumnDef::new(MessageDeliveries::Content).binary().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MessageDeliveries::Table)
                    .drop_column(MessageDeliveries::Content)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum MessageDeliveries {
    Table,
    Content,
}
