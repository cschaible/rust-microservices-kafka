use std::pin::Pin;
use std::sync::Arc;

use common_error::AppError;
use futures::Future;
use mongodb::error::UNKNOWN_TRANSACTION_COMMIT_RESULT;
use mongodb::options::Acknowledgment;
use mongodb::options::ReadConcern;
use mongodb::options::TransactionOptions;
use mongodb::options::WriteConcern;
use mongodb::Client;
use mongodb::ClientSession;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn transactional<R, F>(db_client: Arc<Client>, f: F) -> Result<R, AppError>
where
    R: 'static,
    F: for<'c> Fn(
        &'c ClientSession,
    ) -> Pin<Box<dyn Future<Output = Result<R, AppError>> + Send + 'c>>,
{
    // Start client session
    let mut db_session = db_client.as_ref().start_session(None).await?;

    // Start transaction
    let options = TransactionOptions::builder()
        .read_concern(ReadConcern::majority())
        .write_concern(WriteConcern::builder().w(Acknowledgment::Majority).build())
        .build();

    db_session.start_transaction(options).await?;

    // Invoke closure with transactional context
    let result = f(&db_session).await;

    // Commit or rollback transaction
    if result.is_ok() {
        commit_transaction(&mut db_session).await?;
    } else {
        rollback_transaction(&mut db_session).await?;
    }

    // Return result of the closure
    result
}

pub async fn commit_transaction<'a>(db_session: &mut ClientSession) -> Result<(), AppError> {
    loop {
        let result = db_session.commit_transaction().await;
        if let Err(error) = result {
            if !error.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                return Err(error.into());
            }
        } else {
            return Ok(());
        }
    }
}

pub async fn rollback_transaction<'a>(db_session: &mut ClientSession) -> Result<(), AppError> {
    Ok(db_session.abort_transaction().await?)
}
