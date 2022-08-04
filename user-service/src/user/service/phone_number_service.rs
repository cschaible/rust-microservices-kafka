use std::collections::HashMap;

use anyhow::Result;
use common_error::{AppError, DbError};
use itertools::Itertools;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait,
    Set,
};
use uuid::Uuid;

use crate::user::model::{phone_number, phone_number::Entity as PhoneNumberEntity, user};

use super::{
    dto::{CreatePhoneNumberDto, PhoneNumberDto},
    user_service,
};

pub async fn save<T: ConnectionTrait + Sized>(
    connection: &T,
    user_identifier: Uuid,
    phone_numbers: Vec<CreatePhoneNumberDto>,
) -> Result<(), AppError> {
    let user = match user_service::find_one_by_identifier(connection, user_identifier).await? {
        Some(u) => Ok(u),
        None => Err(AppError::DbError(DbError::NotFound)),
    }?;

    // Delete previously existing phone numbers
    delete_all_by_user_identifier(connection, user_identifier).await?;

    if !phone_numbers.is_empty() {
        // Convert the dtos into entities
        let numbers: Vec<phone_number::ActiveModel> = (&phone_numbers)
            .iter()
            .map(|p| phone_number::ActiveModel {
                user_id: Set(user.id),
                country_code: Set(p.country_code.clone()),
                phone_number_type: Set(p.phone_number_type.clone()),
                call_number: Set(p.call_number.clone()),
                ..Default::default()
            })
            .collect();

        // Save the entities
        phone_number::Entity::insert_many(numbers)
            .exec(connection)
            .await?;
    }
    Ok(())
}

pub async fn find_all_by_user_identifier<T: ConnectionTrait + Sized>(
    connection: &T,
    user_identifier: Uuid,
) -> Result<Vec<PhoneNumberDto>> {
    Ok(PhoneNumberEntity::find()
        .join(JoinType::InnerJoin, phone_number::Relation::User.def())
        .filter(user::Column::Identifier.eq(user_identifier))
        .all(connection)
        .await?
        .into_iter()
        .map(|p| p.into())
        .collect())
}

pub async fn delete_all_by_user_identifier<T: ConnectionTrait + Sized>(
    connection: &T,
    user_identifier: Uuid,
) -> Result<(), AppError> {
    let user = match user_service::find_one_by_identifier(connection, user_identifier).await? {
        Some(u) => Ok(u),
        None => Err(AppError::DbError(DbError::NotFound)),
    }?;

    phone_number::Entity::delete_many()
        .filter(phone_number::Column::Id.eq(user.id))
        .exec(connection)
        .await?;
    Ok(())
}

pub async fn find_all_by_user_identifiers<T: ConnectionTrait + Sized>(
    connection: &T,
    user_identifiers: Vec<Uuid>,
) -> Result<HashMap<i64, Vec<PhoneNumberDto>>> {
    let phone_numbers = PhoneNumberEntity::find()
        .join(JoinType::InnerJoin, phone_number::Relation::User.def())
        .filter(user::Column::Identifier.is_in(user_identifiers))
        .all(connection)
        .await?;

    let mut phone_numbers_by_user_id = HashMap::new();
    for (key, group) in &phone_numbers.into_iter().group_by(|p| p.user_id) {
        phone_numbers_by_user_id.insert(key, group.map(|p| p.into()).collect());
    }

    Ok(phone_numbers_by_user_id)
}
