use kafka_schema_user::schema_create_user::SCHEMA_NAME_CREATE_USER;
use serde::Serialize;
use std::any::Any;
use uuid::Uuid;

use crate::{
    common::model::IsoCountryCodeEnum,
    event::service::dto::SerializableEventDto,
    user::model::{
        phone_number::{self, PhoneNumberTypeEnum},
        user,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
    pub country: IsoCountryCodeEnum,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UserDto {
    pub id: i64,
    pub version: i64,
    pub identifier: Uuid,
    pub name: String,
    pub email: String,
    pub country: IsoCountryCodeEnum,
}

impl From<user::Model> for UserDto {
    fn from(user: user::Model) -> Self {
        UserDto {
            id: user.id,
            version: user.version,
            identifier: user.identifier,
            name: user.name,
            email: user.email,
            country: user.country,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreatePhoneNumberDto {
    pub country_code: String,
    pub phone_number_type: PhoneNumberTypeEnum,
    pub call_number: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhoneNumberDto {
    pub user_id: i64,
    pub country_code: String,
    pub phone_number_type: PhoneNumberTypeEnum,
    pub call_number: String,
}

impl From<phone_number::Model> for PhoneNumberDto {
    fn from(p: phone_number::Model) -> Self {
        PhoneNumberDto {
            user_id: p.user_id,
            country_code: p.country_code,
            phone_number_type: p.phone_number_type,
            call_number: p.call_number,
        }
    }
}

#[derive(Clone)]
pub struct UserWithPhoneNumbersDto {
    pub user: UserDto,
    pub phone_numbers: Option<Vec<PhoneNumberDto>>,
}

impl SerializableEventDto for UserWithPhoneNumbersDto {
    fn event_type(&self) -> String {
        SCHEMA_NAME_CREATE_USER.to_owned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
