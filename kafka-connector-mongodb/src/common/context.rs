use std::sync::Arc;

use common_error::AppError;
use mongodb::error::UNKNOWN_TRANSACTION_COMMIT_RESULT;
use mongodb::Client;
use mongodb::ClientSession;

pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn db_client(&self) -> Arc<Client>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub client: Arc<Client>,
}

impl Context for ContextImpl {
    fn db_client(&self) -> Arc<Client> {
        self.client.clone()
    }
}

pub struct TransactionalContext {
    client: Arc<Client>,
    db_session: ClientSession,
}

impl TransactionalContext {
    fn new(db_session: ClientSession, client: Arc<Client>) -> TransactionalContext {
        TransactionalContext { client, db_session }
    }

    pub async fn from_context(context: &DynContext) -> Result<TransactionalContext, AppError> {
        let db_client = context.db_client().clone();
        let mut db_session = db_client.as_ref().start_session(None).await?;

        db_session.start_transaction(None).await?;

        let transactional_context = Self::new(db_session, db_client);
        Ok(transactional_context)
    }

    pub fn db_client(&self) -> Arc<Client> {
        self.client.clone()
    }
}

pub async fn commit_context<'a>(
    transactional_context: &mut TransactionalContext,
) -> Result<(), AppError> {
    loop {
        let result = transactional_context.db_session.commit_transaction().await;
        if let Err(error) = result {
            if !error.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                return Err(error.into());
            }
            // else retry
        } else {
            return Ok(());
        }
    }
}

pub async fn rollback_context<'a>(
    transactional_context: &mut TransactionalContext,
) -> Result<(), AppError> {
    Ok(transactional_context.db_session.abort_transaction().await?)
}
