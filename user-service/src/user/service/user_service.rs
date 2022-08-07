use anyhow::Result;

use common_error::AppError;
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, QueryOrder, Set};
use tracing::instrument;

use crate::common::context::TransactionalContext;
use crate::common::paging::{Page, PageParams};

use super::super::{model::user, model::user::Entity as UserEntity};

use super::dto::{CreateUserDto, UserDto};

#[instrument(name = "user_service.create_user", skip_all)]
pub async fn create_user(
    tx_context: &TransactionalContext,
    user: &CreateUserDto,
) -> Result<UserDto, AppError> {
    // Build the entity from dto
    let u = user::ActiveModel {
        identifier: Set(Uuid::new_v4()),
        version: Set(0),
        name: Set(user.name.clone()),
        email: Set(user.email.clone()),
        country: Set(user.country.clone()),
        ..Default::default()
    };

    tracing::debug!("Save user with identifier: {:?}", u.identifier);

    // Save entity
    let saved_user = u.insert(tx_context.db_connection()).await?;

    Ok(saved_user.into())
}

#[instrument(name = "user_service.find_one_by_identifier", skip(connection))]
pub async fn find_one_by_identifier<'a, T: ConnectionTrait + Sized>(
    connection: &T,
    identifier: Uuid,
) -> Result<Option<UserDto>> {
    let user_with_number = UserEntity::find()
        .filter(user::Column::Identifier.eq(identifier))
        .all(connection)
        .await?;

    Ok(user_with_number.first().map(|user| user.to_owned().into()))
}

#[instrument(name = "user_service.find_all", skip_all)]
pub async fn find_all<T: ConnectionTrait + Sized>(connection: &T) -> Result<Vec<UserDto>> {
    Ok(UserEntity::find()
        .order_by_asc(user::Column::Identifier)
        .all(connection)
        .await?
        .into_iter()
        .map(|u| u.into())
        .collect())
}

#[instrument(name = "user_service.find_all_paged", skip(connection))]
pub async fn find_all_paged<T: ConnectionTrait + Sized>(
    connection: &T,
    page_params: PageParams,
) -> Result<Page<UserDto>> {
    // Extract page parameters from the request
    let page_size = page_params.page_size.unwrap_or(0);
    let page = page_params.page.unwrap_or(0);

    // Count query
    let total_elements = UserEntity::find().count(connection).await?;

    // Find query
    let items: Vec<UserDto> = UserEntity::find()
        .order_by_asc(user::Column::Identifier)
        .paginate(connection, page_size)
        .fetch_page(page)
        .await?
        .into_iter()
        .map(|u| u.into())
        .collect();

    // Build response
    let size = items.len();
    Ok(Page {
        items,
        page: Some(page),
        size: Some(size),
        total_elements: Some(total_elements),
        total_pages: Some((total_elements as f32 / page_size as f32).ceil() as usize),
    })
}

#[instrument(name = "user_service.find_all_by_identifiers", skip(connection))]
pub async fn find_all_by_identifiers<T: ConnectionTrait + Sized>(
    connection: &T,
    user_identifiers: Vec<Uuid>,
) -> Result<Vec<UserDto>> {
    Ok(UserEntity::find()
        .filter(user::Column::Identifier.is_in(user_identifiers))
        .order_by_asc(user::Column::Identifier)
        .all(connection)
        .await?
        .into_iter()
        .map(|u| u.into())
        .collect())
}
