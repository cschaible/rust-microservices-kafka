use serde::Deserialize;

use crate::user::model::phone_number::PhoneNumberTypeEnum;
use crate::user::service::dto::CreatePhoneNumberDto;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePhoneNumberResource {
    pub country_code: String,
    pub phone_number_type: PhoneNumberTypeEnum,
    pub call_number: String,
}

impl From<CreatePhoneNumberResource> for CreatePhoneNumberDto {
    fn from(resource: CreatePhoneNumberResource) -> Self {
        CreatePhoneNumberDto {
            country_code: resource.country_code,
            phone_number_type: resource.phone_number_type,
            call_number: resource.call_number,
        }
    }
}
