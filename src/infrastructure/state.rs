use std::sync::Arc;

use metrics_exporter_prometheus::PrometheusHandle;

use crate::common::i18n::I18nManager;
use crate::domain::tenant::TenantService;
use crate::infrastructure::event_store::EventStoreClient;
use crate::infrastructure::message_broker::MessageBroker;
use crate::infrastructure::redis::RedisClient;

#[derive(Clone)]
pub struct AppState {
    pub tenant_service: Arc<dyn TenantService>,
    pub i18n: Arc<I18nManager>,
    pub metrics_handle: PrometheusHandle,
    pub redis: Option<Arc<RedisClient>>,
    pub event_store: Option<Arc<EventStoreClient>>,
    pub message_broker: Option<Arc<MessageBroker>>,
}

impl AppState {
    pub fn new(
        tenant_service: Arc<dyn TenantService>,
        i18n: Arc<I18nManager>,
        metrics_handle: PrometheusHandle,
        redis: Arc<RedisClient>,
        event_store: Arc<EventStoreClient>,
        message_broker: Arc<MessageBroker>,
    ) -> Self {
        Self {
            tenant_service,
            i18n,
            metrics_handle,
            redis: Some(redis),
            event_store: Some(event_store),
            message_broker: Some(message_broker),
        }
    }
}
