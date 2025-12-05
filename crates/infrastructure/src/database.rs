pub use redis::aio::MultiplexedConnection;
pub use sea_orm::DatabaseConnection;

pub async fn init_database(database_url: &str) -> anyhow::Result<DatabaseConnection> {
    let db = sea_orm::Database::connect(database_url).await?;
    tracing::info!("Database connected successfully");
    Ok(db)
}

pub async fn init_redis(redis_url: &str) -> anyhow::Result<MultiplexedConnection> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_multiplexed_tokio_connection().await?;
    tracing::info!("Redis connected successfully");
    Ok(conn)
}
