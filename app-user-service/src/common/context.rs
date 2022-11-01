use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::event::service::event_dispatcher::EventDispatcher;

pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn db_connection(&self) -> Arc<DatabaseConnection>;
    fn event_dispatcher(&self) -> Arc<EventDispatcher>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub db: Arc<DatabaseConnection>,
    pub event_dispatcher: Arc<EventDispatcher>,
}

impl ContextImpl {
    pub fn new_dyn_context(
        connection_pool: Arc<DatabaseConnection>,
        event_dispatcher: Arc<EventDispatcher>,
    ) -> DynContext {
        let context = ContextImpl {
            db: connection_pool,
            event_dispatcher,
        };
        let context: DynContext = Arc::new(context);
        context
    }
}

impl Context for ContextImpl {
    fn db_connection(&self) -> Arc<DatabaseConnection> {
        self.db.clone()
    }

    fn event_dispatcher(&self) -> Arc<EventDispatcher> {
        self.event_dispatcher.clone()
    }
}
