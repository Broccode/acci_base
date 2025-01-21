pub mod client;
pub mod config;
pub mod events;

pub use client::{EventStoreClient, RecordedEvent};
pub use config::{EventStoreConfig, RetryPolicy};
pub use events::{DomainEvent, Event, EventCategory, EventMetadata, StreamName, TypeName};

use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamPosition(pub u64);

impl StreamPosition {
    pub const START: StreamPosition = StreamPosition(0);
}

#[derive(Debug, Clone)]
pub struct SubscribeToAllOptions {
    pub from_position: Option<StreamPosition>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionFilter {
    pub event_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WriteResult {
    pub position: u64,
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        message: String,
    }

    impl TypeName for TestEvent {
        fn type_name(&self) -> String {
            "TestEvent".to_string()
        }
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn test_event_store_integration() {
        let config = EventStoreConfig {
            connection_string: "http://localhost:2113".to_string(),
            ..Default::default()
        };

        let client = EventStoreClient::new(config).unwrap();
        let stream_name = format!("test-{}", Uuid::new_v4());

        let event = Event::new(
            TestEvent {
                message: "Test Event".to_string(),
            },
            1,
            Some(Uuid::new_v4()),
            None,
            Some(Uuid::new_v4()),
        );

        client
            .append_to_stream(&stream_name, vec![event.clone()])
            .await
            .unwrap();

        let events: Vec<Event<TestEvent>> = client.read_stream(&stream_name, 0, 10).await.unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data.message, "Test Event");
    }

    #[tokio::test]
    async fn test_append_and_read_events() -> anyhow::Result<()> {
        let config = EventStoreConfig::default();
        let client = EventStoreClient::new(config)?;

        let event = Event::new(
            TestEvent {
                message: "Hello".to_string(),
            },
            1,
            None,
            None,
            None,
        );

        let events = vec![event];
        client.append_to_stream("test-stream", events).await?;

        let read_events: Vec<Event<TestEvent>> = client.read_stream("test-stream", 0, 10).await?;
        assert_eq!(read_events.len(), 1);
        assert_eq!(read_events[0].data.message, "Hello");

        Ok(())
    }
}
