use sea_orm_migration::prelude::*;

use crate::event::model::event;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220703_add_event_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_event_table(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop event table
        Ok(manager
            .drop_table(Table::drop().table(event::Entity).to_owned())
            .await?)
    }
}

async fn create_event_table<'m>(manager: &'m SchemaManager<'m>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(event::Entity)
                .if_not_exists()
                .col(
                    ColumnDef::new(event::Column::Id)
                        .primary_key()
                        .integer()
                        .auto_increment()
                        .not_null(),
                )
                .col(ColumnDef::new(event::Column::Topic).string().not_null())
                .col(
                    ColumnDef::new(event::Column::Partition)
                        .integer()
                        .not_null(),
                )
                .col(ColumnDef::new(event::Column::Key).binary().not_null())
                .col(ColumnDef::new(event::Column::Payload).binary().not_null())
                .to_owned(),
        )
        .await?;

    Ok(())
}
