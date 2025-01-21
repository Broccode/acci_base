use anyhow::Result;
use lapin::{Connection, ConnectionProperties};

use crate::infrastructure::config::RabbitMQConfig;

pub struct MessageBroker {
    connection: Connection,
}

impl MessageBroker {
    pub fn new(config: &RabbitMQConfig) -> Result<Self> {
        let connection = tokio::runtime::Handle::current().block_on(async {
            Connection::connect(&config.url, ConnectionProperties::default()).await
        })?;
        Ok(Self { connection })
    }

    pub async fn check_connection(&self) -> Result<()> {
        // Check if connection is still open
        if !self.connection.status().connected() {
            anyhow::bail!("RabbitMQ connection is not open");
        }
        Ok(())
    }
}
