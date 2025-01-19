use std::sync::Arc;

use metrics_exporter_prometheus::PrometheusHandle;

use crate::common::i18n::I18nManager;
use crate::domain::tenant::TenantService;

#[derive(Clone)]
pub struct AppState {
    pub tenant_service: Arc<dyn TenantService>,
    pub i18n: Arc<I18nManager>,
    pub metrics_handle: PrometheusHandle,
}

impl AppState {
    pub fn new(
        tenant_service: Arc<dyn TenantService>,
        i18n: Arc<I18nManager>,
        metrics_handle: PrometheusHandle,
    ) -> Self {
        Self {
            tenant_service,
            i18n,
            metrics_handle,
        }
    }
}
