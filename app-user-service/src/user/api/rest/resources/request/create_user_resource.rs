use sea_orm::ActiveValue;
use serde::Deserialize;

use super::create_phone_number_resource::CreatePhoneNumberResource;
use crate::common::model::IsoCountryCodeEnum;
use crate::user::model::phone_number;
use crate::user::model::user;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserResource {
    pub name: String,
    pub email: String,
    pub country: IsoCountryCodeEnum,
    pub phone_numbers: Option<Vec<CreatePhoneNumberResource>>,
}

impl From<CreateUserResource> for user::ActiveModel {
    fn from(r: CreateUserResource) -> user::ActiveModel {
        user::ActiveModel {
            id: ActiveValue::NotSet,
            identifier: ActiveValue::NotSet,
            version: ActiveValue::set(0),
            name: ActiveValue::set(r.name.clone()),
            email: ActiveValue::set(r.email.clone()),
            country: ActiveValue::set(r.country),
        }
    }
}

impl From<CreateUserResource> for Option<Vec<phone_number::ActiveModel>> {
    fn from(resource: CreateUserResource) -> Self {
        resource
            .phone_numbers
            .map(|r| r.into_iter().map(|p| p.into()).collect())
    }
}
