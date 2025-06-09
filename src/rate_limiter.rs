use redis::{RedisError, aio::ConnectionManager, Script};

pub struct RateLimiter {
    conn_manager: ConnectionManager,
    script: Script,
}

impl RateLimiter {
    pub async fn new(addr: &'static str) -> Result<Self, RedisError> {
        let client = redis::Client::open(addr)?;
        let conn_manager = ConnectionManager::new(client).await?;
        let script = Script::new(include_str!("gcra.lua"));
        info!("Starting with gcra.lua: {script:?}");
        Ok(Self {
            conn_manager,
            script,
        })
    }

    pub async fn check_and_incr(&self) -> anyhow::Result<bool> {
        // ConnectionManager is cheap to clone
        let mut manager = self.conn_manager.clone();
        let res: i32 = self.script
            .key("test_key")
            .invoke_async(&mut manager).await?;
        Ok(res % 2 == 0)
    }
}
