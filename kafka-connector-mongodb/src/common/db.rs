use std::env::VarError;
use std::fmt::Debug;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use common_error::AppError;
use futures::Future;
use mongodb::options::ClientOptions;
use mongodb::Client;
use mongodb::Collection;

use super::context::commit_context;
use super::context::rollback_context;
use super::context::DynContext;
use super::context::TransactionalContext;

pub const MAX_PAGE_SIZE: usize = 500;

pub async fn init_db_client() -> anyhow::Result<Client> {
    let connection_string = std::env::var("DATABASE_CONNECTION_STRING")
        .expect("Environment variable 'DATABASE_CONNECTION_STRING' not set");
    let max_pool_size = std::env::var("DATABASE_MAX_POOL_SIZE");
    let min_pool_size = std::env::var("DATABASE_MIN_POOL_SIZE");
    let connect_timeout = std::env::var("DATABASE_CONNECT_TIMEOUT");
    let max_idle_time = std::env::var("DATABASE_MAX_IDLE_TIME");

    let mut opt = ClientOptions::parse(connection_string).await?;

    if max_pool_size.is_ok() {
        opt.max_pool_size = Some(parse_variable::<u32>(
            max_pool_size,
            "Invalid max pool size provided",
        ));
    }

    if min_pool_size.is_ok() {
        opt.min_pool_size = Some(parse_variable(
            min_pool_size,
            "Invalid min pool size provided",
        ));
    }

    if connect_timeout.is_ok() {
        opt.connect_timeout = Some(Duration::from_secs(parse_variable(
            connect_timeout,
            "Invalid connect timeout value provided",
        )));
    }

    if max_idle_time.is_ok() {
        opt.max_idle_time = Some(Duration::from_secs(parse_variable(
            max_idle_time,
            "Invalid max idle timeout value provided",
        )));
    }

    Ok(Client::with_options(opt)?)
}

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
        commit_context(&mut transactional_context).await?;
    } else {
        rollback_context(&mut transactional_context).await?;
    }
    result
}

fn parse_variable<T>(variable: Result<String, VarError>, error_message: &str) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    variable.unwrap().trim().parse::<T>().expect(error_message)
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
