mod language;

pub use language::{LanguageExt, LanguageLayer};

pub fn setup_i18n(i18n_manager: std::sync::Arc<crate::common::i18n::I18nManager>) -> LanguageLayer {
    LanguageLayer::new(i18n_manager)
}
