mod language;
mod tenant;

pub use language::LanguageLayer;
pub use tenant::TenantLayer;

pub fn setup_i18n(i18n_manager: std::sync::Arc<crate::common::i18n::I18nManager>) -> LanguageLayer {
    LanguageLayer::new(i18n_manager)
}

#[allow(dead_code)]
pub fn setup_tenant(
    tenant_service: std::sync::Arc<dyn crate::domain::tenant::TenantService>,
) -> TenantLayer {
    TenantLayer::new(tenant_service)
}
