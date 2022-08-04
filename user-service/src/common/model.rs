use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Deserialize, Serialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(2))")]
pub enum IsoCountryCodeEnum {
    #[sea_orm(string_value = "DE")]
    DE,
    #[sea_orm(string_value = "US")]
    US,
}

impl IsoCountryCodeEnum {
    pub fn country_name(&self) -> String {
        match self {
            IsoCountryCodeEnum::DE => "Germany".to_string(),
            IsoCountryCodeEnum::US => "United States of America".to_string(),
        }
    }
}
