use std::pin::Pin;
use std::sync::Arc;

use common_error::AppError;
use futures::Future;
use sea_orm::DatabaseConnection;
use sea_orm::DatabaseTransaction;
use sea_orm::TransactionTrait;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn transactional<R, F>(
    db_connection: Arc<DatabaseConnection>,
    f: F,
) -> Result<R, AppError>
where
    R: 'static,
    F: for<'c> Fn(
        &'c DatabaseTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<R, AppError>> + Send + 'c>>,
{
    // Start transaction
    let transaction = db_connection.begin().await?;

    // Invoke actual function
    let result = f(&transaction).await;

    // Commit or rollback transaction
    if result.is_ok() {
        transaction.commit().await?
    } else {
        transaction.rollback().await?;
    }

    // Return result of the closure
    result
}
