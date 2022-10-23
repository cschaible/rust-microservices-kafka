use std::pin::Pin;
use std::time::Duration;

use common_error::AppError;
use futures::Future;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use tracing::instrument;

use super::context::commit_context;
use super::context::rollback_context;
use super::context::DynContext;
use super::context::TransactionalContext;
use crate::config::configuration::DatabaseConfiguration;

pub const MAX_PAGE_SIZE: usize = 500;

pub async fn init_pool(config: &DatabaseConfiguration) -> Result<DatabaseConnection, AppError> {
    let mut opt = ConnectOptions::new(config.url.clone());

    // Configure pool

    if let Some(max) = config.connection.pool.max {
        opt.max_connections(max);
    }

    if let Some(min) = config.connection.pool.min {
        opt.max_connections(min);
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
        commit_context(transactional_context).await?;
    } else {
        rollback_context(transactional_context).await?;
    }

    // Return result of the closure
    result
}
