use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync {
    /// Returns the type of the event
    fn event_type(&self) -> &str;

    /// Returns the aggregate ID this event belongs to
    fn aggregate_id(&self) -> Uuid;

    /// Returns the version of the event schema
    fn schema_version(&self) -> u32;

    /// Returns the timestamp when the event occurred
    fn timestamp(&self) -> DateTime<Utc>;

    /// Returns the correlation ID for tracing
    fn correlation_id(&self) -> Option<Uuid>;

    /// Returns the causation ID for event chains
    fn causation_id(&self) -> Option<Uuid>;

    /// Returns the tenant ID if applicable
    fn tenant_id(&self) -> Option<Uuid>;
}

/// Common metadata for all events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub schema_version: u32,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
}

pub trait TypeName {
    fn type_name(&self) -> String;
}

/// Base structure for all events
#[derive(Debug, Clone)]
pub struct Event<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + TypeName,
{
    pub data: T,
    pub created_at: DateTime<Utc>,
    pub event_id: Uuid,
    pub version: u64,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
}

impl<T> Event<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + TypeName,
{
    pub fn new(
        data: T,
        version: u64,
        correlation_id: Option<Uuid>,
        causation_id: Option<Uuid>,
        event_id: Option<Uuid>,
    ) -> Self {
        Self {
            data,
            version,
            correlation_id,
            causation_id,
            created_at: Utc::now(),
            event_id: event_id.unwrap_or_else(Uuid::new_v4),
        }
    }

    pub fn to_event_data(&self) -> Result<EventData> {
        Ok(EventData {
            event_type: self.data.type_name(),
            data: serde_json::to_value(&self.data)?,
            metadata: Value::Null,
            event_id: self.event_id,
        })
    }
}

/// Stream naming conventions
pub struct StreamName;

impl StreamName {
    pub fn tenant_stream(tenant_id: Uuid) -> String {
        format!("tenant-{}", tenant_id)
    }

    pub fn user_stream(tenant_id: Uuid, user_id: Uuid) -> String {
        format!("user-{}-{}", tenant_id, user_id)
    }

    pub fn category_stream(category: &str) -> String {
        format!("$ce-{}", category)
    }

    pub fn all_stream() -> &'static str {
        "$all"
    }
}

/// Event categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    Tenant,
    User,
    System,
}

impl EventCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventCategory::Tenant => "tenant",
            EventCategory::User => "user",
            EventCategory::System => "system",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventData {
    #[serde(rename = "eventId")]
    pub event_id: Uuid,
    #[serde(rename = "eventType")]
    pub event_type: String,
    pub data: Value,
    pub metadata: Value,
}

impl EventData {
    pub fn new<T: Serialize>(
        event_type: String,
        data: T,
        metadata: Value,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            event_id: Uuid::new_v4(),
            event_type,
            data: serde_json::to_value(data)?,
            metadata,
        })
    }
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
    fn test_event_creation() {
        let test_event = TestEvent {
            message: "Hello".to_string(),
        };

        let event = Event::new(test_event, 1, None, None, None);
        assert_eq!(event.data.message, "Hello");
    }

    #[test]
    fn test_event_to_event_data() -> Result<()> {
        let test_event = TestEvent {
            message: "Hello".to_string(),
        };

        let event = Event::new(test_event, 1, None, None, None);
        let event_data = event.to_event_data()?;

        assert_eq!(event_data.event_type, "TestEvent");
        assert_eq!(event_data.event_id, event.event_id);

        let data: TestEvent = serde_json::from_value(event_data.data)?;
        assert_eq!(data.message, "Hello");

        Ok(())
    }

    #[test]
    fn test_stream_naming() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        assert_eq!(
            StreamName::tenant_stream(tenant_id),
            format!("tenant-{}", tenant_id)
        );
        assert_eq!(
            StreamName::user_stream(tenant_id, user_id),
            format!("user-{}-{}", tenant_id, user_id)
        );
        assert_eq!(StreamName::category_stream("tenant"), "$ce-tenant");
        assert_eq!(StreamName::all_stream(), "$all");
    }

    #[test]
    fn test_event_categories() {
        assert_eq!(EventCategory::Tenant.as_str(), "tenant");
        assert_eq!(EventCategory::User.as_str(), "user");
        assert_eq!(EventCategory::System.as_str(), "system");
    }
}
