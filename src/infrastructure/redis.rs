use anyhow::Result;
use redis::Client;

use crate::infrastructure::config::RedisConfig;

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub fn new(config: &RedisConfig) -> Result<Self> {
        let client = Client::open(config.url.as_str())?;
        Ok(Self { client })
    }

    pub async fn ping(&self) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        redis::cmd("PING").query_async::<_, ()>(&mut conn).await?;
        Ok(())
    }
}
