use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Conversations::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Conversations::ConvId).uuid().not_null().primary_key().extra("DEFAULT gen_random_uuid()"))
                    .col(ColumnDef::new(Conversations::ConvType).small_integer().not_null())
                    .col(ColumnDef::new(Conversations::Name).text())
                    .col(ColumnDef::new(Conversations::Avatar).text())
                    .col(ColumnDef::new(Conversations::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Conversations::CreatorId).uuid())
                    .col(ColumnDef::new(Conversations::Metadata).json_binary())
                    .check(Expr::col(Conversations::ConvType).between(1, 2))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversations_creator_id")
                            .from(Conversations::Table, Conversations::CreatorId)
                            .to(Users::Table, Users::UserId)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Conversations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Conversations {
    Table,
    ConvId,
    ConvType,
    Name,
    Avatar,
    CreatedAt,
    CreatorId,
    Metadata,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
}
