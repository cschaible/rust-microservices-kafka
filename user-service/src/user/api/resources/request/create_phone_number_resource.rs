use sea_orm::ActiveValue;
use sea_orm::ActiveValue::NotSet;
use serde::Deserialize;

use crate::user::model::phone_number;
use crate::user::model::phone_number::PhoneNumberTypeEnum;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePhoneNumberResource {
    pub country_code: String,
    pub phone_number_type: PhoneNumberTypeEnum,
    pub call_number: String,
}

impl From<CreatePhoneNumberResource> for phone_number::ActiveModel {
    fn from(resource: CreatePhoneNumberResource) -> Self {
        phone_number::ActiveModel {
            id: NotSet,
            user_id: NotSet,
            country_code: ActiveValue::set(resource.country_code.clone()),
            phone_number_type: ActiveValue::set(resource.phone_number_type.clone()),
            call_number: ActiveValue::set(resource.call_number),
        }
    }
}
