use serde::Serialize;
use uuid::Uuid;

use super::phone_number_resource::PhoneNumberResource;
use crate::common::model::IsoCountryCodeEnum;
use crate::user::service::dto::PhoneNumberDto;
use crate::user::service::dto::UserDto;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResource {
    pub identifier: Uuid,
    pub version: i64,
    pub name: String,
    pub email: String,
    pub country: IsoCountryCodeEnum,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_numbers: Option<Vec<PhoneNumberResource>>,
}

impl From<(UserDto, Option<Vec<PhoneNumberDto>>)> for UserResource {
    fn from(tuple: (UserDto, Option<Vec<PhoneNumberDto>>)) -> Self {
        let dto = tuple.0;
        let phone_number_resources: Option<Vec<PhoneNumberResource>> = tuple
            .1
            .map(|dtos| dtos.into_iter().map(|d| d.into()).collect());

        UserResource {
            identifier: dto.identifier,
            version: dto.version,
            name: dto.name,
            email: dto.email,
            country: dto.country,
            phone_numbers: phone_number_resources,
        }
    }
}
