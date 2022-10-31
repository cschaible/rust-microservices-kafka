use std::collections::HashMap;

use anyhow::Result;
use common_error::AppError;
use itertools::Itertools;
use sea_orm::entity::*;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseTransaction;
use sea_orm::DeriveColumn;
use sea_orm::EntityTrait;
use sea_orm::EnumIter;
use sea_orm::JoinType;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::RelationTrait;
use tracing::instrument;
use uuid::Uuid;

use crate::user::model::phone_number;
use crate::user::model::phone_number::Entity as PhoneNumberEntity;
use crate::user::model::projections::PhoneNumberUserIdentifierProjection;
use crate::user::model::user;

#[instrument(name = "phone_number_service.save", skip_all)]
pub async fn save(
    db_connection: &DatabaseTransaction,
    phone_numbers: Vec<phone_number::ActiveModel>,
) -> Result<(), AppError> {
    if !phone_numbers.is_empty() {
        phone_number::Entity::insert_many(phone_numbers)
            .exec(db_connection)
            .await?;
    }

    Ok(())
}

#[instrument(name = "phone_number_service.find_all_by_user_identifiers", skip_all)]
pub async fn find_all_by_user_identifiers(
    db_connection: &DatabaseTransaction,
    user_identifiers: Vec<Uuid>,
) -> Result<HashMap<Uuid, Vec<PhoneNumberUserIdentifierProjection>>> {
    let phone_numbers = PhoneNumberEntity::find()
        .join(JoinType::InnerJoin, phone_number::Relation::User.def())
        .filter(user::Column::Identifier.is_in(user_identifiers))
        .column_as(user::Column::Identifier, QueryAs::UserIdentifier)
        .into_model::<PhoneNumberUserIdentifierProjection>()
        .all(db_connection)
        .await?;

    Ok(phone_numbers
        .into_iter()
        .into_group_map_by(|p| p.user_identifier))
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    UserIdentifier,
}
