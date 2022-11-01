use sea_orm_migration::prelude::*;

use crate::user::model::phone_number;
use crate::user::model::user;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220403_initial_migration"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_user_table(manager).await?;
        create_phone_number_table(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop phone number table
        manager
            .drop_table(Table::drop().table(phone_number::Entity).to_owned())
            .await?;

        // Drop user table
        manager
            .drop_table(Table::drop().table(user::Entity).to_owned())
            .await
    }
}

async fn create_user_table<'m>(manager: &'m SchemaManager<'m>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(user::Entity)
                .if_not_exists()
                .col(
                    ColumnDef::new(user::Column::Id)
                        .primary_key()
                        .big_integer()
                        .auto_increment()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(user::Column::Version)
                        .big_integer()
                        .not_null(),
                )
                .col(ColumnDef::new(user::Column::Country).string().not_null())
                .col(ColumnDef::new(user::Column::Email).string().not_null())
                .col(ColumnDef::new(user::Column::Identifier).uuid().not_null())
                .col(ColumnDef::new(user::Column::Name).string().not_null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            sea_query::Index::create()
                .name("IX_User_Identifier")
                .table(user::Entity)
                .col(user::Column::Identifier)
                .to_owned(),
        )
        .await?;

    Ok(())
}

async fn create_phone_number_table<'m>(manager: &'m SchemaManager<'m>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(phone_number::Entity)
                .if_not_exists()
                .col(
                    ColumnDef::new(phone_number::Column::Id)
                        .primary_key()
                        .big_integer()
                        .auto_increment()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(phone_number::Column::UserId)
                        .big_integer()
                        .not_null(),
                )
                .col(ColumnDef::new(phone_number::Column::CallNumber).string())
                .col(ColumnDef::new(phone_number::Column::CountryCode).string())
                .col(ColumnDef::new(phone_number::Column::PhoneNumberType).string())
                .to_owned(),
        )
        .await?;

    manager
        .create_foreign_key(
            sea_query::ForeignKey::create()
                .name("FK_Phone_Number_User")
                .from(phone_number::Entity, phone_number::Column::UserId)
                .to(user::Entity, user::Column::Id)
                .to_owned(),
        )
        .await?;

    Ok(())
}
