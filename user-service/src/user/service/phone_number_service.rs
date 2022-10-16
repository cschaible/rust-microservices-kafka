use std::collections::HashMap;

use anyhow::Result;
use common_error::AppError;
use itertools::Itertools;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::JoinType;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::RelationTrait;
use tracing::instrument;
use uuid::Uuid;

use crate::user::model::phone_number;
use crate::user::model::phone_number::Entity as PhoneNumberEntity;
use crate::user::model::user;

#[instrument(name = "phone_number_service.save", skip_all)]
pub async fn save<T: ConnectionTrait + Sized>(
    connection: &T,
    phone_numbers: Vec<phone_number::ActiveModel>,
) -> Result<(), AppError> {
    if !phone_numbers.is_empty() {
        phone_number::Entity::insert_many(phone_numbers)
            .exec(connection)
            .await?;
    }

    Ok(())
}

#[instrument(name = "phone_number_service.find_all_by_user_identifiers", skip_all)]
pub async fn find_all_by_user_identifiers<T: ConnectionTrait + Sized>(
    connection: &T,
    user_identifiers: Vec<Uuid>,
) -> Result<HashMap<i64, Vec<phone_number::Model>>> {
    let phone_numbers = PhoneNumberEntity::find()
        .join(JoinType::InnerJoin, phone_number::Relation::User.def())
        .filter(user::Column::Identifier.is_in(user_identifiers))
        .all(connection)
        .await?;

    Ok(phone_numbers.into_iter().into_group_map_by(|p| p.user_id))
}
