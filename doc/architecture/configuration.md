# Configuration System

## Overview

The ACCI Framework uses a layered configuration system that supports:
- Environment-specific settings
- Environment variable overrides
- Default values
- Type-safe configuration
- Configuration validation
- Template-based configuration files

## Configuration Hierarchy

1. Environment Variables (highest priority)
2. Environment-specific config file
3. Default values (lowest priority)

## Configuration Files

### Location
```
config/
├── config.dev.toml
├── config.prod.toml
├── config.test.toml
├── config.dev.toml.template
├── config.prod.toml.template
└── config.test.toml.template
```

### Template Example
```toml
[server]
backend_port = 3333
default_language = "en"

[database]
host = "db"
port = 5432
name = "${DATABASE_NAME}"
user = "${DATABASE_USER}"
password = "${DATABASE_PASSWORD}"
max_connections = 100
connection_timeout = 30
idle_timeout = 300

[redis]
url = "redis://:${REDIS_PASSWORD}@redis:6379"
pool_size = 32

[logging]
level = "info"
format = "json"
```

## Environment Variables

### Naming Convention
- Prefix: `APP__`
- Separator: `__`
- Example: `APP__SERVER__BACKEND_PORT=8080`

### Type Conversion
- Numbers: Parsed as integers or floats
- Booleans: "true"/"false" (case insensitive)
- Arrays: Comma-separated values
- Duration: Supports units (e.g., "30s", "1h")

## Configuration Structs

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub backend_port: u16,
    pub default_language: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub level: String,
    pub format: String,
}
```

## Configuration Loading

```rust
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());
        let mut builder = Config::builder();

        // Set defaults
        builder = builder.set_defaults()?;

        // Load environment-specific config
        if let Some(config_file) = Self::ensure_config_file(&run_mode) {
            builder = builder.add_source(
                File::with_name(&config_file).required(false)
            );
        }

        // Add environment variables
        builder = builder.add_source(
            Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true)
        );

        builder.build()?.try_deserialize()
    }
}
```

## Configuration Validation

### Database Configuration
```rust
impl DatabaseSettings {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_connections < self.min_connections {
            return Err(ConfigError::InvalidValue {
                field: "max_connections",
                message: "must be greater than min_connections".into(),
            });
        }

        if self.connection_timeout == 0 {
            return Err(ConfigError::InvalidValue {
                field: "connection_timeout",
                message: "must be greater than 0".into(),
            });
        }

        Ok(())
    }
}
```

## Environment-Specific Settings

### Development
```toml
[server]
backend_port = 3333
default_language = "en"

[logging]
level = "debug"
format = "pretty"
```

### Production
```toml
[server]
backend_port = 8080
default_language = "en"

[logging]
level = "info"
format = "json"
```

### Test
```toml
[server]
backend_port = 3333
default_language = "en"

[logging]
level = "debug"
format = "pretty"
```

## Configuration Access

### Global Access
```rust
use once_cell::sync::Lazy;

static SETTINGS: Lazy<Settings> = Lazy::new(|| {
    Settings::new().unwrap_or_else(|err| {
        eprintln!("Failed to load settings: {}", err);
        std::process::exit(1);
    })
});

pub fn get_settings() -> &'static Settings {
    &SETTINGS
}
```

### Component-Specific Access
```rust
pub fn get_database_config() -> DatabaseSettings {
    get_settings().database.clone()
}

pub fn get_redis_config() -> RedisSettings {
    get_settings().redis.clone()
}
```

## Error Handling

```rust
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required value for field: {0}")]
    MissingValue(String),

    #[error("Invalid value for {field}: {message}")]
    InvalidValue {
        field: String,
        message: String,
    },

    #[error("Failed to parse {field}: {message}")]
    ParseError {
        field: String,
        message: String,
    },

    #[error("Configuration file error: {0}")]
    FileError(String),
}
```

## Testing Support

### Mock Configuration
```rust
#[cfg(test)]
pub struct MockConfig {
    pub settings: Settings,
}

#[cfg(test)]
impl MockConfig {
    pub fn new() -> Self {
        Self {
            settings: Settings::default(),
        }
    }

    pub fn with_database_config(mut self, config: DatabaseSettings) -> Self {
        self.settings.database = config;
        self
    }

    pub fn with_redis_config(mut self, config: RedisSettings) -> Self {
        self.settings.redis = config;
        self
    }
}
```

### Test Utilities
```rust
#[cfg(test)]
pub fn setup_test_config() -> Settings {
    let mut settings = Settings::default();
    settings.database.name = "test_db".to_string();
    settings.database.max_connections = 5;
    settings
}

#[cfg(test)]
pub fn with_test_config<F>(test: F)
where
    F: FnOnce(&Settings),
{
    let settings = setup_test_config();
    test(&settings);
}
``` 