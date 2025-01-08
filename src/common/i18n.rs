use crate::common::error::{AppError, AppResult};
use fluent::{FluentArgs, FluentResource};
use fluent_bundle::bundle::FluentBundle;
use intl_memoizer::concurrent::IntlLangMemoizer;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

type ConcurrentBundle = FluentBundle<FluentResource, IntlLangMemoizer>;

#[derive(Debug, Clone, Copy)]
pub enum SupportedLanguage {
    En,
    De,
    Fr,
    Es,
    Sq,
}

impl std::fmt::Display for SupportedLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedLanguage::En => write!(f, "en"),
            SupportedLanguage::De => write!(f, "de"),
            SupportedLanguage::Fr => write!(f, "fr"),
            SupportedLanguage::Es => write!(f, "es"),
            SupportedLanguage::Sq => write!(f, "sq"),
        }
    }
}

impl SupportedLanguage {
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::En, Self::De, Self::Fr, Self::Es, Self::Sq]
            .iter()
            .copied()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::De => "de",
            Self::Fr => "fr",
            Self::Es => "es",
            Self::Sq => "sq",
        }
    }
}

#[derive(Clone)]
pub struct I18nManager {
    bundles: Arc<RwLock<HashMap<String, Arc<ConcurrentBundle>>>>,
    default_lang: String,
}

impl std::fmt::Debug for I18nManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18nManager")
            .field("default_lang", &self.default_lang)
            .finish_non_exhaustive()
    }
}

impl I18nManager {
    pub async fn new() -> AppResult<Self> {
        let mut bundles = HashMap::new();

        for lang in SupportedLanguage::iter() {
            let bundle = Self::create_bundle_for_language(lang)
                .await
                .map_err(|e| (AppError::I18n(format!("{:?}", e)), Default::default()))?;
            bundles.insert(lang.as_str().to_string(), Arc::new(bundle));
        }

        Ok(Self {
            bundles: Arc::new(RwLock::new(bundles)),
            default_lang: "en".to_string(),
        })
    }

    pub async fn format_message(
        &self,
        lang: &str,
        message_id: &str,
        args: Option<HashMap<String, String>>,
    ) -> String {
        let bundle = match self.get_bundle(lang).await {
            Ok(b) => b,
            Err(_) => {
                let bundles = self.bundles.read().await;
                bundles
                    .get("en")
                    .unwrap_or_else(|| {
                        bundles
                            .get("en")
                            .unwrap_or_else(|| panic!("Default English bundle not found"))
                    })
                    .clone()
            }
        };

        let mut fluent_args = FluentArgs::new();
        if let Some(args) = args {
            for (key, value) in args {
                fluent_args.set(key, value);
            }
        }

        bundle
            .get_message(message_id)
            .and_then(|msg| msg.value())
            .map(|pattern| {
                bundle
                    .format_pattern(pattern, Some(&fluent_args), &mut vec![])
                    .into_owned()
            })
            .unwrap_or_else(|| message_id.to_string())
    }

    pub async fn get_bundle(&self, lang: &str) -> AppResult<Arc<ConcurrentBundle>> {
        let bundles = self.bundles.read().await;
        bundles
            .get(lang)
            .or_else(|| bundles.get(&self.default_lang))
            .cloned()
            .ok_or_else(|| {
                (
                    AppError::I18n("No bundle found and no default fallback available".into()),
                    Default::default(),
                )
            })
    }

    async fn create_bundle_for_language(lang: SupportedLanguage) -> AppResult<ConcurrentBundle> {
        let mut bundle =
            FluentBundle::new_concurrent(vec![lang.as_str().parse().map_err(|e| {
                (
                    AppError::I18n(format!("Failed to parse language: {:?}", e)),
                    Default::default(),
                )
            })?]);

        let path = PathBuf::from("locales")
            .join(lang.as_str())
            .join("main.ftl");
        let source = fs::read_to_string(&path).map_err(|e| {
            (
                AppError::I18n(format!("Failed to read file: {:?}", e)),
                Default::default(),
            )
        })?;

        let resource = FluentResource::try_new(source).map_err(|(_, errors)| {
            (
                AppError::I18n(format!("Parse errors: {:?}", errors)),
                Default::default(),
            )
        })?;

        bundle.add_resource(resource).map_err(|errors| {
            (
                AppError::I18n(format!("Failed to add resource: {:?}", errors)),
                Default::default(),
            )
        })?;

        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_translations() -> AppResult<()> {
        let test_content = "test-message = Test message content";
        let test_dirs = ["en", "de", "fr", "es", "sq"];

        for dir in test_dirs.iter() {
            let dir_path = PathBuf::from("locales").join(dir);
            fs::create_dir_all(&dir_path).map_err(|e| {
                (
                    AppError::I18n(format!("Failed to create directory: {:?}", e)),
                    Default::default(),
                )
            })?;

            let file_path = dir_path.join("main.ftl");
            fs::write(&file_path, test_content).map_err(|e| {
                (
                    AppError::I18n(format!("Failed to write file: {:?}", e)),
                    Default::default(),
                )
            })?;
        }
        Ok(())
    }

    fn cleanup_test_translations() {
        let _ = fs::remove_dir_all("locales");
    }

    #[tokio::test]
    async fn test_i18n_manager_creation() -> AppResult<()> {
        setup_test_translations()?;
        let manager = I18nManager::new().await?;
        let bundle = manager.get_bundle("en").await?;
        assert!(bundle.has_message("test-message"));
        cleanup_test_translations();
        Ok(())
    }

    #[tokio::test]
    async fn test_format_message() -> AppResult<()> {
        setup_test_translations()?;
        let manager = I18nManager::new().await?;
        let message = manager.format_message("en", "test-message", None).await;
        assert_eq!(message, "Test message content");
        cleanup_test_translations();
        Ok(())
    }
}
