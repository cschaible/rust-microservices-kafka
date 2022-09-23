use std::env::VarError;
use std::fmt::Debug;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use common_error::AppError;
use futures::Future;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use sea_orm::DatabaseTransaction;
use sea_orm::TransactionTrait;
use tracing::instrument;

use super::context::commit_context;
use super::context::rollback_context;
use super::context::DynContext;
use super::context::TransactionalContext;

pub const MAX_PAGE_SIZE: usize = 500;

pub async fn init_db_pool() -> DatabaseConnection {
    let url = std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");
    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS");
    let min_connections = std::env::var("DATABASE_MIN_CONNECTIONS");
    let connect_timeout = std::env::var("DATABASE_CONNECT_TIMEOUT");
    let idle_timeout = std::env::var("DATABASE_IDLE_TIMEOUT");
    let max_lifetime = std::env::var("DATABASE_MAX_LIFETIME");
    let logging = std::env::var("DATABASE_LOGGING");

    let mut opt = ConnectOptions::new(url.to_owned());

    if max_connections.is_ok() {
        opt.max_connections(parse_variable(
            max_connections,
            "Invalid max connection count provided",
        ));
    }

    if min_connections.is_ok() {
        opt.min_connections(parse_variable(
            min_connections,
            "Invalid min connection count provided",
        ));
    }

    if logging.is_ok() {
        opt.sqlx_logging(parse_variable(logging, "Invalid logging flag provided"));
    }

    if connect_timeout.is_ok() {
        opt.connect_timeout(Duration::from_secs(parse_variable(
            connect_timeout,
            "Invalid connect timeout value provided",
        )));
    }

    if idle_timeout.is_ok() {
        opt.idle_timeout(Duration::from_secs(parse_variable(
            idle_timeout,
            "Invalid idle timeout value provided",
        )));
    }

    if max_lifetime.is_ok() {
        opt.max_lifetime(Duration::from_secs(parse_variable(
            max_lifetime,
            "Invalid max lifetime value provided",
        )));
    }

    Database::connect(opt)
        .await
        .expect("Establishing database connection failed")
}

#[instrument(skip_all)]
pub async fn transactional<R, F>(context: DynContext, f: F) -> Result<R, AppError>
where
    R: 'static,
    F: for<'c> Fn(
        &'c DynContext,
        &'c DatabaseTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<R, AppError>> + Send + 'c>>,
{
    let transaction = context.db_connection().as_ref().begin().await?;

    let result = f(&context, &transaction).await;

    if result.is_ok() {
        commit_transaction(transaction).await?;
    } else {
        transaction.rollback().await?;
    }
    result
}

#[instrument(skip_all)]
pub async fn transactional2<R, F>(context: DynContext, f: F) -> Result<R, AppError>
where
    R: 'static,
    F: for<'c> Fn(
        &'c mut TransactionalContext,
    ) -> Pin<Box<dyn Future<Output = Result<R, AppError>> + Send + 'c>>,
{
    let mut transactional_context = TransactionalContext::from_context(&context).await?;

    let result = f(&mut transactional_context).await;

    if result.is_ok() {
        commit_context(transactional_context).await?;
    } else {
        rollback_context(transactional_context).await?;
    }
    result
}

#[instrument(skip_all)]
async fn commit_transaction(transaction: DatabaseTransaction) -> Result<(), AppError> {
    Ok(transaction.commit().await?)
}

fn parse_variable<T>(variable: Result<String, VarError>, error_message: &str) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    variable.unwrap().trim().parse::<T>().expect(error_message)
}
