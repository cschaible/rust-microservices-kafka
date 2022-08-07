use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveActiveModelBehavior, Serialize)]
#[sea_orm(table_name = "event_entity")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub topic: String,
    pub partition: i32,
    pub key: Vec<u8>,
    pub payload: Vec<u8>,
    pub trace_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
