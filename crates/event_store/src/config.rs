use anyhow::Result;
use serde::Deserialize;
use std::time::Duration;

use crate::client::EventStoreClient;

#[derive(Debug, Clone, Deserialize)]
pub struct EventStoreConfig {
    /// The connection string to the EventStore cluster
    pub connection_string: String,

    /// Maximum number of retry attempts for operations
    pub max_retries: u32,

    /// Delay between retry attempts in milliseconds
    pub retry_delay: u64,

    /// Maximum number of events to append in a single batch
    pub max_append_size: usize,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            connection_string: "http://localhost:2113".to_string(),
            max_retries: 3,
            retry_delay: 1000,
            max_append_size: 1000,
        }
    }
}

impl EventStoreConfig {
    pub fn create_client(&self) -> Result<EventStoreClient> {
        EventStoreClient::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub delay: Duration,
}

impl RetryPolicy {
    pub fn new(max_retries: u32, delay_ms: u64) -> Self {
        Self {
            max_retries,
            delay: Duration::from_millis(delay_ms),
        }
    }
}

impl EventStoreConfig {
    pub fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy::new(self.max_retries, self.retry_delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EventStoreConfig::default();
        assert_eq!(config.connection_string, "http://localhost:2113");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, 1000);
        assert_eq!(config.max_append_size, 1000);
    }

    #[test]
    fn test_create_client() -> Result<()> {
        let config = EventStoreConfig::default();
        let _client = config.create_client()?;
        Ok(())
    }
}
