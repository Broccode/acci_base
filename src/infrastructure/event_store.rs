use anyhow::Result;
use event_store::{EventStoreClient as EsClient, TypeName};
use serde::{Deserialize, Serialize};

use crate::infrastructure::config::EventStoreConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestEvent {}

impl TypeName for TestEvent {
    fn type_name(&self) -> String {
        "TestEvent".to_string()
    }
}

pub struct EventStoreClient {
    client: EsClient,
}

impl EventStoreClient {
    pub fn new(config: EventStoreConfig) -> Result<Self> {
        let client = EsClient::new(event_store::EventStoreConfig {
            connection_string: config.url,
            ..Default::default()
        })?;
        Ok(Self { client })
    }

    pub async fn check_connection(&self) -> Result<()> {
        // Check connection by reading from a test stream
        let _: Vec<event_store::Event<TestEvent>> = self.client.read_stream("$all", 0, 1).await?;
        Ok(())
    }
}
