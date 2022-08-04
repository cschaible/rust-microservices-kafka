use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveActiveModelBehavior)]
#[sea_orm(table_name = "event_entity")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub topic: String,
    pub partition: i32,
    pub key: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
