use std::any::Any;

use kafka_schema_user::schema_create_user::SCHEMA_NAME_CREATE_USER;

use crate::event::service::dto::SerializableEventDto;
use crate::user::model::phone_number;
use crate::user::model::user;

#[derive(Clone)]
pub struct UserWithPhoneNumbersDto {
    pub user: user::Model,
    pub phone_numbers: Option<Vec<phone_number::ActiveModel>>,
}

impl SerializableEventDto for UserWithPhoneNumbersDto {
    fn event_type(&self) -> String {
        SCHEMA_NAME_CREATE_USER.to_owned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
