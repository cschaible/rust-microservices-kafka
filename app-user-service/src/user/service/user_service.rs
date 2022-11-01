use anyhow::Result;
use common_error::AppError;
use sea_orm::entity::prelude::*;
use sea_orm::DatabaseTransaction;
use sea_orm::QueryOrder;
use tracing::instrument;

use super::super::model::user;
use super::super::model::user::Entity as UserEntity;
use crate::common::paging::Page;
use crate::common::paging::PageParams;

#[instrument(name = "user_service.create_user", skip_all)]
pub async fn create_user(
    db_connection: &DatabaseTransaction,
    user: &user::ActiveModel,
) -> Result<user::Model, AppError> {
    tracing::debug!("Save user with identifier: {:?}", user.identifier);
    Ok(user.clone().insert(db_connection).await?)
}

#[instrument(name = "user_service.find_one_by_identifier", skip(db_connection))]
pub async fn find_one_by_identifier(
    db_connection: &DatabaseTransaction,
    identifier: Uuid,
) -> Result<Option<user::Model>> {
    Ok(UserEntity::find()
        .filter(user::Column::Identifier.eq(identifier))
        .one(db_connection)
        .await?)
}

#[instrument(name = "user_service.find_all", skip_all)]
pub async fn find_all(db_connection: &DatabaseTransaction) -> Result<Vec<user::Model>> {
    Ok(UserEntity::find()
        .order_by_asc(user::Column::Identifier)
        .all(db_connection)
        .await?)
}

#[instrument(name = "user_service.find_all_paged", skip(db_connection))]
pub async fn find_all_paged(
    db_connection: &DatabaseTransaction,
    page_params: PageParams,
) -> Result<Page<user::Model>> {
    // Extract page parameters from the request
    let page_size = page_params.page_size.unwrap_or(0);
    let page = page_params.page.unwrap_or(0);

    // Count query
    let total_elements = UserEntity::find().count(db_connection).await?;

    // Find query
    let users: Vec<user::Model> = UserEntity::find()
        .order_by_asc(user::Column::Identifier)
        .paginate(db_connection, page_size)
        .fetch_page(page)
        .await?;

    // Build response
    let size = users.len();
    Ok(Page {
        items: users,
        page: Some(page),
        size: Some(size),
        total_elements: Some(total_elements),
        total_pages: Some((total_elements as f32 / page_size as f32).ceil() as usize),
    })
}

#[instrument(name = "user_service.find_all_by_identifiers", skip(db_connection))]
pub async fn find_all_by_identifiers(
    db_connection: &DatabaseTransaction,
    user_identifiers: Vec<Uuid>,
) -> Result<Vec<user::Model>> {
    Ok(UserEntity::find()
        .filter(user::Column::Identifier.is_in(user_identifiers))
        .order_by_asc(user::Column::Identifier)
        .all(db_connection)
        .await?)
}
