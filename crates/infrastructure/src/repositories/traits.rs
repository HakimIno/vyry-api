// Trait definitions for repository pattern
// This allows easy database switching
// Note: Using fully qualified async_trait to avoid conflict with crate::core

use uuid::Uuid;

// Example trait for user repository
// In production, you would define traits for each entity type

#[::async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<core::entities::users::Model>>;
    async fn find_by_phone_number(&self, phone_number: &str) -> anyhow::Result<Option<core::entities::users::Model>>;
    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<core::entities::users::Model>>;
    async fn create(&self, user: core::entities::users::ActiveModel) -> anyhow::Result<core::entities::users::Model>;
    async fn update(&self, user: core::entities::users::ActiveModel) -> anyhow::Result<core::entities::users::Model>;
}

#[::async_trait::async_trait]
pub trait MessageRepository: Send + Sync {
    // Message operations would go here
    // This will be implemented by ScyllaDB repository in the future
}

#[::async_trait::async_trait]
pub trait ConversationRepository: Send + Sync {
    // Conversation operations would go here
}

#[::async_trait::async_trait]
pub trait SignalRepository: Send + Sync {
    // Fetch a prekey bundle for a specific device
    async fn get_prekey_bundle(&self, device_id: i64) -> anyhow::Result<Option<core::signal::wrapper::PreKeyBundle>>;
    
    // Store one-time prekeys for a device
    async fn store_one_time_prekeys(&self, device_id: i64, keys: Vec<core::signal::wrapper::PublicPreKey>) -> anyhow::Result<()>;
    
    // Count remaining prekeys for a device
    async fn count_one_time_prekeys(&self, device_id: i64) -> anyhow::Result<u64>;
}
