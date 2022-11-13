use serde::Deserialize;
use serde::Serialize;

use crate::IsoCountryCodeEnumAvro;
use crate::PhoneNumberAvro;

pub const SCHEMA_NAME_CREATE_USER: &str = "CreateUserAvroV1";

pub const RAW_SCHEMA_CREATE_USER_V1: &str = include_str!("../resources/create_user_v1.avsc");

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserAvro {
    pub identifier: String,
    pub name: String,
    pub email: String,
    pub country: IsoCountryCodeEnumAvro,
    pub phone_numbers: Vec<PhoneNumberAvro>,
}
