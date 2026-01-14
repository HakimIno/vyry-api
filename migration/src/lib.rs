pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20251201000001_create_users;
mod m20251201000002_create_devices;
mod m20251201000003_create_one_time_prekeys;
mod m20251201000004_create_signal_sessions;
mod m20251201000005_create_conversations;
mod m20251201000006_create_conv_members;
mod m20251201000007_create_messages;
mod m20251201000008_create_message_deliveries;
mod m20251201000009_create_push_tokens;
mod m20251202000010_create_otp_verifications;
mod m20251205_alter_message_deliveries;
mod m20251205_add_client_message_id_to_messages;
mod m20251207000001_add_pin_to_users;
mod m20251207000002_add_device_type_to_devices;
mod m20251207000003_create_device_linking_sessions;
mod m20251207000004_add_background_image_to_users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20251201000001_create_users::Migration),
            Box::new(m20251201000002_create_devices::Migration),
            Box::new(m20251201000003_create_one_time_prekeys::Migration),
            Box::new(m20251201000004_create_signal_sessions::Migration),
            Box::new(m20251201000005_create_conversations::Migration),
            Box::new(m20251201000006_create_conv_members::Migration),
            Box::new(m20251201000007_create_messages::Migration),
            Box::new(m20251201000008_create_message_deliveries::Migration),
            Box::new(m20251201000009_create_push_tokens::Migration),
            Box::new(m20251202000010_create_otp_verifications::Migration),
            Box::new(m20251205_alter_message_deliveries::Migration),
            Box::new(m20251205_add_client_message_id_to_messages::Migration),
            Box::new(m20251207000001_add_pin_to_users::Migration),
            Box::new(m20251207000002_add_device_type_to_devices::Migration),
            Box::new(m20251207000003_create_device_linking_sessions::Migration),
            Box::new(m20251207000004_add_background_image_to_users::Migration),
        ]
    }
}
