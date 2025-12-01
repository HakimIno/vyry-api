use redis::aio::MultiplexedConnection;
use serde::Serialize;

pub struct RedisClient {
    conn: MultiplexedConnection,
}

impl RedisClient {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }

    pub async fn publish<T: Serialize>(&mut self, channel: &str, message: &T) -> anyhow::Result<()> {
        let payload = serde_json::to_string(message)?;
        redis::cmd("PUBLISH")
            .arg(channel)
            .arg(payload)
            .query_async::<()>(&mut self.conn)
            .await?;
        Ok(())
    }

    pub async fn set_user_online(&mut self, user_id: &str, device_id: i64) -> anyhow::Result<()> {
        let key = format!("user:{}:online", user_id);
        redis::cmd("SADD")
            .arg(&key)
            .arg(device_id)
            .query_async::<()>(&mut self.conn)
            .await?;
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(3600)
            .query_async::<()>(&mut self.conn)
            .await?;
        Ok(())
    }

    pub async fn set_user_offline(&mut self, user_id: &str, device_id: i64) -> anyhow::Result<()> {
        let key = format!("user:{}:online", user_id);
        redis::cmd("SREM")
            .arg(&key)
            .arg(device_id)
            .query_async::<()>(&mut self.conn)
            .await?;
        Ok(())
    }
}
