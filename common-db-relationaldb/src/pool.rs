use std::time::Duration;

use common_error::AppError;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;

use crate::config::DatabaseConfiguration;

pub async fn init(config: &DatabaseConfiguration) -> Result<DatabaseConnection, AppError> {
    let mut opt = ConnectOptions::new(config.url.clone());

    // Configure pool

    if let Some(max) = config.connection.pool.max {
        opt.max_connections(max);
    }

    if let Some(min) = config.connection.pool.min {
        opt.min_connections(min);
    }

    // Configure SQLx Logging

    if let Some(logging_enabled) = config.logging.enabled {
        opt.sqlx_logging(logging_enabled);
    }

    // Configure timeouts

    if let Some(connect_timeout) = config.connection.connect_timeout {
        opt.connect_timeout(Duration::from_secs(connect_timeout));
    }

    if let Some(idle_timeout) = config.connection.idle_timeout {
        opt.idle_timeout(Duration::from_secs(idle_timeout));
    }

    if let Some(max_lifetime) = config.connection.max_lifetime {
        opt.max_lifetime(Duration::from_secs(max_lifetime));
    }

    // Create pool

    Ok(Database::connect(opt).await?)
}
