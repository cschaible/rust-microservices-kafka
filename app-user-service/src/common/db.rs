use std::sync::Arc;

use common_error::AppError;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;

use crate::Migrator;

pub async fn migrate(connection: Arc<DatabaseConnection>) -> Result<(), AppError> {
    match Migrator::up(connection.as_ref(), None).await {
        Ok(_) => {
            tracing::debug!("Finished migration steps");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to apply migration: {:?}", e);
            Err(AppError::RelDbUnhandledDbError(e))
        }
    }
}
