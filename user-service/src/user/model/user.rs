use sea_orm::entity::prelude::*;
use serde::Serialize;

use crate::common::model::IsoCountryCodeEnum;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveActiveModelBehavior, Serialize)]
#[sea_orm(table_name = "user_entity")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub version: i64,
    #[sea_orm(unique, indexed)]
    pub identifier: Uuid,
    pub name: String,
    pub email: String,
    pub country: IsoCountryCodeEnum,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::phone_number::Entity")]
    PhoneNumber,
}

impl Related<super::phone_number::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PhoneNumber.def()
    }
}
