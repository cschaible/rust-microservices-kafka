use serde::Serialize;

use crate::user::model::phone_number;
use crate::user::model::phone_number::PhoneNumberTypeEnum;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumberResource {
    pub country_code: String,
    pub phone_number_type: PhoneNumberTypeEnum,
    pub call_number: String,
}

impl From<phone_number::Model> for PhoneNumberResource {
    fn from(p: phone_number::Model) -> Self {
        PhoneNumberResource {
            country_code: p.country_code,
            phone_number_type: p.phone_number_type,
            call_number: p.call_number,
        }
    }
}
