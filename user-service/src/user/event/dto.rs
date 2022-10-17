use std::any::Any;

use kafka_schema_user::schema_create_user::CreateUserAvro;
use kafka_schema_user::IsoCountryCodeEnumAvro;
use kafka_schema_user::PhoneNumberAvro;
use kafka_schema_user::PhoneNumberTypeEnumAvro;

use crate::common::model::IsoCountryCodeEnum;
use crate::event::service::dto::SerializableEventDto;
use crate::user::model::phone_number;
use crate::user::model::phone_number::PhoneNumberTypeEnum;
use crate::user::model::user;

#[derive(Clone)]
pub struct UserWithPhoneNumbersDto {
    pub user: user::Model,
    pub phone_numbers: Option<Vec<phone_number::ActiveModel>>,
}

impl SerializableEventDto for UserWithPhoneNumbersDto {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl From<phone_number::ActiveModel> for PhoneNumberAvro {
    fn from(phone_number: phone_number::ActiveModel) -> Self {
        PhoneNumberAvro {
            country_code: phone_number.country_code.clone().unwrap(),
            call_number: phone_number.call_number.clone().unwrap(),
            phone_number_type: match phone_number.phone_number_type.unwrap() {
                PhoneNumberTypeEnum::Business => PhoneNumberTypeEnumAvro::Business,
                PhoneNumberTypeEnum::Home => PhoneNumberTypeEnumAvro::Home,
                PhoneNumberTypeEnum::Mobile => PhoneNumberTypeEnumAvro::Mobile,
            },
        }
    }
}

impl From<UserWithPhoneNumbersDto> for CreateUserAvro {
    fn from(dto: UserWithPhoneNumbersDto) -> CreateUserAvro {
        CreateUserAvro {
            identifier: format!("{}", dto.user.identifier),
            name: dto.user.name,
            email: dto.user.email,
            country: match dto.user.country {
                IsoCountryCodeEnum::DE => IsoCountryCodeEnumAvro::DE,
                IsoCountryCodeEnum::US => IsoCountryCodeEnumAvro::US,
            },
            phone_numbers: match dto.phone_numbers {
                Some(phone_numbers) => phone_numbers.into_iter().map(|p| p.into()).collect(),
                None => Vec::with_capacity(0),
            },
        }
    }
}
