pub use redis::aio::MultiplexedConnection;
pub use sea_orm::DatabaseConnection;

use anyhow::Result;

/// Database connections container
#[derive(Clone)]
pub struct DatabaseConnections {
    /// PostgreSQL connection for users, conversations, metadata
    pub postgres: DatabaseConnection,
    /// Redis connection for cache, pub/sub, rate limiting
    pub redis: MultiplexedConnection,
}

impl DatabaseConnections {
    pub async fn new(
        postgres_url: &str,
        redis_url: &str,
    ) -> Result<Self> {
        let postgres = init_postgres(postgres_url).await?;
        let redis = init_redis(redis_url).await?;
        
        Ok(Self {
            postgres,
            redis,
        })
    }
}

/// Initialize PostgreSQL connection
pub async fn init_postgres(database_url: &str) -> Result<DatabaseConnection> {
    let db = sea_orm::Database::connect(database_url).await?;
    tracing::info!("PostgreSQL connected successfully");
    Ok(db)
}

/// Initialize Redis connection
pub async fn init_redis(redis_url: &str) -> Result<MultiplexedConnection> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_multiplexed_tokio_connection().await?;
    tracing::info!("Redis connected successfully");
    Ok(conn)
}

/// Initialize database (backward compatibility)
/// Use DatabaseConnections::new() for new code
pub async fn init_database(database_url: &str) -> Result<DatabaseConnection> {
    init_postgres(database_url).await
}
