use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ConvMembers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ConvMembers::ConvId).uuid().not_null())
                    .col(ColumnDef::new(ConvMembers::UserId).uuid().not_null())
                    .col(ColumnDef::new(ConvMembers::Role).small_integer().default(0))
                    .col(
                        ColumnDef::new(ConvMembers::JoinedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(ConvMembers::LeftAt).timestamp_with_time_zone())
                    .primary_key(
                        Index::create()
                            .col(ConvMembers::ConvId)
                            .col(ConvMembers::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conv_members_conv_id")
                            .from(ConvMembers::Table, ConvMembers::ConvId)
                            .to(Conversations::Table, Conversations::ConvId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conv_members_user_id")
                            .from(ConvMembers::Table, ConvMembers::UserId)
                            .to(Users::Table, Users::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ConvMembers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ConvMembers {
    Table,
    ConvId,
    UserId,
    Role,
    JoinedAt,
    LeftAt,
}

#[derive(DeriveIden)]
enum Conversations {
    Table,
    ConvId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
}
