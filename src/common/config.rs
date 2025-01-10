use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{env, fs, path::Path};
use tracing::Level;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub backend_port: u16,
    pub default_language: String,
    pub log_level: String,
}

impl Settings {
    #[allow(clippy::disallowed_methods)]
    fn get_default_settings(run_mode: &str) -> Self {
        match run_mode {
            "dev" => Settings {
                backend_port: 3000,
                default_language: String::from("en"),
                log_level: String::from("debug"),
            },
            "prod" => Settings {
                backend_port: 8080,
                default_language: String::from("en"),
                log_level: String::from("info"),
            },
            "test" => Settings {
                backend_port: 3333,
                default_language: String::from("en"),
                log_level: String::from("debug"),
            },
            _ => {
                tracing::event!(
                    Level::WARN,
                    "Unknown run mode: {}, falling back to dev defaults",
                    run_mode
                );
                Settings::get_default_settings("dev")
            }
        }
    }

    #[allow(clippy::disallowed_methods)]
    fn ensure_env_file(run_mode: &str) -> Option<String> {
        if run_mode == "test" {
            return Some(format!("config/.env.{}.template", run_mode));
        }

        let env_file = format!(".env.{}", run_mode);
        let template_file = format!("config/.env.{}.template", run_mode);

        if !Path::new(&env_file).exists() {
            match fs::read_to_string(&template_file) {
                Ok(content) => {
                    if let Err(e) = fs::write(&env_file, content) {
                        tracing::event!(
                            Level::WARN,
                            "Failed to create {} from template: {}",
                            env_file,
                            e
                        );
                        return None;
                    }
                    tracing::event!(Level::INFO, "Created {} from template", env_file);
                }
                Err(e) => {
                    tracing::event!(
                        Level::WARN,
                        "Failed to read template {}: {}",
                        template_file,
                        e
                    );
                    return None;
                }
            }
        }

        Some(env_file)
    }

    #[allow(clippy::disallowed_methods)]
    fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());
        let default_settings = Settings::get_default_settings(&run_mode);

        let mut builder = Config::builder()
            .set_default("backend_port", default_settings.backend_port)?
            .set_default(
                "default_language",
                default_settings.default_language.as_str(),
            )?
            .set_default("log_level", default_settings.log_level.as_str())?;

        if let Some(env_file) = Settings::ensure_env_file(&run_mode) {
            builder = builder.add_source(File::with_name(&env_file).required(false));
        } else {
            tracing::event!(
                Level::WARN,
                "Using default values for {} environment",
                run_mode
            );
        }

        builder = builder.add_source(Environment::with_prefix("APP").separator("__"));

        builder.build()?.try_deserialize()
    }
}

static SETTINGS: Lazy<Settings> = Lazy::new(|| {
    Settings::new().unwrap_or_else(|err| {
        eprintln!("Failed to load settings: {}", err);
        std::process::exit(1);
    })
});

#[allow(dead_code)]
pub fn get_settings() -> &'static Settings {
    &SETTINGS
}

pub fn get_backend_port() -> u16 {
    SETTINGS.backend_port
}

pub fn get_default_language() -> &'static str {
    &SETTINGS.default_language
}

pub fn get_log_level() -> &'static str {
    &SETTINGS.log_level
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn cleanup_test_files() {
        let test_files = [".env.test_mode", ".env.cleanup_test"];
        for file in test_files.iter() {
            let _ = fs::remove_file(file);
        }
    }

    fn setup() {
        env::remove_var("APP__BACKEND_PORT");
        env::remove_var("APP__LOG_LEVEL");
        env::remove_var("APP__DEFAULT_LANGUAGE");
        env::remove_var("RUN_MODE");
    }

    #[test]
    fn test_default_settings() {
        setup();
        let dev_settings = Settings::get_default_settings("dev");
        assert_eq!(dev_settings.backend_port, 3000);
        assert_eq!(dev_settings.default_language, "en");
        assert_eq!(dev_settings.log_level, "debug");

        let prod_settings = Settings::get_default_settings("prod");
        assert_eq!(prod_settings.backend_port, 8080);
        assert_eq!(prod_settings.log_level, "info");

        let test_settings = Settings::get_default_settings("test");
        assert_eq!(test_settings.backend_port, 3333);
        assert_eq!(test_settings.log_level, "debug");

        let unknown_settings = Settings::get_default_settings("unknown");
        assert_eq!(unknown_settings.backend_port, 3000);
    }

    #[test]
    fn test_ensure_env_file_test_mode() {
        setup();
        let result = Settings::ensure_env_file("test");
        assert_eq!(result, Some("config/.env.test.template".to_string()));
    }

    #[test]
    fn test_ensure_env_file_creates_from_template() {
        setup();
        cleanup_test_files();

        // Create a temporary template file
        let template_content = "TEST_VALUE=123\n";
        #[allow(clippy::unwrap_used)]
        fs::create_dir_all("config").unwrap();
        #[allow(clippy::unwrap_used)]
        fs::write("config/.env.test_mode.template", template_content).unwrap();

        let result = Settings::ensure_env_file("test_mode");
        assert!(result.is_some());

        // Verify file was created and contains template content
        #[allow(clippy::unwrap_used)]
        let created_content = fs::read_to_string(".env.test_mode").unwrap();
        assert_eq!(created_content, template_content);

        // Cleanup
        #[allow(clippy::unwrap_used)]
        fs::remove_file("config/.env.test_mode.template").unwrap();
        cleanup_test_files();
    }

    #[test]
    fn test_ensure_env_file_missing_template() {
        setup();
        cleanup_test_files();

        let result = Settings::ensure_env_file("missing_template");
        assert!(result.is_none());
    }

    #[test]
    fn test_settings_new() {
        setup();

        // Test production settings
        env::set_var("RUN_MODE", "prod");
        #[allow(clippy::unwrap_used)]
        let prod_settings = Settings::new().unwrap();
        assert_eq!(prod_settings.backend_port, 8080);
        assert_eq!(prod_settings.default_language.as_str(), "en");
        assert_eq!(prod_settings.log_level.as_str(), "info");
        env::remove_var("RUN_MODE");

        // Test with environment override
        env::set_var("APP__BACKEND_PORT", "5000");
        #[allow(clippy::unwrap_used)]
        let override_settings = Settings::new().unwrap();
        assert_eq!(override_settings.backend_port, 5000);
        env::remove_var("APP__BACKEND_PORT");
    }
}
