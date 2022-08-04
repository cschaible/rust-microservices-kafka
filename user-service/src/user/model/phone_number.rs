use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(10))")]
pub enum PhoneNumberTypeEnum {
    #[sea_orm(string_value = "Business")]
    Business,
    #[sea_orm(string_value = "Home")]
    Home,
    #[sea_orm(string_value = "Mobile")]
    Mobile,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveActiveModelBehavior, Serialize)]
#[sea_orm(table_name = "phone_number")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub country_code: String,
    pub phone_number_type: PhoneNumberTypeEnum,
    pub call_number: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
