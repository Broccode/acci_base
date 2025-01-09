use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub backend_port: u16,
    pub default_language: String,
    pub log_level: String,
}

impl Settings {
    fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());
        let env_file = format!("config/.env.{}", run_mode);

        let s = Config::builder()
            .set_default("backend_port", 3333)?
            .set_default("default_language", "en")?
            .set_default("log_level", "info")?
            .add_source(File::with_name(&env_file).required(false))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        s.try_deserialize()
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

    #[test]
    fn test_settings_defaults() {
        let settings = Settings::new().unwrap();
        assert_eq!(settings.backend_port, 3333);
        assert_eq!(settings.default_language, "en");
        assert_eq!(settings.log_level, "info");
    }
}
