use std::pin::Pin;
use std::time::Duration;

use common_error::AppError;
use futures::Future;
use mongodb::options::ClientOptions;
use mongodb::Client;
use mongodb::Collection;
use tracing::instrument;

use super::context::commit_context;
use super::context::rollback_context;
use super::context::DynContext;
use super::context::TransactionalContext;
use crate::config::configuration::DatabaseConfiguration;

pub const MAX_PAGE_SIZE: usize = 500;

pub async fn init_db_client(config: &DatabaseConfiguration) -> anyhow::Result<Client> {
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

#[instrument(skip_all)]
pub async fn transactional2<R, F>(context: DynContext, f: F) -> Result<R, AppError>
where
    R: 'static,
    F: for<'c> Fn(
        &'c mut TransactionalContext,
    ) -> Pin<Box<dyn Future<Output = Result<R, AppError>> + Send + 'c>>,
{
    // Initialize transactional context and start transaction
    let mut transactional_context = TransactionalContext::from_context(&context).await?;

    // Invoke closure with transactional context
    let result = f(&mut transactional_context).await;

    // Commit or rollback transaction
    if result.is_ok() {
        commit_context(&mut transactional_context).await?;
    } else {
        rollback_context(&mut transactional_context).await?;
    }

    // Return result of the closure
    result
}

pub fn get_collection<T>(
    tx_context: &TransactionalContext,
    collection_name: &str,
) -> Collection<T> {
    let database = tx_context
        .db_client()
        .default_database()
        .expect("No default db specified");

    database.collection::<T>(collection_name)
}
