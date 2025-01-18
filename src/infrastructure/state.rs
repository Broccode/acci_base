use std::sync::Arc;

use crate::common::i18n::I18nManager;
use crate::domain::tenant::TenantService;

#[derive(Clone)]
pub struct AppState {
    pub tenant_service: Arc<dyn TenantService>,
    pub i18n: Arc<I18nManager>,
}

impl AppState {
    pub fn new(tenant_service: Arc<dyn TenantService>, i18n: Arc<I18nManager>) -> Self {
        Self {
            tenant_service,
            i18n,
        }
    }
}
