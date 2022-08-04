use std::pin::Pin;

use common_error::AppError;
use futures::Future;
use sea_orm::{DatabaseTransaction, TransactionTrait};
use tracing::instrument;

use crate::connection::DbConnectionPool;

#[instrument(skip_all)]
pub async fn transactional<R, F>(pool: DbConnectionPool, f: F) -> Result<R, AppError>
where
    R: 'static,
    F: for<'c> Fn(
        &'c DbConnectionPool,
        &'c DatabaseTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<R, AppError>> + Send + 'c>>,
{
    let transaction = pool.db_connection().as_ref().begin().await?;

    let result = f(&pool, &transaction).await;

    if result.is_ok() {
        commit_transaction(transaction).await?;
    } else {
        transaction.rollback().await?;
    }
    result
}

#[instrument(skip_all)]
async fn commit_transaction(transaction: DatabaseTransaction) -> Result<(), AppError> {
    Ok(transaction.commit().await?)
}
