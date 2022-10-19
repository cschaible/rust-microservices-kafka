use serde::Serialize;

use crate::user::model::phone_number::PhoneNumberTypeEnum;
use crate::user::model::projections::PhoneNumberUserIdentifierProjection;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumberResource {
    pub country_code: String,
    pub phone_number_type: PhoneNumberTypeEnum,
    pub call_number: String,
}

impl From<PhoneNumberUserIdentifierProjection> for PhoneNumberResource {
    fn from(p: PhoneNumberUserIdentifierProjection) -> Self {
        PhoneNumberResource {
            country_code: p.country_code,
            phone_number_type: p.phone_number_type,
            call_number: p.call_number,
        }
    }
}
