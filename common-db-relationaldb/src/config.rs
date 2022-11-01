use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DatabaseConfiguration {
    pub url: String,
    pub connection: DatabaseConnectionProperties,
    pub logging: DatabaseLoggingProperties,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DatabaseLoggingProperties {
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DatabaseConnectionProperties {
    pub pool: DatabasePoolProperties,
    pub connect_timeout: Option<u64>,
    pub idle_timeout: Option<u64>,
    pub max_lifetime: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DatabasePoolProperties {
    pub min: Option<u32>,
    pub max: Option<u32>,
}
