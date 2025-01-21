use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub redis: RedisConfig,
    pub event_store: EventStoreConfig,
    pub rabbitmq: RabbitMQConfig,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct EventStoreConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct RabbitMQConfig {
    pub url: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // For now, just load from environment variables
        Ok(Config {
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            event_store: EventStoreConfig {
                url: env::var("EVENTSTORE_URL")
                    .unwrap_or_else(|_| "http://localhost:2113".to_string()),
            },
            rabbitmq: RabbitMQConfig {
                url: env::var("RABBITMQ_URL")
                    .unwrap_or_else(|_| "amqp://localhost:5672".to_string()),
            },
        })
    }
}
