use sea_orm::entity::prelude::*;
use sea_orm_migration::{MigrationTrait, MigratorTrait};

mod m20220403_initial_migration;
mod m20220703_add_event_table;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "seaql_migrations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub version: String,
    pub applied_at: i64,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220403_initial_migration::Migration),
            Box::new(m20220703_add_event_table::Migration),
        ]
    }
}
