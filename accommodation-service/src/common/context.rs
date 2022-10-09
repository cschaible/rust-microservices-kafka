use std::sync::Arc;

use common_error::AppError;
use mongodb::error::UNKNOWN_TRANSACTION_COMMIT_RESULT;
use mongodb::Client;
use mongodb::ClientSession;
use tokio::sync::Mutex;

use crate::common::kafka::RecordDecoder;
use crate::event::service::dto::EventDto;
use crate::event::service::dto::SerializableEventDto;
use crate::event::service::event_dispatcher::EventDispatcher;

pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn avro_decoder(&self) -> Arc<Mutex<dyn RecordDecoder>>;
    fn db_client(&self) -> Arc<Client>;
    fn event_dispatcher(&self) -> Arc<Mutex<EventDispatcher>>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub avro_decoder: Arc<Mutex<dyn RecordDecoder>>,
    pub client: Arc<Client>,
    pub event_dispatcher: Arc<Mutex<EventDispatcher>>,
}

impl Context for ContextImpl {
    fn avro_decoder(&self) -> Arc<Mutex<dyn RecordDecoder>> {
        self.avro_decoder.clone()
    }

    fn db_client(&self) -> Arc<Client> {
        self.client.clone()
    }

    fn event_dispatcher(&self) -> Arc<Mutex<EventDispatcher>> {
        self.event_dispatcher.clone()
    }
}

pub struct TransactionalContext {
    client: Arc<Client>,
    db_session: ClientSession,
    event_dispatcher: Arc<Mutex<EventDispatcher>>,
}

impl TransactionalContext {
    fn new(
        event_dispatcher: Arc<Mutex<EventDispatcher>>,
        db_session: ClientSession,
        client: Arc<Client>,
    ) -> TransactionalContext {
        TransactionalContext {
            client,
            db_session,
            event_dispatcher,
        }
    }

    pub async fn from_context(context: &DynContext) -> Result<TransactionalContext, AppError> {
        let event_dispatcher = context.event_dispatcher().clone();
        let db_client = context.db_client().clone();
        let mut db_session = db_client.as_ref().start_session(None).await?;

        db_session.start_transaction(None).await?;

        let transactional_context = Self::new(event_dispatcher, db_session, db_client);
        Ok(transactional_context)
    }

    // pub fn db_session(&mut self) -> &mut ClientSession {
    // &mut self.db_session
    // }

    pub fn db_client(&self) -> Arc<Client> {
        self.client.clone()
    }

    pub async fn dispatch_events(
        &mut self,
        event_type: String,
        event: Box<dyn SerializableEventDto>,
    ) -> Result<Vec<EventDto>, AppError> {
        self.event_dispatcher
            .lock()
            .await
            .dispatch(event_type, event)
            .await
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
