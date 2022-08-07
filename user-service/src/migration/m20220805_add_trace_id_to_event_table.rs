use sea_orm_migration::prelude::*;

use crate::event::model::event;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220853_add_trace_id_to_event_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add trace_id column
        manager
            .alter_table(
                Table::alter()
                    .table(event::Entity)
                    .add_column(ColumnDef::new(event::Column::TraceId).string())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop trace_id column
        Ok(manager
            .alter_table(
                Table::alter()
                    .table(event::Entity)
                    .drop_column(event::Column::TraceId)
                    .to_owned(),
            )
            .await?)
    }
}
