use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Messages::MessageId).big_integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Messages::ConvId).uuid().not_null())
                    .col(ColumnDef::new(Messages::SenderUserId).uuid().not_null())
                    .col(ColumnDef::new(Messages::SenderDeviceId).big_integer().not_null())
                    .col(ColumnDef::new(Messages::MessageType).small_integer().not_null())
                    .col(ColumnDef::new(Messages::Content).text().not_null())
                    .col(ColumnDef::new(Messages::Iv).binary().not_null())
                    .col(ColumnDef::new(Messages::AttachmentUrl).text())
                    .col(ColumnDef::new(Messages::ThumbnailUrl).text())
                    .col(ColumnDef::new(Messages::SenderKeyDistribution).binary())
                    .col(ColumnDef::new(Messages::ReplyToMessageId).big_integer())
                    .col(ColumnDef::new(Messages::SentAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Messages::EditedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Messages::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Messages::ExpiresAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Messages::Extra).json_binary())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_messages_conv_id")
                            .from(Messages::Table, Messages::ConvId)
                            .to(Conversations::Table, Conversations::ConvId)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_messages_sender_user_id")
                            .from(Messages::Table, Messages::SenderUserId)
                            .to(Users::Table, Users::UserId)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_messages_reply_to_message_id")
                            .from(Messages::Table, Messages::ReplyToMessageId)
                            .to(Messages::Table, Messages::MessageId)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    MessageId,
    ConvId,
    SenderUserId,
    SenderDeviceId,
    MessageType,
    Content,
    Iv,
    AttachmentUrl,
    ThumbnailUrl,
    SenderKeyDistribution,
    ReplyToMessageId,
    SentAt,
    EditedAt,
    DeletedAt,
    ExpiresAt,
    Extra,
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
