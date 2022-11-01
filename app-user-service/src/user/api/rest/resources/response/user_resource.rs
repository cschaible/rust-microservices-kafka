use serde::Serialize;
use uuid::Uuid;

use super::phone_number_resource::PhoneNumberResource;
use crate::common::model::IsoCountryCodeEnum;
use crate::user::model::projections::PhoneNumberUserIdentifierProjection;
use crate::user::model::user;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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

impl
    From<(
        user::Model,
        Option<Vec<PhoneNumberUserIdentifierProjection>>,
    )> for UserResource
{
    fn from(
        tuple: (
            user::Model,
            Option<Vec<PhoneNumberUserIdentifierProjection>>,
        ),
    ) -> Self {
        let user = tuple.0;
        let phone_numbers: Option<Vec<PhoneNumberResource>> = tuple
            .1
            .map(|phone_number| phone_number.into_iter().map(|n| n.into()).collect());

        UserResource {
            identifier: user.identifier,
            version: user.version,
            name: user.name,
            email: user.email,
            country: user.country,
            phone_numbers,
        }
    }
}
