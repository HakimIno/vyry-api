// Repository pattern for database abstraction
// This allows easy switching between different database implementations

pub mod traits;
pub mod postgres;
// pub mod scylladb; // Future: ScyllaDB implementation

// Note: We're not exporting everything yet to avoid breaking existing code
// Gradually migrate use cases to use repositories
