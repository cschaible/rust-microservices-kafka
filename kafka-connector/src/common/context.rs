use std::sync::Arc;

use common_error::AppError;
use sea_orm::DatabaseConnection;
use sea_orm::DatabaseTransaction;
use sea_orm::TransactionTrait;

pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn db_connection(&self) -> Arc<DatabaseConnection>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub db: Arc<DatabaseConnection>,
}

impl Context for ContextImpl {
    fn db_connection(&self) -> Arc<DatabaseConnection> {
        self.db.clone()
    }
}

pub struct TransactionalContext {
    db_transaction: DatabaseTransaction,
}

impl TransactionalContext {
    fn new(db_transaction: DatabaseTransaction) -> TransactionalContext {
        TransactionalContext { db_transaction }
    }

    pub async fn from_context(context: &DynContext) -> Result<TransactionalContext, AppError> {
        let db_transaction = context.db_connection().clone().as_ref().begin().await?;

        let transactional_context = Self::new(db_transaction);
        Ok(transactional_context)
    }

    pub fn db_connection(&self) -> &DatabaseTransaction {
        &self.db_transaction
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
