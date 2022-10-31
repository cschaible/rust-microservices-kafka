use std::sync::Arc;

use mongodb::Client;

pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn db_client(&self) -> Arc<Client>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub client: Arc<Client>,
}

impl ContextImpl {
    pub fn new_dyn_context(client: Arc<Client>) -> DynContext {
        let context = ContextImpl { client };
        let context: DynContext = Arc::new(context);
        context
    }
}

impl Context for ContextImpl {
    fn db_client(&self) -> Arc<Client> {
        self.client.clone()
    }
}
