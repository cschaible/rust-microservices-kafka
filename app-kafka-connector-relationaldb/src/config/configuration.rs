use std::env;

use common_db_relationaldb::config::DatabaseConfiguration;
use config::Config;
use config::ConfigError;
use config::File;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Configuration {
    pub database: DatabaseConfiguration,
    pub kafka: KafkaConfiguration,
    pub logging: LoggingConfiguration,
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

        // Return config
        parsed_config
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct KafkaConfiguration {
    pub broker: BrokerProperties,
    pub producer: ProducerProperties,
    pub schema_registry: SchemaRegistryProperties,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BrokerProperties {
    pub urls: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ProducerProperties {
    pub client_id: String,
    pub transactional_id: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct SchemaRegistryProperties {
    pub url: String,
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
