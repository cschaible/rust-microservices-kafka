use std::sync::Arc;

use mongodb::Client;

use crate::common::kafka::RecordDecoder;
use crate::event::service::event_dispatcher::EventDispatcher;

pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn avro_decoder(&self) -> Arc<dyn RecordDecoder>;
    fn db_client(&self) -> Arc<Client>;
    fn event_dispatcher(&self) -> Arc<EventDispatcher>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub avro_decoder: Arc<dyn RecordDecoder>,
    pub client: Arc<Client>,
    pub event_dispatcher: Arc<EventDispatcher>,
}

impl ContextImpl {
    pub fn new_dyn_context(
        avro_decoder: Arc<dyn RecordDecoder>,
        client: Arc<Client>,
        event_dispatcher: Arc<EventDispatcher>,
    ) -> DynContext {
        let context = ContextImpl {
            avro_decoder,
            client,
            event_dispatcher,
        };
        let context: DynContext = Arc::new(context);
        context
    }
}

impl Context for ContextImpl {
    fn avro_decoder(&self) -> Arc<dyn RecordDecoder> {
        self.avro_decoder.clone()
    }

    fn db_client(&self) -> Arc<Client> {
        self.client.clone()
    }

    fn event_dispatcher(&self) -> Arc<EventDispatcher> {
        self.event_dispatcher.clone()
    }
}
