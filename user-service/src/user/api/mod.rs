use axum::extract::Extension;
use axum::extract::Path;
use axum::extract::Query;
use axum::Json;
use common_error::AppError;
use common_error::DbError;
use futures::FutureExt;
use sea_orm::ActiveValue;
use tracing::instrument;
use uuid::Uuid;

use self::resources::response::build_user_resource;
use self::resources::response::build_user_resource_page_from_page;
use self::resources::response::build_user_resource_page_from_vec;
use self::resources::response::build_user_resources;
use super::service::phone_number_service;
use crate::common::context::DynContext;
use crate::common::db::transactional2;
use crate::common::paging::Page;
use crate::common::paging::PageParams;
use crate::event::service::dto::SerializableEventDto;
use crate::event::service::event_service;
use crate::user::api::resources::request::create_user_resource::CreateUserResource;
use crate::user::api::resources::response::user_resource::UserResource;
use crate::user::event::dto::UserWithPhoneNumbersDto;
use crate::user::model::phone_number;
use crate::user::model::user;
use crate::user::service::user_service;

pub mod resources;
pub mod routing;

#[instrument(name = "user.api.create_user", skip_all)]
pub async fn create_user<'a>(
    Json(create_user_resource): Json<CreateUserResource>,
    Extension(context): Extension<DynContext>,
) -> Result<Json<UserResource>, AppError> {
    transactional2(context, |tx| {
        let mut user: user::ActiveModel = create_user_resource.clone().into();
        let phone_numbers: Option<Vec<phone_number::ActiveModel>> =
            create_user_resource.clone().into();

        Box::pin(async move {
            user.identifier = ActiveValue::set(Uuid::new_v4());
            let user = user_service::create_user(tx, &user).await?;

            let phone_numbers = if let Some(numbers) = phone_numbers {
                // Set references to user
                let updated_numbers: Vec<phone_number::ActiveModel> = numbers
                    .into_iter()
                    .map(|mut n| {
                        n.user_id = ActiveValue::set(user.id);
                        n
                    })
                    .collect();

                // Save phone numbers in database
                phone_number_service::save(tx.db_connection(), updated_numbers.clone()).await?;

                Some(updated_numbers)
            } else {
                None
            };

            let dto: Box<dyn SerializableEventDto> = Box::new(UserWithPhoneNumbersDto {
                user: user.clone(),
                phone_numbers,
            });

            let events = tx.dispatch_events(dto).await?;
            for event in events {
                event_service::save(tx, &event).await?;
            }

            Ok(build_user_resource(tx.db_connection(), user).await?.into())
        })
    })
    .await
}

#[instrument(name = "user.api.find_one_by_identifier", skip(context))]
pub async fn find_one_by_identifier(
    Path(identifier): Path<Uuid>,
    Extension(context): Extension<DynContext>,
) -> Result<Json<UserResource>, AppError> {
    transactional2(context, |tx| {
        Box::pin(async move {
            match user_service::find_one_by_identifier(tx.db_connection(), identifier).await {
                Ok(found_user) => match found_user {
                    Some(user) => Ok(build_user_resource(tx.db_connection(), user).await?.into()),
                    None => Err(AppError::DbError(DbError::NotFound)),
                },
                Err(e) => Err(e.into()),
            }
        })
    })
    .await
}

#[instrument(name = "user.api.find_all", skip(context))]
pub async fn find_all(
    Query(page_params): Query<PageParams>,
    Extension(context): Extension<DynContext>,
) -> Result<Json<Page<UserResource>>, AppError> {
    transactional2(context, |tx| {
        let params = page_params.clone();

        Box::pin(async move {
            if let (Some(_page), Some(_page_size)) = (params.page, params.page_size) {
                match user_service::find_all_paged(tx.db_connection(), params).await {
                    Ok(users) => Ok(
                        build_user_resource_page_from_page(tx.db_connection(), users)
                            .await?
                            .into(),
                    ),
                    Err(e) => Err(e.into()),
                }
            } else {
                match user_service::find_all(tx.db_connection()).await {
                    Ok(users) => {
                        let resource_page =
                            build_user_resource_page_from_vec(tx.db_connection(), users).await?;
                        let json: Json<Page<UserResource>> = resource_page.into();
                        Ok(json)
                    }
                    Err(e) => Err(e.into()),
                }
            }
        })
    })
    .await
}

#[instrument(name = "user.api.find_all_by_identifiers", skip_all)]
pub async fn find_all_by_identifiers(
    Json(user_identifiers): Json<Vec<Uuid>>,
    Extension(context): Extension<DynContext>,
) -> Result<Json<Vec<UserResource>>, AppError> {
    transactional2(context, |tx| {
        let identifiers = user_identifiers.clone();

        async move {
            match user_service::find_all_by_identifiers(tx.db_connection(), identifiers).await {
                Ok(users) => Ok(build_user_resources(tx.db_connection(), users)
                    .await?
                    .into()),
                Err(e) => Err(e.into()),
            }
        }
        .boxed()
    })
    .await
}
