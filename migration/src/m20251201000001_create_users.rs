use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::UserId).uuid().not_null().primary_key().extra("DEFAULT gen_random_uuid()"))
                    .col(ColumnDef::new(Users::PhoneNumber).string_len(20).not_null().unique_key())
                    .col(ColumnDef::new(Users::PhoneNumberHash).binary().not_null().unique_key())
                    .col(ColumnDef::new(Users::Username).string_len(50).unique_key())
                    .col(ColumnDef::new(Users::DisplayName).text())
                    .col(ColumnDef::new(Users::Bio).text())
                    .col(ColumnDef::new(Users::ProfilePicture).text())
                    .col(ColumnDef::new(Users::LastSeenAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Users::IsOnline).boolean().default(false))
                    .col(ColumnDef::new(Users::IsDeleted).boolean().default(false))
                    .col(ColumnDef::new(Users::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
    PhoneNumber,
    PhoneNumberHash,
    Username,
    DisplayName,
    Bio,
    ProfilePicture,
    LastSeenAt,
    IsOnline,
    IsDeleted,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}
