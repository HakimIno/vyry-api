use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OtpVerifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OtpVerifications::PhoneNumber)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OtpVerifications::OtpCode)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OtpVerifications::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OtpVerifications::AttemptCount)
                            .integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(OtpVerifications::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OtpVerifications::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OtpVerifications {
    Table,
    PhoneNumber,
    OtpCode,
    ExpiresAt,
    AttemptCount,
    CreatedAt,
}
