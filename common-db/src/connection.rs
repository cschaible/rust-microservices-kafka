use std::sync::Arc;

use sea_orm::DatabaseConnection;

pub type DbConnectionPool = Box<dyn DbPool>;

pub trait DbPool {
    fn db_connection(&self) -> Arc<DatabaseConnection>;
}
