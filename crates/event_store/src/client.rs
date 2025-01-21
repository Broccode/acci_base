use anyhow::Result;
use chrono::{DateTime, Utc};
use metrics::{counter, histogram};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::instrument;
use url::Url;
use uuid::Uuid;

use crate::config::EventStoreConfig;
use crate::events::{Event, EventData, TypeName};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    #[serde(rename = "eventId")]
    pub event_id: Uuid,
    #[serde(rename = "eventType")]
    pub event_type: String,
    pub data: Value,
    pub metadata: Value,
    pub created: DateTime<Utc>,
}

impl RecordedEvent {
    pub fn into_domain_event<T>(&self) -> Result<Event<T>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone + TypeName,
    {
        let data: T = serde_json::from_value(self.data.clone())?;
        Ok(Event::new(data, 1, None, None, Some(self.event_id)))
    }
}

pub struct EventStoreClient {
    http_client: HttpClient,
    base_url: Url,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadStreamResponse {
    pub entries: Vec<RecordedEvent>,
}

impl EventStoreClient {
    pub fn new(config: EventStoreConfig) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let base_url = Url::parse(&config.connection_string)?;

        Ok(Self {
            http_client,
            base_url,
        })
    }

    #[instrument(skip(self, events), fields(stream_name))]
    pub async fn append_to_stream<T>(&self, stream_name: &str, events: Vec<Event<T>>) -> Result<()>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone + TypeName,
    {
        let url = self.base_url.join(&format!("/streams/{}", stream_name))?;

        let events: Vec<EventData> = events
            .into_iter()
            .map(|e| e.to_event_data())
            .collect::<Result<_>>()?;

        let start = std::time::Instant::now();
        let _response = self
            .http_client
            .post(url)
            .json(&events)
            .send()
            .await?
            .error_for_status()?;

        histogram!(
            "eventstore.append.duration_ms",
            start.elapsed().as_millis() as f64
        );
        counter!("eventstore.append.success_total", 1);
        Ok(())
    }

    #[instrument(skip(self), fields(stream_name, start, count))]
    pub async fn read_stream<T>(
        &self,
        stream_name: &str,
        start: u64,
        count: u64,
    ) -> Result<Vec<Event<T>>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone + TypeName,
    {
        let url = self.base_url.join(&format!(
            "/streams/{}/{}?count={}",
            stream_name, start, count
        ))?;

        let start = std::time::Instant::now();
        let response = self.http_client.get(url).send().await?.error_for_status()?;

        let events: Vec<RecordedEvent> = response.json().await?;

        let domain_events = events
            .into_iter()
            .map(|e| e.into_domain_event())
            .collect::<Result<_>>()?;

        histogram!(
            "eventstore.read.duration_ms",
            start.elapsed().as_millis() as f64
        );
        counter!("eventstore.read.success_total", 1);
        Ok(domain_events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        message: String,
    }

    impl TypeName for TestEvent {
        fn type_name(&self) -> String {
            "TestEvent".to_string()
        }
    }

    #[tokio::test]
    async fn test_append_to_stream() -> Result<()> {
        let mock_server = MockServer::start().await;

        let config = EventStoreConfig {
            connection_string: mock_server.uri(),
            ..Default::default()
        };

        let client = EventStoreClient::new(config)?;
        let test_event = TestEvent {
            message: "Hello".to_string(),
        };
        let event = Event::new(test_event, 1, None, None, None);

        Mock::given(method("POST"))
            .and(path("/streams/test-stream"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        client.append_to_stream("test-stream", vec![event]).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_stream() -> Result<()> {
        let mock_server = MockServer::start().await;

        let config = EventStoreConfig {
            connection_string: mock_server.uri(),
            ..Default::default()
        };

        let client = EventStoreClient::new(config)?;
        let event_id = Uuid::new_v4();
        let created = Utc::now();

        let recorded_event = RecordedEvent {
            event_id,
            event_type: "TestEvent".to_string(),
            data: serde_json::json!({
                "message": "Hello"
            }),
            metadata: Value::Null,
            created,
        };

        Mock::given(method("GET"))
            .and(path("/streams/test-stream/0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![recorded_event]))
            .mount(&mock_server)
            .await;

        let events = client.read_stream::<TestEvent>("test-stream", 0, 1).await?;
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_id, event_id);
        assert_eq!(event.created_at, created);
        assert_eq!(event.data.message, "Hello");

        Ok(())
    }
}
