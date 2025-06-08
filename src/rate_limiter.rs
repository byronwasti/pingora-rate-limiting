use redis::{AsyncCommands, RedisError, aio::ConnectionManager};

pub struct RateLimiter {
    conn_manager: ConnectionManager,
}

impl RateLimiter {
    pub async fn new(addr: &'static str) -> Result<Self, RedisError> {
        let client = redis::Client::open(addr)?;
        let conn_manager = ConnectionManager::new(client).await?;
        Ok(Self { conn_manager })
    }

    pub async fn check_and_incr(&self) -> anyhow::Result<bool> {
        // The ConnectionManager is cheap to clone
        let mut manager = self.conn_manager.clone();
        let res: i32 = manager.incr("test_key", 1).await?;
        Ok(res % 2 == 0)
    }
}
