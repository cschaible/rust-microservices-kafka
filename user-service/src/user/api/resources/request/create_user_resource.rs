use serde::Deserialize;

use super::create_phone_number_resource::CreatePhoneNumberResource;
use crate::common::model::IsoCountryCodeEnum;
use crate::user::service::dto::CreatePhoneNumberDto;
use crate::user::service::dto::CreateUserDto;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserResource {
    pub name: String,
    pub email: String,
    pub country: IsoCountryCodeEnum,
    pub phone_numbers: Option<Vec<CreatePhoneNumberResource>>,
}

impl From<CreateUserResource> for CreateUserDto {
    fn from(r: CreateUserResource) -> Self {
        Self {
            name: r.name,
            email: r.email,
            country: r.country,
        }
    }
}

impl From<CreateUserResource> for Option<Vec<CreatePhoneNumberDto>> {
    fn from(resource: CreateUserResource) -> Self {
        resource
            .phone_numbers
            .map(|r| r.into_iter().map(|p| p.into()).collect())
    }
}
