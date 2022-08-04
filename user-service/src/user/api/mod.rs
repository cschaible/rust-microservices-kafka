use common_error::{AppError, DbError};
use futures::FutureExt;

use axum::{
    extract::{Extension, Path, Query},
    Json,
};

use tracing::instrument;
use uuid::Uuid;

use crate::common::context::TransactionalContext;
use crate::event::service::dto::SerializableEventDto;
use crate::user::api::resources::request::create_user_resource::CreateUserResource;
use crate::user::api::resources::response::user_resource::UserResource;
use crate::user::service::dto::CreateUserDto;
use crate::user::service::user_service;
use crate::{
    common::{
        context::DynContext,
        db::{transactional, transactional2},
        paging::{Page, PageParams},
    },
    event::service::event_service,
};

use self::resources::response::{
    build_user_resource, build_user_resource_page_from_page, build_user_resource_page_from_vec,
    build_user_resources,
};

use super::service::{
    dto::{CreatePhoneNumberDto, UserWithPhoneNumbersDto},
    phone_number_service,
};

pub mod resources;
pub mod routing;

#[instrument(name = "user.api.create_user", skip_all)]
pub async fn create_user<'a>(
    Json(create_user_resource): Json<CreateUserResource>,
    Extension(context): Extension<DynContext>,
) -> Result<Json<UserResource>, AppError> {
    transactional2(context, |tx_context: &mut TransactionalContext| {
        let user_dto: CreateUserDto = create_user_resource.clone().into();
        let phone_number_dtos: Option<Vec<CreatePhoneNumberDto>> =
            create_user_resource.clone().into();

        Box::pin(async move {
            let user = user_service::create_user(tx_context, &user_dto).await?;
            let phone_numbers = if let Some(phone_numbers) = phone_number_dtos {
                phone_number_service::save(
                    tx_context.db_connection(),
                    user.identifier,
                    phone_numbers,
                )
                .await?;

                Some(
                    phone_number_service::find_all_by_user_identifier(
                        tx_context.db_connection(),
                        user.identifier,
                    )
                    .await?,
                )
            } else {
                None
            };

            let dto: Box<dyn SerializableEventDto> = Box::new(UserWithPhoneNumbersDto {
                user: user.clone(),
                phone_numbers,
            });

            let events = tx_context.dispatch_events(dto).await?;
            for event in events {
                event_service::save(tx_context, &event).await?;
            }

            Ok(build_user_resource(tx_context.db_connection(), user)
                .await?
                .into())
        })
    })
    .await
}

#[instrument(name = "user.api.find_one_by_identifier", skip(context))]
pub async fn find_one_by_identifier(
    Path(identifier): Path<Uuid>,
    Extension(context): Extension<DynContext>,
) -> Result<Json<UserResource>, AppError> {
    transactional(context, |_context, tx| {
        Box::pin(async move {
            match user_service::find_one_by_identifier(tx, identifier).await {
                Ok(found_user) => match found_user {
                    Some(user) => Ok(build_user_resource(tx, user).await?.into()),
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
    transactional(context, |_context, tx| {
        let params = page_params.clone();

        Box::pin(async move {
            if let (Some(_page), Some(_page_size)) = (params.page, params.page_size) {
                match user_service::find_all_paged(tx, params).await {
                    Ok(users) => Ok(build_user_resource_page_from_page(tx, users).await?.into()),
                    Err(e) => Err(e.into()),
                }
            } else {
                match user_service::find_all(tx).await {
                    Ok(users) => {
                        let resource_page = build_user_resource_page_from_vec(tx, users).await?;
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
    transactional(context, |_context, tx| {
        let identifiers = user_identifiers.clone();

        async move {
            match user_service::find_all_by_identifiers(tx, identifiers).await {
                Ok(users) => Ok(build_user_resources(tx, users).await?.into()),
                Err(e) => Err(e.into()),
            }
        }
        .boxed()
    })
    .await
}
