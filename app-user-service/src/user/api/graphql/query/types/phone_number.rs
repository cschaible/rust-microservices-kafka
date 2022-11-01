use async_graphql::Object;

use crate::user::api::graphql::shared::types::PhoneNumberType;
use crate::user::model::phone_number::PhoneNumberTypeEnum;
use crate::user::model::projections::PhoneNumberUserIdentifierProjection;

/// A type of phone number including properties.
pub struct PhoneNumberPayload(pub PhoneNumberUserIdentifierProjection);

#[Object]
impl PhoneNumberPayload {
    /// Country code
    async fn country_code(&self) -> String {
        self.0.country_code.clone()
    }

    /// Call number
    async fn call_number(&self) -> String {
        self.0.call_number.clone()
    }

    /// Type of phone number
    async fn phone_number_type(&self) -> PhoneNumberType {
        self.0.phone_number_type.clone().into()
    }
}

impl From<PhoneNumberTypeEnum> for PhoneNumberType {
    fn from(p: PhoneNumberTypeEnum) -> Self {
        match p {
            PhoneNumberTypeEnum::Business => PhoneNumberType::Business,
            PhoneNumberTypeEnum::Home => PhoneNumberType::Home,
            PhoneNumberTypeEnum::Mobile => PhoneNumberType::Mobile,
        }
    }
}

impl From<PhoneNumberType> for PhoneNumberTypeEnum {
    fn from(p: PhoneNumberType) -> Self {
        match p {
            PhoneNumberType::Business => PhoneNumberTypeEnum::Business,
            PhoneNumberType::Home => PhoneNumberTypeEnum::Home,
            PhoneNumberType::Mobile => PhoneNumberTypeEnum::Mobile,
        }
    }
}
