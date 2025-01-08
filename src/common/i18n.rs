use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use fluent::{FluentArgs, FluentResource};
use fluent_bundle::bundle::FluentBundle as BaseFluentBundle;
use fluent_langneg::{negotiate_languages, NegotiationStrategy, LanguageIdentifier as FluentLanguageIdentifier};
use unic_langid::LanguageIdentifier;
use intl_memoizer::concurrent::IntlLangMemoizer;
use tokio::sync::RwLock;

/// Supported languages in the application
#[derive(Debug, Clone, Copy)]
pub enum SupportedLanguage {
    English,
    German,
    Albanian,
    French,
    Spanish,
}

impl SupportedLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedLanguage::English => "en",
            SupportedLanguage::German => "de",
            SupportedLanguage::Albanian => "sq",
            SupportedLanguage::French => "fr",
            SupportedLanguage::Spanish => "es",
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::English,
            Self::German,
            Self::Albanian,
            Self::French,
            Self::Spanish,
        ]
        .into_iter()
    }
}

type ConcurrentBundle = BaseFluentBundle<FluentResource, IntlLangMemoizer>;

/// Thread-safe i18n manager that handles translations using Fluent
#[derive(Clone)]
pub struct I18nManager {
    bundles: Arc<RwLock<HashMap<String, Arc<ConcurrentBundle>>>>,
    default_lang: LanguageIdentifier,
}

impl std::fmt::Debug for I18nManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18nManager")
            .field("default_lang", &self.default_lang)
            .finish_non_exhaustive()
    }
}

impl I18nManager {
    /// Creates a new I18nManager instance
    pub async fn new() -> anyhow::Result<Self> {
        let mut bundles = HashMap::new();
        let default_lang: LanguageIdentifier = "en"
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse default language identifier: {}", e))?;

        // Initialize bundles for all supported languages
        for lang in SupportedLanguage::iter() {
            let bundle = Self::create_bundle_for_language(lang).await?;
            bundles.insert(lang.as_str().to_string(), Arc::new(bundle));
        }

        Ok(Self {
            bundles: Arc::new(RwLock::new(bundles)),
            default_lang,
        })
    }

    /// Gets the best matching bundle for the given language preferences
    pub async fn get_bundle(&self, requested_languages: &[&str]) -> Arc<ConcurrentBundle> {
        let bundles = self.bundles.read().await;
        
        let requested: Vec<FluentLanguageIdentifier> = requested_languages
            .iter()
            .filter_map(|lang| lang.parse().ok())
            .collect();

        let available: Vec<FluentLanguageIdentifier> = bundles
            .keys()
            .filter_map(|lang| lang.parse().ok())
            .collect();

        let default_lang: FluentLanguageIdentifier = self.default_lang.to_string().parse().unwrap();

        let negotiated = negotiate_languages(
            &requested,
            &available,
            Some(&default_lang),
            NegotiationStrategy::Filtering,
        );

        let selected = negotiated
            .first()
            .map(|lang| lang.to_string())
            .unwrap_or_else(|| self.default_lang.to_string());

        bundles
            .get(&selected)
            .unwrap_or_else(|| bundles.get(self.default_lang.to_string().as_str()).unwrap())
            .clone()
    }

    /// Formats a message with the given ID and arguments
    pub async fn format_message(
        &self,
        lang: &str,
        message_id: &str,
        args: Option<HashMap<String, String>>,
    ) -> String {
        let bundle = self.get_bundle(&[lang]).await;

        let msg = match bundle.get_message(message_id) {
            Some(msg) => msg,
            None => return format!("Message '{}' not found", message_id),
        };

        let pattern = match msg.value() {
            Some(pattern) => pattern,
            None => return format!("No pattern for message '{}'", message_id),
        };

        let mut errors = vec![];
        let args_vec: Vec<(&str, &str)> = args
            .iter()
            .flat_map(|args| args.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .collect();
        let mut fluent_args = if !args_vec.is_empty() {
            FluentArgs::from_iter(args_vec)
        } else {
            FluentArgs::new()
        };

        bundle
            .format_pattern(pattern, Some(&mut fluent_args), &mut errors)
            .to_string()
    }

    /// Creates a FluentBundle for a specific language
    async fn create_bundle_for_language(lang: SupportedLanguage) -> anyhow::Result<ConcurrentBundle> {
        let lang_id: LanguageIdentifier = lang.as_str().parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse language identifier: {}", e))?;
        let mut bundle = BaseFluentBundle::new_concurrent(vec![lang_id]);
        
        // Construct the path to the FTL file
        let mut path = PathBuf::from("locales");
        path.push(lang.as_str());
        path.push("main.ftl");

        // Read the FTL file
        let ftl_string = fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read FTL file for language {}: {}", lang.as_str(), e))?;
        
        let resource = FluentResource::try_new(ftl_string)
            .map_err(|_| anyhow::anyhow!("Failed to parse FTL resource for language {}", lang.as_str()))?;
        
        bundle.add_resource(resource)
            .map_err(|_| anyhow::anyhow!("Failed to add resource to bundle for language {}", lang.as_str()))?;

        // Set bundle attributes
        bundle.set_use_isolating(false); // Don't use Unicode isolation marks
        
        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_i18n_manager_creation() {
        let manager = I18nManager::new().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_language_negotiation() {
        let manager = I18nManager::new().await.unwrap();
        let bundle = manager.get_bundle(&["fr", "de"]).await;
        assert!(
            bundle.locales.contains(&"fr".parse().unwrap())
                || bundle.locales.contains(&"de".parse().unwrap())
                || bundle.locales.contains(&"en".parse().unwrap())
        );
    }

    #[tokio::test]
    async fn test_message_formatting() {
        let manager = I18nManager::new().await.unwrap();

        let mut args = HashMap::new();
        args.insert("name".to_string(), "Test User".to_string());

        let message = manager
            .format_message("en", "user-greeting", Some(args))
            .await;
        assert!(message.contains("Test User"));
    }

    #[tokio::test]
    async fn test_fallback_to_default_language() {
        let manager = I18nManager::new().await.unwrap();
        let message = manager
            .format_message("invalid-lang", "welcome", None)
            .await;
        assert!(!message.is_empty());
    }
}
