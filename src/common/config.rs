use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{env, fs, path::Path};
use tracing::Level;

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

// Mock file system for testing
#[cfg(test)]
#[derive(Default)]
struct MockFs {
    files: HashMap<String, String>,
}

#[cfg(test)]
impl MockFs {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    fn write(&mut self, path: &str, content: &str) {
        self.files.insert(path.to_string(), content.to_string());
    }

    fn read(&self, path: &str) -> Option<String> {
        self.files.get(path).cloned()
    }

    fn exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    fn clear(&mut self) {
        self.files.clear();
    }
}

#[cfg(test)]
static MOCK_FS: Lazy<Mutex<MockFs>> = Lazy::new(|| Mutex::new(MockFs::new()));

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

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct AppConfig {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub logging: LoggingSettings,
    pub keycloak: KeycloakConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                backend_port: 3333,
                default_language: "en".to_string(),
            },
            database: DatabaseSettings {
                host: "localhost".to_string(),
                port: 5432,
                name: "acci_test".to_string(),
                user: "acci".to_string(),
                password: "acci".to_string(),
            },
            redis: RedisSettings {
                url: "redis://localhost:6379".to_string(),
            },
            logging: LoggingSettings {
                level: "debug".to_string(),
            },
            keycloak: KeycloakConfig {
                url: "http://localhost:8080".to_string(),
                realm: "acci".to_string(),
                client_id: "acci-backend".to_string(),
                client_secret: "test_secret".to_string(),
                verify_token: true,
                public_key_cache_ttl: 3600,
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct KeycloakConfig {
    pub url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    #[serde(default = "default_verify_token")]
    pub verify_token: bool,
    #[serde(default = "default_public_key_cache_ttl")]
    pub public_key_cache_ttl: u64,
}

fn default_verify_token() -> bool {
    true
}

fn default_public_key_cache_ttl() -> u64 {
    3600 // 1 hour in seconds
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
            },
        }
    }

    #[allow(clippy::disallowed_methods)]
    fn ensure_config_file(run_mode: &str) -> Option<String> {
        let config_file = format!("config/config.{}.toml", run_mode);
        let template_file = format!("config/config.{}.toml.template", run_mode);

        if !Settings::file_exists(&config_file) {
            match Settings::read_file(&template_file) {
                Some(content) => {
                    if let Err(e) = Settings::write_file(&config_file, &content) {
                        tracing::event!(
                            Level::WARN,
                            "Failed to create {} from template: {}",
                            config_file,
                            e
                        );
                        return None;
                    }
                    tracing::event!(Level::INFO, "Created {} from template", config_file);
                },
                None => {
                    tracing::event!(Level::WARN, "Failed to read template {}", template_file,);
                    return None;
                },
            }
        }

        Some(config_file)
    }

    #[allow(clippy::disallowed_methods)]
    fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());
        let default_settings = Settings::get_default_settings(&run_mode);

        let mut builder = Config::builder();

        // First set defaults (lowest priority)
        builder = builder
            .set_default("server.backend_port", default_settings.server.backend_port)?
            .set_default(
                "server.default_language",
                default_settings.server.default_language.as_str(),
            )?
            .set_default("logging.level", default_settings.logging.level.as_str())?;

        // Then load environment-specific config file (middle priority)
        if let Some(config_file) = Settings::ensure_config_file(&run_mode) {
            if Settings::file_exists(&config_file) {
                builder = builder.add_source(File::with_name(&config_file).required(false));
            }
        }

        // Finally add environment variables (highest priority)
        builder = builder.add_source(
            Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        );

        builder.build()?.try_deserialize()
    }

    #[cfg(not(test))]
    fn file_exists(path: &str) -> bool {
        Path::new(path).exists()
    }

    #[cfg(not(test))]
    fn read_file(path: &str) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    #[cfg(not(test))]
    fn write_file(path: &str, content: &str) -> Result<(), std::io::Error> {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)
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

pub fn get_database_config() -> DatabaseSettings {
    AppConfig::default().database
}

#[cfg(test)]
impl Settings {
    fn with_mock_fs() -> &'static Mutex<MockFs> {
        &MOCK_FS
    }

    fn read_file(path: &str) -> Option<String> {
        if cfg!(test) {
            Settings::with_mock_fs().lock().unwrap().read(path)
        } else {
            fs::read_to_string(path).ok()
        }
    }

    fn write_file(path: &str, content: &str) -> Result<(), std::io::Error> {
        if cfg!(test) {
            Settings::with_mock_fs()
                .lock()
                .unwrap()
                .write(path, content);
            Ok(())
        } else {
            fs::write(path, content)
        }
    }

    fn file_exists(path: &str) -> bool {
        if cfg!(test) {
            Settings::with_mock_fs().lock().unwrap().exists(path)
        } else {
            Path::new(path).exists()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn setup() {
        let mock_fs = Settings::with_mock_fs();
        mock_fs.lock().unwrap().clear();
        env::remove_var("APP__SERVER__BACKEND_PORT");
        env::remove_var("APP__LOGGING__LEVEL");
        env::remove_var("APP__SERVER__DEFAULT_LANGUAGE");
        env::remove_var("RUN_MODE");
    }

    #[test]
    #[serial]
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
    #[serial]
    fn test_ensure_config_file_test_mode() {
        setup();

        // Setup mock test config
        Settings::with_mock_fs().lock().unwrap().write(
            "config/config.test.toml",
            r#"[server]
backend_port = 3333
default_language = "en"

[logging]
level = "debug"
"#,
        );

        let result = Settings::ensure_config_file("test");
        assert_eq!(result, Some("config/config.test.toml".to_string()));
    }

    #[test]
    #[serial]
    fn test_ensure_config_file_creates_from_template() {
        setup();

        // Setup mock template
        let template_content = r#"[server]
backend_port = 123
default_language = "en"

[logging]
level = "debug"
"#;
        Settings::with_mock_fs()
            .lock()
            .unwrap()
            .write("config/config.test_mode.toml.template", template_content);

        let result = Settings::ensure_config_file("test_mode");
        assert!(result.is_some());

        // Verify file was created with template content
        let created_content = Settings::with_mock_fs()
            .lock()
            .unwrap()
            .read("config/config.test_mode.toml")
            .unwrap();
        assert_eq!(created_content, template_content);
    }

    #[test]
    #[serial]
    fn test_ensure_config_file_missing_template() {
        setup();
        let result = Settings::ensure_config_file("missing_template");
        assert!(result.is_none());
    }

    #[test]
    #[serial]
    fn test_settings_new_production() {
        setup();

        // Setup mock production config
        Settings::with_mock_fs().lock().unwrap().write(
            "config/config.prod.toml",
            r#"[server]
backend_port = 8080
default_language = "en"

[logging]
level = "info"
"#,
        );

        env::set_var("RUN_MODE", "prod");
        let prod_settings = Settings::new().unwrap();
        assert_eq!(prod_settings.server.backend_port, 8080);
        assert_eq!(prod_settings.server.default_language.as_str(), "en");
        assert_eq!(prod_settings.logging.level.as_str(), "info");
    }

    #[test]
    #[serial]
    fn test_settings_new_with_env_override() {
        setup();

        // Setup mock config
        Settings::with_mock_fs().lock().unwrap().write(
            "config/config.dev.toml",
            r#"[server]
backend_port = 3333
default_language = "en"

[logging]
level = "debug"
"#,
        );

        env::set_var("APP__SERVER__BACKEND_PORT", "5000");
        let override_settings = Settings::new().unwrap();
        assert_eq!(override_settings.server.backend_port, 5000);
    }
}
