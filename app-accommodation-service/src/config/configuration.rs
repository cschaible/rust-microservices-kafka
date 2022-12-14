use std::env;
use std::sync::atomic::Ordering::SeqCst;

use common_db_mongodb::config::DatabaseConfiguration;
use common_security::config::SecurityConfiguration;
use config::Config;
use config::ConfigError;
use config::File;
use serde;
use serde::Deserialize;

use crate::common::api::SERVER_PORT;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Configuration {
    pub database: DatabaseConfiguration,
    pub kafka: KafkaConfiguration,
    pub logging: LoggingConfiguration,
    pub security: SecurityConfiguration,
    pub server: ServerConfiguration,
}

impl Configuration {
    pub fn load() -> Result<Self, ConfigError> {
        let profiles_raw_string = env::var("RUST_PROFILES_ACTIVE").unwrap_or_default();
        let active_profiles: Vec<&str> = profiles_raw_string
            .split(',')
            .into_iter()
            .map(|p| p.trim())
            .filter(|p| !(*p).is_empty())
            .collect();

        // Load always properties of application.yml
        let mut builder =
            Config::builder().add_source(File::with_name("resources/application.yml"));

        // Load property files for profiles
        for profile in active_profiles {
            builder = builder.add_source(
                File::with_name(&format!("resources/application-{}.yml", profile)).required(false),
            );
        }

        let parsed_config: Result<Configuration, ConfigError> = builder.build()?.try_deserialize();

        // Set server port statically
        if let Ok(config) = &parsed_config {
            SERVER_PORT.store(config.server.port, SeqCst);
        }

        // Return config
        parsed_config
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct KafkaConfiguration {
    pub broker: BrokerProperties,
    pub consumer: Vec<ConsumerConfiguration>,
    pub schema_registry: SchemaRegistryProperties,
    pub topic: TopicConfiguration,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BrokerProperties {
    pub urls: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ConsumerConfiguration {
    pub id: String,
    pub topic: Vec<String>,
    pub client_id: String,
    pub group_id: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct SchemaRegistryProperties {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct TopicConfiguration {
    pub mappings: Vec<TopicProperties>,
}

impl TopicConfiguration {
    pub fn get_mapping(&self, id: &str) -> TopicProperties {
        let mapping: Vec<TopicProperties> = self
            .mappings
            .clone()
            .into_iter()
            .filter(|t| t.id == id)
            .collect();

        mapping
            .first()
            .unwrap_or_else(|| panic!("{} topic configuration not found", id))
            .clone()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct TopicProperties {
    pub id: String,
    pub topic_name: String,
    pub partitions: i32,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct LoggingConfiguration {
    pub level: LogLevelConfiguration,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct LogLevelConfiguration {
    pub root: Option<String>,
    pub directives: Vec<LoggingDirective>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct LoggingDirective {
    pub namespace: String,
    pub level: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ServerConfiguration {
    pub port: u16,
}
