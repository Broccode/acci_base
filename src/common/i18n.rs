use {
    crate::common::{
        config,
        error::{AppError, AppResult},
    },
    fluent::{FluentArgs, FluentResource},
    fluent_bundle::bundle::FluentBundle,
    intl_memoizer::concurrent::IntlLangMemoizer,
    std::{collections::HashMap, fs, path::PathBuf, sync::Arc},
    tokio::sync::RwLock,
};

type ConcurrentBundle = FluentBundle<FluentResource, IntlLangMemoizer>;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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

const LOCALES_DIR: &str = "locales";

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

#[async_trait::async_trait]
pub trait ResourceProvider: Send + Sync {
    async fn get_resource(&self, lang: SupportedLanguage) -> AppResult<String>;
}

pub struct FileResourceProvider;

#[async_trait::async_trait]
impl ResourceProvider for FileResourceProvider {
    async fn get_resource(&self, lang: SupportedLanguage) -> AppResult<String> {
        let path = PathBuf::from(LOCALES_DIR)
            .join(lang.as_str())
            .join("main.ftl");
        fs::read_to_string(&path).map_err(|e| {
            (
                AppError::I18n(format!("Failed to read file: {:?}", e)),
                Default::default(),
            )
        })
    }
}

impl I18nManager {
    pub async fn new() -> AppResult<Self> {
        Self::new_with_provider(FileResourceProvider).await
    }

    pub async fn new_with_provider<P: ResourceProvider>(provider: P) -> AppResult<Self> {
        let mut bundles = HashMap::new();

        for lang in SupportedLanguage::iter() {
            let bundle = Self::create_bundle_for_language(lang, &provider)
                .await
                .map_err(|e| (AppError::I18n(format!("{:?}", e)), Default::default()))?;
            bundles.insert(lang.as_str().to_string(), Arc::new(bundle));
        }

        Ok(Self {
            bundles: Arc::new(RwLock::new(bundles)),
            default_lang: config::get_default_language().to_string(),
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

    async fn create_bundle_for_language<P: ResourceProvider>(
        lang: SupportedLanguage,
        provider: &P,
    ) -> AppResult<ConcurrentBundle> {
        let mut bundle =
            FluentBundle::new_concurrent(vec![lang.as_str().parse().map_err(|e| {
                (
                    AppError::I18n(format!("Failed to parse language: {:?}", e)),
                    Default::default(),
                )
            })?]);

        let source = provider.get_resource(lang).await?;

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
pub struct TestResourceProvider {
    resources: HashMap<SupportedLanguage, String>,
}

#[cfg(test)]
impl TestResourceProvider {
    pub fn new() -> Self {
        let mut resources = HashMap::new();
        let test_content = "test-message = Test message content
health-status = System health status
system-status-healthy = Healthy
system-status-ready = Ready
system-ready-message = System is ready to accept requests";

        for lang in SupportedLanguage::iter() {
            resources.insert(lang, test_content.to_string());
        }

        Self { resources }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl ResourceProvider for TestResourceProvider {
    async fn get_resource(&self, lang: SupportedLanguage) -> AppResult<String> {
        self.resources.get(&lang).cloned().ok_or_else(|| {
            (
                AppError::I18n(format!("Test resource not found for language: {:?}", lang)),
                Default::default(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> AppResult<I18nManager> {
        I18nManager::new_with_provider(TestResourceProvider::new()).await
    }

    #[tokio::test]
    async fn test_i18n_manager_creation() -> AppResult<()> {
        let manager = setup().await?;
        let bundle = manager.get_bundle("en").await?;
        assert!(bundle.has_message("test-message"));
        Ok(())
    }

    #[tokio::test]
    async fn test_format_message() -> AppResult<()> {
        let manager = setup().await?;
        let message = manager.format_message("en", "test-message", None).await;
        assert_eq!(message, "Test message content");
        Ok(())
    }
}
