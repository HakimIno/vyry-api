use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Friends::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Friends::UserId).uuid().not_null())
                    .col(ColumnDef::new(Friends::FriendId).uuid().not_null())
                    .col(
                        ColumnDef::new(Friends::Status)
                            .small_integer()
                            .not_null()
                            .comment("0=Pending, 1=Accepted, 2=Blocked"),
                    )
                    .col(
                        ColumnDef::new(Friends::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Friends::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(Friends::UserId)
                            .col(Friends::FriendId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_friends_user_id")
                            .from(Friends::Table, Friends::UserId)
                            .to(Users::Table, Users::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_friends_friend_id")
                            .from(Friends::Table, Friends::FriendId)
                            .to(Users::Table, Users::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for querying friends by status
        manager
            .create_index(
                Index::create()
                    .name("idx_friends_status")
                    .table(Friends::Table)
                    .col(Friends::UserId)
                    .col(Friends::Status)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Friends::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Friends {
    Table,
    UserId,
    FriendId,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
}
