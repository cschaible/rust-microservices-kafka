use serde::{Deserialize, Serialize};

pub mod schema_create_user;

pub const DATA_TYPE_USER: &str = "user";

#[derive(Debug, Deserialize, Serialize)]
pub enum IsoCountryCodeEnumAvro {
    DE,
    US,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumberAvro {
    pub country_code: String,
    pub phone_number_type: PhoneNumberTypeEnumAvro,
    pub call_number: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PhoneNumberTypeEnumAvro {
    Business,
    Home,
    Mobile,
}
