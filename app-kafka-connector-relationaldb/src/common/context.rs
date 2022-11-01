use std::sync::Arc;

use sea_orm::DatabaseConnection;

pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn db_connection(&self) -> Arc<DatabaseConnection>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub db: Arc<DatabaseConnection>,
}

impl ContextImpl {
    pub fn new_dyn_context(connection_pool: Arc<DatabaseConnection>) -> DynContext {
        let context = ContextImpl {
            db: connection_pool,
        };
        let context: DynContext = Arc::new(context);
        context
    }
}

impl Context for ContextImpl {
    fn db_connection(&self) -> Arc<DatabaseConnection> {
        self.db.clone()
    }
}
