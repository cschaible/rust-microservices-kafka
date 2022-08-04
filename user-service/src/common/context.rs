use std::sync::Arc;

use crate::event::service::dto::{EventDto, SerializableEventDto};
use common_error::AppError;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use tokio::sync::Mutex;

use crate::event::service::event_dispatcher::EventDispatcher;

pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn db_connection(&self) -> Arc<DatabaseConnection>;
    fn event_dispatcher(&self) -> Arc<Mutex<EventDispatcher>>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub db: Arc<DatabaseConnection>,
    pub(crate) event_dispatcher: Arc<Mutex<EventDispatcher>>,
}

impl Context for ContextImpl {
    fn db_connection(&self) -> Arc<DatabaseConnection> {
        self.db.clone()
    }

    fn event_dispatcher(&self) -> Arc<Mutex<EventDispatcher>> {
        self.event_dispatcher.clone()
    }
}

pub struct TransactionalContext {
    db_transaction: DatabaseTransaction,
    event_dispatcher: Arc<Mutex<EventDispatcher>>,
}

impl TransactionalContext {
    fn new(
        event_dispatcher: Arc<Mutex<EventDispatcher>>,
        db_transaction: DatabaseTransaction,
    ) -> TransactionalContext {
        TransactionalContext {
            db_transaction,
            event_dispatcher,
        }
    }

    pub async fn from_context(context: &DynContext) -> Result<TransactionalContext, AppError> {
        let event_dispatcher = context.event_dispatcher().clone();
        let db_transaction = context.db_connection().clone().as_ref().begin().await?;

        let transactional_context = Self::new(event_dispatcher, db_transaction);
        Ok(transactional_context)
    }

    pub fn db_connection(&self) -> &DatabaseTransaction {
        &self.db_transaction
    }

    pub async fn dispatch_events(
        &mut self,
        event: Box<dyn SerializableEventDto>,
    ) -> Result<Vec<EventDto>, AppError> {
        self.event_dispatcher.lock().await.dispatch(event).await
    }
}

pub async fn commit_context<'a>(
    transactional_context: TransactionalContext,
) -> Result<(), AppError> {
    Ok(transactional_context.db_transaction.commit().await?)
}

pub async fn rollback_context<'a>(
    transactional_context: TransactionalContext,
) -> Result<(), AppError> {
    Ok(transactional_context.db_transaction.rollback().await?)
}
