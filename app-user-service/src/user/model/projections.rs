use sea_orm::FromQueryResult;
use uuid::Uuid;

use crate::user::model::phone_number::PhoneNumberTypeEnum;

#[derive(Clone, FromQueryResult)]
pub struct PhoneNumberUserIdentifierProjection {
    pub id: i64,
    pub user_identifier: Uuid,
    pub country_code: String,
    pub phone_number_type: PhoneNumberTypeEnum,
    pub call_number: String,
}
