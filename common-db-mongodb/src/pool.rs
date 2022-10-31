use std::time::Duration;

use common_error::AppError;
use mongodb::options::ClientOptions;
use mongodb::Client;

use crate::config::DatabaseConfiguration;

pub async fn init_db_client(config: &DatabaseConfiguration) -> Result<Client, AppError> {
    let mut opt = ClientOptions::parse(config.url.clone()).await?;

    // Configure pool

    if let Some(max) = config.connection.pool.max {
        opt.max_pool_size = Some(max);
    }

    if let Some(min) = config.connection.pool.min {
        opt.min_pool_size = Some(min);
    }

    // Configure timeouts

    if let Some(connect_timeout) = config.connection.connect_timeout {
        opt.connect_timeout = Some(Duration::from_secs(connect_timeout));
    }

    if let Some(idle_timeout) = config.connection.idle_timeout {
        opt.max_idle_time = Some(Duration::from_secs(idle_timeout));
    }

    // Instantiate client

    Ok(Client::with_options(opt)?)
}
