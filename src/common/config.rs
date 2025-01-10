use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{env, fs, path::Path};
use tracing::Level;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub backend_port: u16,
    pub default_language: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub logging: LoggingSettings,
}

impl Settings {
    #[allow(clippy::disallowed_methods)]
    fn get_default_settings(run_mode: &str) -> Self {
        match run_mode {
            "dev" => Settings {
                server: ServerSettings {
                    backend_port: 3333,
                    default_language: String::from("en"),
                },
                logging: LoggingSettings {
                    level: String::from("debug"),
                },
            },
            "prod" => Settings {
                server: ServerSettings {
                    backend_port: 8080,
                    default_language: String::from("en"),
                },
                logging: LoggingSettings {
                    level: String::from("info"),
                },
            },
            "test" => Settings {
                server: ServerSettings {
                    backend_port: 3333,
                    default_language: String::from("en"),
                },
                logging: LoggingSettings {
                    level: String::from("debug"),
                },
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
    fn ensure_config_file(run_mode: &str) -> Option<String> {
        if run_mode == "test" {
            return Some("config/config.test.toml".to_string());
        }

        let config_file = format!("config/config.{}.toml", run_mode);
        let template_file = format!("config/config.{}.toml.template", run_mode);

        if !Path::new(&config_file).exists() {
            match fs::read_to_string(&template_file) {
                Ok(content) => {
                    if let Err(e) = fs::write(&config_file, content) {
                        tracing::event!(
                            Level::WARN,
                            "Failed to create {} from template: {}",
                            config_file,
                            e
                        );
                        return None;
                    }
                    tracing::event!(Level::INFO, "Created {} from template", config_file);
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

        Some(config_file)
    }

    #[allow(clippy::disallowed_methods)]
    fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());
        let default_settings = Settings::get_default_settings(&run_mode);

        let mut builder = Config::builder();

        // First try to load the config file
        if let Some(config_file) = Settings::ensure_config_file(&run_mode) {
            builder = builder.add_source(File::with_name(&config_file).required(false));
        } else {
            tracing::event!(
                Level::WARN,
                "Config file not found for {} environment, will use defaults for missing values",
                run_mode
            );
        }

        // Then add environment variables
        builder = builder.add_source(
            Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        );

        // Finally add defaults for any missing values
        builder = builder
            .set_default("server.backend_port", default_settings.server.backend_port)?
            .set_default(
                "server.default_language",
                default_settings.server.default_language.as_str(),
            )?
            .set_default("logging.level", default_settings.logging.level.as_str())?;

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
    SETTINGS.server.backend_port
}

pub fn get_default_language() -> &'static str {
    &SETTINGS.server.default_language
}

pub fn get_log_level() -> &'static str {
    &SETTINGS.logging.level
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn cleanup_test_files() {
        let test_files = [
            "config/config.test_mode.toml",
            "config/config.cleanup_test.toml",
        ];
        for file in test_files.iter() {
            let _ = fs::remove_file(file);
        }
    }

    fn setup() {
        env::remove_var("APP__SERVER__BACKEND_PORT");
        env::remove_var("APP__LOGGING__LEVEL");
        env::remove_var("APP__SERVER__DEFAULT_LANGUAGE");
        env::remove_var("RUN_MODE");
    }

    #[test]
    fn test_default_settings() {
        setup();
        let dev_settings = Settings::get_default_settings("dev");
        assert_eq!(dev_settings.server.backend_port, 3333);
        assert_eq!(dev_settings.server.default_language, "en");
        assert_eq!(dev_settings.logging.level, "debug");

        let prod_settings = Settings::get_default_settings("prod");
        assert_eq!(prod_settings.server.backend_port, 8080);
        assert_eq!(prod_settings.logging.level, "info");

        let test_settings = Settings::get_default_settings("test");
        assert_eq!(test_settings.server.backend_port, 3333);
        assert_eq!(test_settings.logging.level, "debug");

        let unknown_settings = Settings::get_default_settings("unknown");
        assert_eq!(unknown_settings.server.backend_port, 3333);
    }

    #[test]
    fn test_ensure_config_file_test_mode() {
        setup();
        let result = Settings::ensure_config_file("test");
        assert_eq!(result, Some("config/config.test.toml".to_string()));
    }

    #[test]
    fn test_ensure_config_file_creates_from_template() {
        setup();
        cleanup_test_files();

        // Create a temporary template file
        let template_content = r#"[server]
backend_port = 123
default_language = "en"

[logging]
level = "debug"
"#;
        #[allow(clippy::unwrap_used)]
        fs::create_dir_all("config").unwrap();
        #[allow(clippy::unwrap_used)]
        fs::write("config/config.test_mode.toml.template", template_content).unwrap();

        let result = Settings::ensure_config_file("test_mode");
        assert!(result.is_some());

        // Verify file was created and contains template content
        #[allow(clippy::unwrap_used)]
        let created_content = fs::read_to_string("config/config.test_mode.toml").unwrap();
        assert_eq!(created_content, template_content);

        // Cleanup
        #[allow(clippy::unwrap_used)]
        fs::remove_file("config/config.test_mode.toml.template").unwrap();
        cleanup_test_files();
    }

    #[test]
    fn test_ensure_config_file_missing_template() {
        setup();
        cleanup_test_files();

        let result = Settings::ensure_config_file("missing_template");
        assert!(result.is_none());
    }

    #[test]
    fn test_settings_new() {
        setup();

        // Test production settings
        env::set_var("RUN_MODE", "prod");
        #[allow(clippy::unwrap_used)]
        let prod_settings = Settings::new().unwrap();
        assert_eq!(prod_settings.server.backend_port, 8080);
        assert_eq!(prod_settings.server.default_language.as_str(), "en");
        assert_eq!(prod_settings.logging.level.as_str(), "info");
        env::remove_var("RUN_MODE");

        // Test with environment override
        env::set_var("APP__SERVER__BACKEND_PORT", "5000");
        #[allow(clippy::unwrap_used)]
        let override_settings = Settings::new().unwrap();
        assert_eq!(override_settings.server.backend_port, 5000);
        env::remove_var("APP__SERVER__BACKEND_PORT");
    }
}
